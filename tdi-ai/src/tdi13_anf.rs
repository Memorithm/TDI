//! Algebraic-normal-form (Zhegalkin) bootstrap primitives for TDI-13.
//!
//! This module is intentionally small and auditable. Terms are evaluated over
//! GF(2); no floating-point score is produced.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnfTerm {
    /// Bit mask identifying variables multiplied in this monomial.
    pub variables: u64,
}

impl AnfTerm {
    pub const fn eval(self, assignment: u64) -> bool {
        (assignment & self.variables) == self.variables
    }
}

pub fn eval_anf(constant: bool, terms: &[AnfTerm], assignment: u64) -> bool {
    let mut value = constant;
    for term in terms {
        value ^= term.eval(assignment);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anf_uses_xor_sum_and_boolean_products() {
        // f(x0,x1) = 1 xor x0 xor x0*x1
        let terms = [
            AnfTerm { variables: 0b01 },
            AnfTerm { variables: 0b11 },
        ];
        assert!(eval_anf(true, &terms, 0b00));
        assert!(!eval_anf(true, &terms, 0b01));
        assert!(eval_anf(true, &terms, 0b11));
    }
}
