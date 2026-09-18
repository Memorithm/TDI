//! TDI-24 Stage-0 mirror-coupled chiral algebra scaffold.
//!
//! This module defines the common chiral contract shared by the TDI-24
//! vector-vs-chiral campaign and the downstream TDI-25 torsor-vs-chiral
//! campaign.  It intentionally contains no learned model, no softmax kernel,
//! no confirmatory evaluator, and no quality/performance claim.
//!
//! The six-dimensional carrier is split into parity sectors
//! `H = H+ direct_sum H-`, each of dimension three.  The mirror involution is
//!
//! `M(x+, x-) = (x+, -x-)`,
//!
//! and the orthogonal complex structure is
//!
//! `J(x+, x-) = (x-, -x+)`.
//!
//! Therefore `M^2 = I`, `J^T = -J`, `J^2 = -I`, and `M J M = -J`.
//! The pseudoscalar bilinear `chi(q, k) = q^T J k` is parity odd:
//! `chi(Mq, Mk) = -chi(q, k)`.

use core::fmt;

/// Versioned Stage-0 chiral algebra contract.
pub const CHIRAL_CONTRACT: &str = "tdi24-mirror-coupled-chiral-v1";

/// Fixed Stage-0 carrier width. TDI-25 uses the same six-component budget as
/// the TDI-22 torsor representation.
pub const CHIRAL_WIDTH: usize = 6;

/// Versioned contract for the unaggregated `s/m/chi` channel surface.
pub const CHANNEL_DECOMPOSITION_CONTRACT: &str = "tdi24-channel-decomposition-v1";

/// Finite six-component parity carrier `(x+, x-)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chiral6 {
    data: [f64; CHIRAL_WIDTH],
}

impl Chiral6 {
    /// Construct a finite carrier from the parity-even and parity-odd triples.
    pub fn new(even: [f64; 3], odd: [f64; 3]) -> Result<Self, ChiralError> {
        let data = [even[0], even[1], even[2], odd[0], odd[1], odd[2]];
        Self::from_array(data)
    }

    /// Construct a finite carrier from an explicit six-component array.
    pub fn from_array(data: [f64; CHIRAL_WIDTH]) -> Result<Self, ChiralError> {
        if data.iter().all(|value| value.is_finite()) {
            Ok(Self { data })
        } else {
            Err(ChiralError::NonFiniteVector)
        }
    }

    /// Return the complete six-component representation.
    #[must_use]
    pub const fn as_array(self) -> [f64; CHIRAL_WIDTH] {
        self.data
    }

    /// Parity-even sector `x+`.
    #[must_use]
    pub const fn even(self) -> [f64; 3] {
        [self.data[0], self.data[1], self.data[2]]
    }

    /// Parity-odd sector `x-`.
    #[must_use]
    pub const fn odd(self) -> [f64; 3] {
        [self.data[3], self.data[4], self.data[5]]
    }

    /// Mirror involution `M(x+,x-)=(x+,-x-)`.
    #[must_use]
    pub fn mirror(self) -> Self {
        Self {
            data: [
                self.data[0],
                self.data[1],
                self.data[2],
                -self.data[3],
                -self.data[4],
                -self.data[5],
            ],
        }
    }

    /// Orthogonal complex structure `J(x+,x-)=(x-,-x+)`.
    #[must_use]
    pub fn complex_structure(self) -> Self {
        Self {
            data: [
                self.data[3],
                self.data[4],
                self.data[5],
                -self.data[0],
                -self.data[1],
                -self.data[2],
            ],
        }
    }

    /// Additive inverse.
    #[must_use]
    pub fn negate(self) -> Self {
        let mut data = self.data;
        for value in &mut data {
            *value = -*value;
        }
        Self { data }
    }

    /// Euclidean pairing with fail-closed product and accumulation checks.
    ///
    /// Every product and every partial sum must remain finite. This deliberately
    /// rejects a computation as soon as overflow occurs rather than allowing a
    /// later term to hide the invalid intermediate through cancellation.
    pub fn dot(self, rhs: Self) -> Result<f64, ChiralError> {
        let mut accumulator = 0.0;
        for (lhs, rhs) in self.data.iter().zip(rhs.data.iter()) {
            let product = finite_mul(*lhs, *rhs, "dot_product")?;
            accumulator = finite_add(accumulator, product, "dot_accumulator")?;
        }
        Ok(accumulator)
    }

