//! Bounded non-final replay adapters over real TDI libraries.
//!
//! `FiniteCycle` calls `tdi_core::TableSystem`; `JacobiSweep` calls
//! `tdi_operator::GreenBands`. No model, frozen population or hypothesis-specific
//! operator coefficients are involved. Their encoded checkpoints are complete
//! value snapshots; the Hub/TDI graph still owns plan identity and permissions.

use tdi_ai::adapter_sdk::{AdapterContract, ReplayCodec, Reproduction};
use tdi_ai::experiment::{ReplayAdapter, StepContext};
use tdi_core::{Action, State, TableSystem, TransitionSystem};
use tdi_operator::{GreenBands, JacobiMatrix};

/// Typed failure shared by these two bounded software adapters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterError {
    /// Unknown version, shape, numeric value or immutable configuration.
    InvalidCheckpoint,
    /// Unsupported stream, non-sequential absolute depth or exhausted horizon.
    InvalidContext,
    /// The underlying finite transition or positive-pivot operator rejected input.
    LibraryRejected,
}

/// Complete state for the fixed four-state transition table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FiniteCheckpoint {
    state: State,
    depth: usize,
}

/// Fixed `0→1→2→3→0` table evaluated by the actual finite-state library.
#[derive(Clone)]
pub struct FiniteCycle {
    table: TableSystem,
    checkpoint: FiniteCheckpoint,
}

impl FiniteCycle {
    /// Generate a bounded source with initial state `seed mod 4`; no hidden RNG.
    ///
    /// ```
    /// use tdi_bench::engine_adapters::FiniteCycle;
    /// use tdi_ai::experiment::{ReplayAdapter, StepContext};
    /// let mut a = FiniteCycle::new(3).unwrap();
    /// assert_eq!(a.advance(StepContext { depth: 1, noise_stream: 0 }).unwrap(), 0);
    /// ```
    pub fn new(seed: u64) -> Result<Self, AdapterError> {
        let mut table = TableSystem::new(2).map_err(|_| AdapterError::LibraryRejected)?;
        for bits in 0..4 {
            table
                .insert(
                    State::new(bits, 2).unwrap(),
                    Action::Noop,
                    vec![State::new((bits + 1) % 4, 2).unwrap()],
                )
                .map_err(|_| AdapterError::LibraryRejected)?;
        }
        Ok(Self {
            table,
            checkpoint: FiniteCheckpoint {
                state: State::new(seed % 4, 2).unwrap(),
                depth: 0,
            },
        })
    }

    /// Prepare an independent intervention by flipping one declared state bit.
    /// The original source is unchanged; node values outside 0..2 are rejected.
    pub fn flipped(&self, node: u8) -> Result<FiniteCheckpoint, AdapterError> {
        Ok(FiniteCheckpoint {
            state: self
                .checkpoint
                .state
                .flip(node)
                .map_err(|_| AdapterError::InvalidCheckpoint)?,
            depth: self.checkpoint.depth,
        })
    }
}

impl ReplayAdapter for FiniteCycle {
    type Checkpoint = FiniteCheckpoint;
    type Observation = u64;
    type Error = AdapterError;

    fn fork(&self, checkpoint: &FiniteCheckpoint) -> Result<Self, AdapterError> {
        if checkpoint.state.width() != 2 || checkpoint.depth > 64 {
            return Err(AdapterError::InvalidCheckpoint);
        }
        Ok(Self {
            table: self.table.clone(),
            checkpoint: checkpoint.clone(),
        })
    }
    fn checkpoint(&self) -> Result<FiniteCheckpoint, AdapterError> {
        Ok(self.checkpoint.clone())
    }
    fn advance(&mut self, context: StepContext) -> Result<u64, AdapterError> {
        if context.noise_stream != 0
            || self.checkpoint.depth >= 64
            || context.depth != self.checkpoint.depth + 1
        {
            return Err(AdapterError::InvalidContext);
        }
        let next = self
            .table
            .successors(self.checkpoint.state, Action::Noop)
            .map_err(|_| AdapterError::LibraryRejected)?;
        if next.len() != 1 {
            return Err(AdapterError::LibraryRejected);
        }
        self.checkpoint = FiniteCheckpoint {
            state: next[0],
            depth: context.depth,
        };
        Ok(next[0].bits())
    }
}

