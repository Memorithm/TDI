//! Exact target-blind symbolic readout candidate for bounded TDI-8.1.
//!
//! The symbolic executor requires a concrete arm adapter to return one [`TaskSymbol`]
//! at each query, while the merged A1/A2/A3 references currently expose only their
//! binary64 recurrent state. This module qualifies a minimal readout boundary:
//! two caller-selected recurrent-state coordinates must themselves contain the exact
//! canonical two-limb encoding of the predicted `u64` symbol.
//!
//! The readout receives only arm state. It has no target, source index, task-family
//! collision metadata, candidate vocabulary, nearest-neighbour table, learned decoder,
//! tolerance, or rounding path. Coordinate indices are caller supplied and this module
//! deliberately provides no experimental defaults.
//!
//! This is bounded software-preflight infrastructure, not a frozen TDI-8.1 readout
//! choice and not H8-A/H8-B evidence.

use core::fmt;

use crate::task_encoding::{EXACT_U64_BINARY64_WIDTH, ExactU64Binary64, TaskEncodingError};
use crate::task_generators::TaskSymbol;

/// Caller-selected exact-symbol readout positions inside recurrent state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExactStateReadoutLayout {
    state_width: u64,
    high_limb_index: u64,
    low_limb_index: u64,
}

impl ExactStateReadoutLayout {
    /// Validate a non-empty state and two distinct in-range limb coordinates.
    pub fn new(
        state_width: u64,
        high_limb_index: u64,
        low_limb_index: u64,
    ) -> Result<Self, TaskReadoutError> {
        if state_width < EXACT_U64_BINARY64_WIDTH as u64 {
            return Err(TaskReadoutError::StateWidthTooSmall {
                minimum: EXACT_U64_BINARY64_WIDTH as u64,
                actual: state_width,
            });
        }
        let _ = usize::try_from(state_width)
            .map_err(|_| TaskReadoutError::HostDimensionTooLarge { value: state_width })?;
        if high_limb_index >= state_width {
            return Err(TaskReadoutError::ReadoutIndexOutOfBounds {
                index: high_limb_index,
                state_width,
            });
        }
        if low_limb_index >= state_width {
            return Err(TaskReadoutError::ReadoutIndexOutOfBounds {
                index: low_limb_index,
                state_width,
            });
        }
        if high_limb_index == low_limb_index {
            return Err(TaskReadoutError::DuplicateReadoutIndex {
                index: high_limb_index,
            });
        }
        Ok(Self {
            state_width,
            high_limb_index,
            low_limb_index,
        })
    }

    /// Caller-selected recurrent-state width.
    #[must_use]
    pub const fn state_width(self) -> u64 {
        self.state_width
    }

    /// State coordinate interpreted as the exact high 32-bit limb.
    #[must_use]
    pub const fn high_limb_index(self) -> u64 {
        self.high_limb_index
    }

    /// State coordinate interpreted as the exact low 32-bit limb.
    #[must_use]
    pub const fn low_limb_index(self) -> u64 {
        self.low_limb_index
    }

    fn host_indices(self) -> Result<(usize, usize, usize), TaskReadoutError> {
        let width = usize::try_from(self.state_width)
            .map_err(|_| TaskReadoutError::HostDimensionTooLarge {
                value: self.state_width,
            })?;
        let high = usize::try_from(self.high_limb_index).map_err(|_| {
            TaskReadoutError::HostDimensionTooLarge {
                value: self.high_limb_index,
            }
        })?;
        let low = usize::try_from(self.low_limb_index).map_err(|_| {
            TaskReadoutError::HostDimensionTooLarge {
                value: self.low_limb_index,
            }
        })?;
        Ok((width, high, low))
    }
}

/// Stateless exact-symbol readout using only recurrent-state coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExactStateSymbolReadout {
    layout: ExactStateReadoutLayout,
}

impl ExactStateSymbolReadout {
    /// Build a readout from an explicitly supplied layout.
    #[must_use]
    pub const fn new(layout: ExactStateReadoutLayout) -> Self {
        Self { layout }
    }

    /// Caller-selected layout. No default coordinate selection exists.
    #[must_use]
    pub const fn layout(self) -> ExactStateReadoutLayout {
        self.layout
    }

    /// Decode one predicted symbol from state and nothing else.
    ///
    /// The complete recurrent state must be finite. The two designated readout
    /// coordinates must additionally be canonical exact limbs; no clipping,
    /// rounding, tolerance or nearest-symbol fallback is permitted.
    pub fn decode_state(self, state: &[f64]) -> Result<TaskSymbol, TaskReadoutError> {
        let (width, high_index, low_index) = self.layout.host_indices()?;
        if state.len() != width {
            return Err(TaskReadoutError::StateWidthMismatch {
                expected: width,
                actual: state.len(),
            });
        }
        if let Some(index) = state.iter().position(|value| !value.is_finite()) {
            return Err(TaskReadoutError::NonFiniteState { index });
        }
        decode_exact_symbol_coordinates([state[high_index], state[low_index]])
    }
}