    /// Mirror-even bilinear `q^T M k`.
    pub fn mirror_pairing(self, key: Self) -> Result<f64, ChiralError> {
        self.dot(key.mirror())
    }

    /// Parity-odd bilinear `chi(q,k)=q^T J k`.
    ///
    /// The six products are accumulated as three coordinate-local antisymmetric
    /// differences `q+_i k-_i - q-_i k+_i`.  This keeps the fail-closed success
    /// domain symmetric under argument exchange: swapping `(q, k)` negates each
    /// coordinate contribution before the same accumulation order is applied.
    pub fn chiral_pairing(self, key: Self) -> Result<f64, ChiralError> {
        let query_even = self.even();
        let query_odd = self.odd();
        let key_even = key.even();
        let key_odd = key.odd();
        let mut accumulator = 0.0;

        for index in 0..3 {
            let even_odd =
                finite_mul(query_even[index], key_odd[index], "chiral_even_odd_product")?;
            let odd_even =
                finite_mul(query_odd[index], key_even[index], "chiral_odd_even_product")?;
            let component = finite_add(even_odd, -odd_even, "chiral_component")?;
            accumulator = finite_add(accumulator, component, "chiral_accumulator")?;
        }

        Ok(accumulator)
    }
}

/// Coefficients for the Stage-0 scalar score family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralScoreWeights {
    /// Conventional vector similarity coefficient.
    pub alpha: f64,
    /// Mirror-even coupling coefficient.
    pub beta: f64,
    /// Parity-odd coupling coefficient.
    pub gamma: f64,
}

impl ChiralScoreWeights {
    /// Construct finite score coefficients.
    pub fn new(alpha: f64, beta: f64, gamma: f64) -> Result<Self, ChiralError> {
        if alpha.is_finite() && beta.is_finite() && gamma.is_finite() {
            Ok(Self { alpha, beta, gamma })
        } else {
            Err(ChiralError::NonFiniteWeights)
        }
    }
}

/// Three primitive observables retained separately for controlled ablations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralObservables {
    /// Conventional dot product `q^T k`.
    pub direct: f64,
    /// Mirror-even channel `q^T M k`.
    pub mirrored: f64,
    /// Parity-odd channel `q^T J k`.
    pub chiral: f64,
}

/// Stable mathematical identity of one primitive TDI-24 channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChiralChannelId {
    /// `s(q,k) = q^T k`.
    S,
    /// `m(q,k) = q^T M k`.
    M,
    /// `chi(q,k) = q^T J k`.
    Chi,
}

/// Transformation parity under simultaneous reflection `(q,k) -> (Mq,Mk)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReflectionParity {
    /// Channel value is invariant under simultaneous reflection.
    Even,
    /// Channel value changes sign under simultaneous reflection.
    Odd,
}

/// Provenance attached to an unaggregated primitive channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelProvenance {
    /// Stable mathematical channel identity.
    pub channel: ChiralChannelId,
    /// Declared simultaneous-reflection parity.
    pub reflection_parity: ReflectionParity,
    /// Algebra contract that defines `M`, `J` and the carrier.
    pub algebra_contract: &'static str,
    /// Contract that defines this tagged decomposition surface.
    pub decomposition_contract: &'static str,
}

/// One primitive value together with immutable semantic provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TaggedChannel {
    /// Computed scalar value.
    pub value: f64,
    /// Stable semantic provenance.
    pub provenance: ChannelProvenance,
}

/// Explicit unaggregated `s/m/chi` decomposition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChannelDecomposition {
    /// Conventional direct channel `s=q^T k`.
    pub s: TaggedChannel,
    /// Mirror-even channel `m=q^T M k`.
    pub m: TaggedChannel,
    /// Parity-odd channel `chi=q^T J k`.
    pub chi: TaggedChannel,
}

/// Evaluate the three primitive observables without collapsing them into one
/// scalar. Keeping them separate is required for later matched ablations.
pub fn observables(query: Chiral6, key: Chiral6) -> Result<ChiralObservables, ChiralError> {
    Ok(ChiralObservables {
        direct: query.dot(key)?,
        mirrored: query.mirror_pairing(key)?,
        chiral: query.chiral_pairing(key)?,
    })
}

