#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Split {
    Development,
    Validation,
}

impl Split {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Validation => "validation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Schedule {
    Affine,
    LowWeightFirst,
    HighWeightFirst,
}

impl Schedule {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Affine => "affine",
            Self::LowWeightFirst => "low_weight_first",
            Self::HighWeightFirst => "high_weight_first",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedWidth,
    InvalidDensity,
    InvalidQueryLoad,
    SplitWidthMismatch,
    DuplicateGeneratedMask,
    CounterOverflow,
}