impl ReplayCodec for FiniteCycle {
    fn progress(&self) -> usize {
        self.checkpoint.depth
    }
    fn contract(&self) -> AdapterContract {
        AdapterContract {
            implementation: "tdi-finite-cycle/v1",
            advancement_unit: "finite transition",
            observation_unit: "state integer in [0,3]",
            precision: "exact u64",
            reproduction: Reproduction::ExactBuildTarget,
            rng: "none; seed selects initial state",
            max_checkpoint_bytes: 10,
            max_steps: 64,
            max_state_scalars: 10,
            access: "Development/Validation software fixtures only",
            execution_limits: "cooperative cancellation/deadline between bounded steps; no device or hostile-code isolation",
        }
    }
    fn encode_checkpoint(&self) -> Result<Vec<u8>, AdapterError> {
        let mut raw = b"TDIFCP1\0".to_vec();
        raw.push(self.checkpoint.state.bits() as u8);
        raw.push(self.checkpoint.depth as u8);
        Ok(raw)
    }
    fn decode_checkpoint(&self, bytes: &[u8]) -> Result<FiniteCheckpoint, AdapterError> {
        if bytes.len() != 10 || &bytes[..8] != b"TDIFCP1\0" || bytes[8] > 3 || bytes[9] > 64 {
            return Err(AdapterError::InvalidCheckpoint);
        }
        Ok(FiniteCheckpoint {
            state: State::new(bytes[8] as u64, 2).unwrap(),
            depth: bytes[9] as usize,
        })
    }
}

/// Complete Jacobi state, including immutable matrix and mutable cached diagonal.
#[derive(Clone, Debug, PartialEq)]
pub struct JacobiCheckpoint {
    matrix: JacobiMatrix,
    shift: f64,
    depth: usize,
    cached_diagonal: Vec<f64>,
}

/// Sweep the resolvent shift in steps of 0.25 through the actual operator library.
#[derive(Clone)]
pub struct JacobiSweep {
    checkpoint: JacobiCheckpoint,
}

impl JacobiSweep {
    /// Generate a 1..16 row finite matrix and initial shift. Construction checks
    /// dimensions and finiteness; positive-pivot failure is reported on advance.
    ///
    /// ```
    /// use tdi_bench::engine_adapters::JacobiSweep;
    /// use tdi_ai::experiment::{ReplayAdapter, StepContext};
    /// let mut a = JacobiSweep::new(vec![4.0,4.0], vec![1.0], 0.0).unwrap();
    /// let diagonal = a.advance(StepContext { depth: 1, noise_stream: 0 }).unwrap();
    /// assert_eq!(diagonal.len(), 2);
    /// assert!(JacobiSweep::new(vec![], vec![], 0.0).is_err());
    /// ```
    pub fn new(
        diagonal: Vec<f64>,
        off_diagonal: Vec<f64>,
        shift: f64,
    ) -> Result<Self, AdapterError> {
        if !(1..=16).contains(&diagonal.len()) || !shift.is_finite() {
            return Err(AdapterError::InvalidCheckpoint);
        }
        let matrix = JacobiMatrix::new(diagonal, off_diagonal)
            .map_err(|_| AdapterError::InvalidCheckpoint)?;
        Ok(Self {
            checkpoint: JacobiCheckpoint {
                matrix,
                shift,
                depth: 0,
                cached_diagonal: Vec::new(),
            },
        })
    }

    /// Return an intervention checkpoint with a changed shift and invalidated
    /// cache. A non-finite delta/result is rejected without mutating the source.
    pub fn shifted(&self, delta: f64) -> Result<JacobiCheckpoint, AdapterError> {
        let shift = self.checkpoint.shift + delta;
        if !delta.is_finite() || !shift.is_finite() {
            return Err(AdapterError::InvalidCheckpoint);
        }
        Ok(JacobiCheckpoint {
            shift,
            cached_diagonal: Vec::new(),
            ..self.checkpoint.clone()
        })
    }

    fn valid_checkpoint(&self, cp: &JacobiCheckpoint) -> bool {
        if cp.matrix != self.checkpoint.matrix || cp.depth > 64 || !cp.shift.is_finite() {
            return false;
        }
        if cp.cached_diagonal.is_empty() {
            return true;
        }
        GreenBands::compute(&cp.matrix, cp.shift).is_ok_and(|g| g.diagonal() == cp.cached_diagonal)
    }
}