/// Evaluate and tag the primitive `s/m/chi` channels.
///
/// This function adds provenance only: it performs no weighting, normalization,
/// masking, aggregation or winner selection.
pub fn decomposed_observables(
    query: Chiral6,
    key: Chiral6,
) -> Result<ChannelDecomposition, ChiralError> {
    let raw = observables(query, key)?;
    let tagged = |channel, reflection_parity, value| TaggedChannel {
        value,
        provenance: ChannelProvenance {
            channel,
            reflection_parity,
            algebra_contract: CHIRAL_CONTRACT,
            decomposition_contract: CHANNEL_DECOMPOSITION_CONTRACT,
        },
    };

    Ok(ChannelDecomposition {
        s: tagged(ChiralChannelId::S, ReflectionParity::Even, raw.direct),
        m: tagged(ChiralChannelId::M, ReflectionParity::Even, raw.mirrored),
        chi: tagged(ChiralChannelId::Chi, ReflectionParity::Odd, raw.chiral),
    })
}

/// Conventional vector control score using the identical six-component carrier.
pub fn vector_score(query: Chiral6, key: Chiral6) -> Result<f64, ChiralError> {
    query.dot(key)
}

/// Stage-0 chiral scalar score `alpha*s + beta*m + gamma*chi`.
///
/// No `sqrt(d)` normalization is baked into this algebraic contract; score
/// scaling is a later preregistered comparison choice shared by all arms.
pub fn chiral_score(
    query: Chiral6,
    key: Chiral6,
    weights: ChiralScoreWeights,
) -> Result<f64, ChiralError> {
    let channels = observables(query, key)?;
    let direct = finite_mul(weights.alpha, channels.direct, "weighted_direct")?;
    let mirrored = finite_mul(weights.beta, channels.mirrored, "weighted_mirrored")?;
    let chiral = finite_mul(weights.gamma, channels.chiral, "weighted_chiral")?;
    let even = finite_add(direct, mirrored, "chiral_even_sum")?;
    finite_add(even, chiral, "chiral_score")
}

/// Right/left enantiomorphic scores. Reflection swaps the two scores because
/// only the parity-odd channel changes sign.
pub fn enantiomorphic_scores(
    query: Chiral6,
    key: Chiral6,
    weights: ChiralScoreWeights,
) -> Result<(f64, f64), ChiralError> {
    let channels = observables(query, key)?;
    let direct = finite_mul(weights.alpha, channels.direct, "weighted_direct")?;
    let mirrored = finite_mul(weights.beta, channels.mirrored, "weighted_mirrored")?;
    let common = finite_add(direct, mirrored, "enantiomorphic_common")?;
    let odd = finite_mul(weights.gamma, channels.chiral, "weighted_chiral")?;
    Ok((
        finite_add(common, odd, "right_score")?,
        finite_add(common, -odd, "left_score")?,
    ))
}

fn finite_mul(lhs: f64, rhs: f64, field: &'static str) -> Result<f64, ChiralError> {
    finite_scalar(lhs * rhs, field)
}

fn finite_add(lhs: f64, rhs: f64, field: &'static str) -> Result<f64, ChiralError> {
    finite_scalar(lhs + rhs, field)
}

fn finite_scalar(value: f64, field: &'static str) -> Result<f64, ChiralError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ChiralError::NonFiniteScalar { field })
    }
}

/// Fail-closed Stage-0 input and arithmetic errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChiralError {
    /// A carrier contains NaN or infinity.
    NonFiniteVector,
    /// One or more score coefficients are non-finite.
    NonFiniteWeights,
    /// A derived scalar overflowed or became non-finite.
    NonFiniteScalar {
        /// Name of the rejected derived quantity.
        field: &'static str,
    },
}

impl fmt::Display for ChiralError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteVector => formatter.write_str("chiral carrier must be finite"),
            Self::NonFiniteWeights => formatter.write_str("chiral score weights must be finite"),
            Self::NonFiniteScalar { field } => write!(formatter, "{field} must be finite"),
        }
    }
}