/// Decode the exact two-coordinate symbol representation used by A0 values or a
/// recurrent readout. This helper never receives a query target.
pub fn decode_exact_symbol_coordinates(
    coordinates: [f64; EXACT_U64_BINARY64_WIDTH],
) -> Result<TaskSymbol, TaskReadoutError> {
    ExactU64Binary64::decode(coordinates)
        .map(TaskSymbol::new)
        .map_err(TaskReadoutError::Encoding)
}

/// Fail-closed exact-readout errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskReadoutError {
    /// At least two recurrent coordinates are required for an exact `u64` readout.
    StateWidthTooSmall { minimum: u64, actual: u64 },
    /// A platform-independent dimension/index cannot be represented on this host.
    HostDimensionTooLarge { value: u64 },
    /// One requested limb coordinate lies outside recurrent state.
    ReadoutIndexOutOfBounds { index: u64, state_width: u64 },
    /// High and low limbs must occupy distinct state coordinates.
    DuplicateReadoutIndex { index: u64 },
    /// Runtime recurrent-state width drifted from the caller-selected layout.
    StateWidthMismatch { expected: usize, actual: usize },
    /// Recurrent state contains a non-finite coordinate.
    NonFiniteState { index: usize },
    /// Designated coordinates are not the canonical lossless symbol encoding.
    Encoding(TaskEncodingError),
}

impl fmt::Display for TaskReadoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TaskReadoutError {}

#[cfg(test)]
mod tests {
    use super::{
        ExactStateReadoutLayout, ExactStateSymbolReadout, TaskReadoutError,
        decode_exact_symbol_coordinates,
    };
    use crate::task_encoding::{ExactU64Binary64, TaskEncodingError};
    use crate::task_generators::TaskSymbol;

    #[test]
    fn exact_state_readout_round_trips_without_target_or_codebook() {
        let expected = 0xfedc_ba98_7654_3210u64;
        let encoded = ExactU64Binary64::encode(expected).coordinates();
        let state = [0.25, encoded[0], -0.5, encoded[1], 0.0];
        let layout = ExactStateReadoutLayout::new(5, 1, 3).expect("layout");
        let readout = ExactStateSymbolReadout::new(layout);
        assert_eq!(readout.decode_state(&state), Ok(TaskSymbol::new(expected)));
    }

    #[test]
    fn coordinate_selection_has_no_default_and_is_order_sensitive() {
        let expected = 0x0123_4567_89ab_cdefu64;
        let encoded = ExactU64Binary64::encode(expected).coordinates();
        let state = [encoded[1], 0.0, encoded[0]];
        let layout = ExactStateReadoutLayout::new(3, 2, 0).expect("layout");
        assert_eq!(
            ExactStateSymbolReadout::new(layout).decode_state(&state),
            Ok(TaskSymbol::new(expected))
        );
    }

    #[test]
    fn layout_rejects_small_duplicate_and_out_of_range_positions() {
        assert_eq!(
            ExactStateReadoutLayout::new(1, 0, 0),
            Err(TaskReadoutError::StateWidthTooSmall {
                minimum: 2,
                actual: 1,
            })
        );
        assert_eq!(
            ExactStateReadoutLayout::new(4, 2, 2),
            Err(TaskReadoutError::DuplicateReadoutIndex { index: 2 })
        );
        assert_eq!(
            ExactStateReadoutLayout::new(4, 4, 1),
            Err(TaskReadoutError::ReadoutIndexOutOfBounds {
                index: 4,
                state_width: 4,
            })
        );
    }

    #[test]
    fn runtime_width_and_non_finite_state_fail_closed() {
        let readout = ExactStateSymbolReadout::new(
            ExactStateReadoutLayout::new(3, 0, 1).expect("layout"),
        );
        assert_eq!(
            readout.decode_state(&[0.0, 0.0]),
            Err(TaskReadoutError::StateWidthMismatch {
                expected: 3,
                actual: 2,
            })
        );
        assert_eq!(
            readout.decode_state(&[0.0, 0.0, f64::NAN]),
            Err(TaskReadoutError::NonFiniteState { index: 2 })
        );
    }

    #[test]
    fn readout_rejects_noncanonical_limb_instead_of_rounding() {
        let readout = ExactStateSymbolReadout::new(
            ExactStateReadoutLayout::new(2, 0, 1).expect("layout"),
        );
        let error = readout
            .decode_state(&[0.1, 0.0])
            .expect_err("off-grid limb must fail");
        assert!(matches!(
            error,
            TaskReadoutError::Encoding(TaskEncodingError::NonCanonicalEncodedLimb {
                index: 0,
                ..
            })
        ));
    }

    #[test]
    fn a0_value_coordinates_share_the_same_exact_decoder() {
        let symbol = TaskSymbol::new(u64::MAX);
        let encoded = ExactU64Binary64::encode(symbol.code()).coordinates();
        assert_eq!(decode_exact_symbol_coordinates(encoded), Ok(symbol));
    }
}