impl ReplayAdapter for JacobiSweep {
    type Checkpoint = JacobiCheckpoint;
    type Observation = Vec<f64>;
    type Error = AdapterError;

    fn fork(&self, checkpoint: &JacobiCheckpoint) -> Result<Self, AdapterError> {
        if !self.valid_checkpoint(checkpoint) {
            return Err(AdapterError::InvalidCheckpoint);
        }
        Ok(Self {
            checkpoint: checkpoint.clone(),
        })
    }
    fn checkpoint(&self) -> Result<JacobiCheckpoint, AdapterError> {
        Ok(self.checkpoint.clone())
    }
    fn advance(&mut self, context: StepContext) -> Result<Vec<f64>, AdapterError> {
        if context.noise_stream != 0
            || self.checkpoint.depth >= 64
            || context.depth != self.checkpoint.depth + 1
        {
            return Err(AdapterError::InvalidContext);
        }
        let shift = self.checkpoint.shift + 0.25;
        if !shift.is_finite() {
            return Err(AdapterError::LibraryRejected);
        }
        let diagonal = GreenBands::compute(&self.checkpoint.matrix, shift)
            .map_err(|_| AdapterError::LibraryRejected)?
            .diagonal()
            .to_vec();
        // Commit only after the full observation succeeds.
        self.checkpoint.shift = shift;
        self.checkpoint.depth = context.depth;
        self.checkpoint.cached_diagonal.clone_from(&diagonal);
        Ok(diagonal)
    }
}

impl ReplayCodec for JacobiSweep {
    fn progress(&self) -> usize {
        self.checkpoint.depth
    }
    fn contract(&self) -> AdapterContract {
        AdapterContract {
            implementation: "tdi-jacobi-sweep/v1",
            advancement_unit: "resolvent shift increment of 0.25",
            observation_unit: "inverse-matrix diagonal; reciprocal coefficient units",
            precision: "binary64 sequential GreenBands",
            reproduction: Reproduction::NumericalWithDeclaredTolerance,
            rng: "none",
            max_checkpoint_bytes: 512,
            max_steps: 64,
            max_state_scalars: 64,
            access: "Development/Validation generic finite operators only",
            execution_limits: "1..16 rows; positive pivots required; cooperative cancellation/deadline between steps; no asymptotic/hardware claim",
        }
    }
    fn encode_checkpoint(&self) -> Result<Vec<u8>, AdapterError> {
        let cp = &self.checkpoint;
        let mut raw = b"TDIJCP1\0".to_vec();
        raw.extend([
            cp.matrix.len() as u8,
            cp.depth as u8,
            cp.cached_diagonal.len() as u8,
        ]);
        for value in std::iter::once(&cp.shift)
            .chain(cp.matrix.diagonal())
            .chain(cp.matrix.off_diagonal())
            .chain(&cp.cached_diagonal)
        {
            raw.extend(value.to_bits().to_le_bytes());
        }
        Ok(raw)
    }
    fn decode_checkpoint(&self, bytes: &[u8]) -> Result<JacobiCheckpoint, AdapterError> {
        if bytes.len() < 19 || bytes.len() > 512 || &bytes[..8] != b"TDIJCP1\0" {
            return Err(AdapterError::InvalidCheckpoint);
        }
        let (n, depth, cache_len) = (bytes[8] as usize, bytes[9] as usize, bytes[10] as usize);
        if !(1..=16).contains(&n)
            || depth > 64
            || ![0, n].contains(&cache_len)
            || bytes.len() != 11 + 8 * (2 * n + cache_len)
        {
            return Err(AdapterError::InvalidCheckpoint);
        }
        let values: Vec<f64> = bytes[11..]
            .chunks_exact(8)
            .map(|x| f64::from_bits(u64::from_le_bytes(x.try_into().unwrap())))
            .collect();
        if values.iter().any(|x| !x.is_finite()) {
            return Err(AdapterError::InvalidCheckpoint);
        }
        let matrix = JacobiMatrix::new(values[1..n + 1].to_vec(), values[n + 1..2 * n].to_vec())
            .map_err(|_| AdapterError::InvalidCheckpoint)?;
        let cp = JacobiCheckpoint {
            matrix,
            shift: values[0],
            depth,
            cached_diagonal: values[2 * n..].to_vec(),
        };
        if !self.valid_checkpoint(&cp) {
            return Err(AdapterError::InvalidCheckpoint);
        }
        Ok(cp)
    }
}