impl std::error::Error for ChiralError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(even: [f64; 3], odd: [f64; 3]) -> Chiral6 {
        Chiral6::new(even, odd).unwrap()
    }

    fn close(lhs: f64, rhs: f64) {
        let scale = 1.0_f64.max(lhs.abs()).max(rhs.abs());
        assert!(
            (lhs - rhs).abs() <= 64.0 * f64::EPSILON * scale,
            "lhs={lhs:?}, rhs={rhs:?}"
        );
    }

    #[test]
    fn mirror_is_an_involution() {
        let x = c([1.0, -2.0, 3.0], [4.0, 5.0, -6.0]);
        assert_eq!(x.mirror().mirror(), x);
    }

    #[test]
    fn complex_structure_squares_to_minus_identity() {
        let x = c([1.0, -2.0, 3.0], [4.0, 5.0, -6.0]);
        assert_eq!(x.complex_structure().complex_structure(), x.negate());
    }

    #[test]
    fn mirror_conjugates_j_to_minus_j() {
        let x = c([1.0, -2.0, 3.0], [4.0, 5.0, -6.0]);
        let lhs = x.mirror().complex_structure().mirror();
        let rhs = x.complex_structure().negate();
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn direct_and_mirror_channels_are_even_under_common_reflection() {
        let q = c([1.0, 2.0, -3.0], [0.5, -1.5, 2.5]);
        let k = c([-4.0, 1.0, 2.0], [3.0, 0.25, -0.75]);
        let base = observables(q, k).unwrap();
        let reflected = observables(q.mirror(), k.mirror()).unwrap();
        close(base.direct, reflected.direct);
        close(base.mirrored, reflected.mirrored);
    }

    #[test]
    fn chiral_channel_is_odd_under_common_reflection() {
        let q = c([1.0, 2.0, -3.0], [0.5, -1.5, 2.5]);
        let k = c([-4.0, 1.0, 2.0], [3.0, 0.25, -0.75]);
        let chi = q.chiral_pairing(k).unwrap();
        let mirrored = q.mirror().chiral_pairing(k.mirror()).unwrap();
        close(mirrored, -chi);
    }

    #[test]
    fn chiral_pairing_is_antisymmetric_and_zero_on_self_pairing() {
        let q = c([1.0, 2.0, -3.0], [0.5, -1.5, 2.5]);
        let k = c([-4.0, 1.0, 2.0], [3.0, 0.25, -0.75]);
        close(q.chiral_pairing(k).unwrap(), -k.chiral_pairing(q).unwrap());
        close(q.chiral_pairing(q).unwrap(), 0.0);
    }

    #[test]
    fn chiral_pairing_overflow_domain_is_symmetric_under_argument_exchange() {
        let q = c([f64::MAX, f64::MAX, 0.0], [f64::MAX, 0.0, 0.0]);
        let k = c([1.0, 0.0, 0.0], [1.0, 1.0, 0.0]);

        let qk = q.chiral_pairing(k).unwrap();
        let kq = k.chiral_pairing(q).unwrap();
        assert_eq!(qk, f64::MAX);
        assert_eq!(kq, -f64::MAX);
        assert_eq!(qk, -kq);
    }

    #[test]
    fn decomposed_channels_preserve_unaggregated_observable_values() {
        let q = c([1.0, 2.0, -3.0], [0.5, -1.5, 2.5]);
        let k = c([-4.0, 1.0, 2.0], [3.0, 0.25, -0.75]);
        let raw = observables(q, k).unwrap();
        let channels = decomposed_observables(q, k).unwrap();
        assert_eq!(channels.s.value, raw.direct);
        assert_eq!(channels.m.value, raw.mirrored);
        assert_eq!(channels.chi.value, raw.chiral);
    }

    #[test]
    fn decomposed_channels_have_stable_ids_parity_and_provenance() {
        let q = c([1.0, 2.0, -3.0], [0.5, -1.5, 2.5]);
        let k = c([-4.0, 1.0, 2.0], [3.0, 0.25, -0.75]);
        let channels = decomposed_observables(q, k).unwrap();
        let expected = [
            (channels.s, ChiralChannelId::S, ReflectionParity::Even),
            (channels.m, ChiralChannelId::M, ReflectionParity::Even),
            (channels.chi, ChiralChannelId::Chi, ReflectionParity::Odd),
        ];
        for (channel, id, parity) in expected {
            assert_eq!(channel.provenance.channel, id);
            assert_eq!(channel.provenance.reflection_parity, parity);
            assert_eq!(channel.provenance.algebra_contract, CHIRAL_CONTRACT);
            assert_eq!(
                channel.provenance.decomposition_contract,
                CHANNEL_DECOMPOSITION_CONTRACT
            );
        }
    }

    #[test]
    fn decomposed_channel_parity_matches_actual_reflection_transform() {
        let q = c([1.0, 2.0, -3.0], [0.5, -1.5, 2.5]);
        let k = c([-4.0, 1.0, 2.0], [3.0, 0.25, -0.75]);
        let base = decomposed_observables(q, k).unwrap();
        let reflected = decomposed_observables(q.mirror(), k.mirror()).unwrap();
        close(reflected.s.value, base.s.value);
        close(reflected.m.value, base.m.value);
        close(reflected.chi.value, -base.chi.value);
    }

    #[test]
    fn reflection_swaps_enantiomorphic_scores() {
        let q = c([1.0, 2.0, -3.0], [0.5, -1.5, 2.5]);
        let k = c([-4.0, 1.0, 2.0], [3.0, 0.25, -0.75]);
        let weights = ChiralScoreWeights::new(0.7, -0.2, 1.3).unwrap();
        let (right, left) = enantiomorphic_scores(q, k, weights).unwrap();
        let (mirrored_right, mirrored_left) =
            enantiomorphic_scores(q.mirror(), k.mirror(), weights).unwrap();
        close(mirrored_right, left);
        close(mirrored_left, right);
    }

    #[test]
    fn non_finite_inputs_and_derived_overflow_fail_closed() {
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                Chiral6::new([invalid, 0.0, 0.0], [0.0; 3]),
                Err(ChiralError::NonFiniteVector)
            );
            assert_eq!(
                ChiralScoreWeights::new(1.0, invalid, 0.0),
                Err(ChiralError::NonFiniteWeights)
            );
        }

        let huge = c([f64::MAX, 0.0, 0.0], [0.0; 3]);
        assert_eq!(
            huge.dot(huge),
            Err(ChiralError::NonFiniteScalar {
                field: "dot_product"
            })
        );

        let accumulation = c([f64::MAX, f64::MAX, 0.0], [0.0; 3]);
        let ones = c([1.0, 1.0, 0.0], [0.0; 3]);
        assert_eq!(
            accumulation.dot(ones),
            Err(ChiralError::NonFiniteScalar {
                field: "dot_accumulator"
            })
        );

        let finite_channels = c([2.0, 0.0, 0.0], [0.0; 3]);
        let huge_weight = ChiralScoreWeights::new(f64::MAX, 0.0, 0.0).unwrap();
        assert_eq!(
            chiral_score(finite_channels, finite_channels, huge_weight),
            Err(ChiralError::NonFiniteScalar {
                field: "weighted_direct"
            })
        );
    }

    #[test]
    fn deterministic_property_fixture_preserves_stage0_algebra() {
        let fixtures = [
            c([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            c([1.0, -2.0, 3.0], [-4.0, 5.0, -6.0]),
            c([-0.5, 0.25, 2.0], [1.5, -3.0, 0.75]),
            c([8.0, -1.0, 0.125], [-0.25, 4.0, -2.0]),
            c([-7.0, 3.5, -1.75], [0.5, -0.125, 2.25]),
        ];

        for q in fixtures {
            assert_eq!(q.mirror().mirror(), q);
            assert_eq!(q.complex_structure().complex_structure(), q.negate());
            assert_eq!(
                q.mirror().complex_structure().mirror(),
                q.complex_structure().negate()
            );
            close(q.chiral_pairing(q).unwrap(), 0.0);

            for k in fixtures {
                let channels = observables(q, k).unwrap();
                let reflected = observables(q.mirror(), k.mirror()).unwrap();
                close(reflected.direct, channels.direct);
                close(reflected.mirrored, channels.mirrored);
                close(reflected.chiral, -channels.chiral);
                close(q.chiral_pairing(k).unwrap(), -k.chiral_pairing(q).unwrap());
            }
        }
    }
}
