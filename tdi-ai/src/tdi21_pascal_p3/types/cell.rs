use super::{Schedule, Split};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub split: Split,
    pub n: u8,
    pub k: usize,
    pub density: usize,
    pub schedule: Schedule,
    pub query_load: usize,
    pub reuse: usize,
    pub checksums: [u64; 3],
    pub mismatches: [u64; 2],
    pub counts: [u64; 4],
    pub representation: [usize; 3],
    pub pairwise_token_comparisons: u64,
    pub work: [u64; 3],
    pub deltas: [i128; 2],
}
