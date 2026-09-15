from pathlib import Path

p = Path("tdi-ai/src/tdi22_torsor.rs")
s = p.read_text()
old = '''    /// First reduction-point invariant used by Stage 0: `||R||^2`.
    #[must_use]
    pub fn resultant_norm_squared(self) -> f64 {
        self.resultant.norm_squared()
    }

    /// Second reduction-point invariant used by Stage 0: `R . M(P)`.
    #[must_use]
    pub fn scalar_invariant(self) -> f64 {
        self.resultant.dot(self.moment)
    }
'''
new = '''    /// First reduction-point invariant used by Stage 0: `||R||^2`.
    ///
    /// Finite vector components can still overflow during the dot product, so a
    /// non-finite derived scalar is rejected rather than exposed as an invariant.
    pub fn resultant_norm_squared(self) -> Result<f64, TorsorError> {
        let value = self.resultant.norm_squared();
        if value.is_finite() {
            Ok(value)
        } else {
            Err(TorsorError::NonFiniteScalar {
                field: "resultant_norm_squared",
            })
        }
    }

    /// Second reduction-point invariant used by Stage 0: `R . M(P)`.
    ///
    /// As with the norm invariant, arithmetic overflow is a typed failure even
    /// when every input component is individually finite.
    pub fn scalar_invariant(self) -> Result<f64, TorsorError> {
        let value = self.resultant.dot(self.moment);
        if value.is_finite() {
            Ok(value)
        } else {
            Err(TorsorError::NonFiniteScalar {
                field: "scalar_invariant",
            })
        }
    }
'''
if old not in s:
    raise SystemExit("expected invariant API block not found")
s = s.replace(old, new, 1)
old_test = '''        close(
            torsor.resultant_norm_squared(),
            moved.resultant_norm_squared(),
        );
        close(torsor.scalar_invariant(), moved.scalar_invariant());
'''
new_test = '''        close(
            torsor.resultant_norm_squared().unwrap(),
            moved.resultant_norm_squared().unwrap(),
        );
        close(
            torsor.scalar_invariant().unwrap(),
            moved.scalar_invariant().unwrap(),
        );
'''
if old_test not in s:
    raise SystemExit("expected invariant comparison test not found")
s = s.replace(old_test, new_test, 1)
anchor = '''    #[test]
    fn direct_and_factorized_pairings_are_equivalent() {
'''
regression = '''    #[test]
    fn derived_invariants_reject_finite_inputs_that_overflow() {
        let huge = v(f64::MAX, 0.0, 0.0);
        let torsor = Torsor3::new(huge, huge, Vec3::zero()).unwrap();
        assert_eq!(
            torsor.resultant_norm_squared(),
            Err(TorsorError::NonFiniteScalar {
                field: "resultant_norm_squared",
            })
        );
        assert_eq!(
            torsor.scalar_invariant(),
            Err(TorsorError::NonFiniteScalar {
                field: "scalar_invariant",
            })
        );
    }

    #[test]
    fn direct_and_factorized_pairings_are_equivalent() {
'''
if anchor not in s:
    raise SystemExit("expected test insertion anchor not found")
s = s.replace(anchor, regression, 1)
p.write_text(s)
