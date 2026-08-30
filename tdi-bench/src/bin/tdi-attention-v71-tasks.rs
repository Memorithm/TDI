//! Deterministic task generators for the frozen TDI-7.0 H-AI-1 protocol.
//!
//! This binary is a bounded software-oracle fixture. It does not read or emit
//! final-holdout results and it has no model-fitting path.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TaskKind {
    AssociativeRecall,
    Copy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TaskExample {
    seed: u64,
    kind: TaskKind,
    input: Vec<u16>,
    target: Vec<u16>,
    retrieval_distance: usize,
    distractor_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn bounded(&mut self, upper_exclusive: u16) -> u16 {
        assert!(upper_exclusive > 0);
        (self.next_u64() % u64::from(upper_exclusive)) as u16
    }
}

fn unique_tokens(rng: &mut SplitMix64, count: usize, base: u16, width: u16) -> Vec<u16> {
    assert!(count <= usize::from(width));
    let mut out = Vec::with_capacity(count);
    while out.len() < count {
        let token = base + rng.bounded(width);
        if !out.contains(&token) {
            out.push(token);
        }
    }
    out
}

fn generate_associative_recall(seed: u64) -> TaskExample {
    const PAIRS: usize = 4;
    const QUERY_MARKER: u16 = 250;

    let mut rng = SplitMix64::new(seed ^ 0x5444_4937_4152_0001);
    let keys = unique_tokens(&mut rng, PAIRS, 1, 64);
    let values = unique_tokens(&mut rng, PAIRS, 128, 64);
    let query_pair = usize::from(rng.bounded(PAIRS as u16));

    let mut input = Vec::with_capacity(PAIRS * 2 + 2);
    for index in 0..PAIRS {
        input.push(keys[index]);
        input.push(values[index]);
    }
    input.push(QUERY_MARKER);
    input.push(keys[query_pair]);

    let query_position = input.len() - 1;
    let value_position = query_pair * 2 + 1;

    TaskExample {
        seed,
        kind: TaskKind::AssociativeRecall,
        input,
        target: vec![values[query_pair]],
        retrieval_distance: query_position - value_position,
        distractor_count: PAIRS - 1,
    }
}

fn generate_copy(seed: u64) -> TaskExample {
    const COPY_MARKER: u16 = 251;
    const SOURCE_LEN: usize = 4;

    let mut rng = SplitMix64::new(seed ^ 0x5444_4937_434F_0001);
    let source = unique_tokens(&mut rng, SOURCE_LEN, 16, 96);
    let distractor_count = usize::from(rng.bounded(4)) + 1;
    let distractors = unique_tokens(&mut rng, distractor_count, 160, 64);

    let mut input = Vec::with_capacity(SOURCE_LEN + distractor_count + 1);
    input.extend_from_slice(&source);
    input.extend_from_slice(&distractors);
    input.push(COPY_MARKER);

    TaskExample {
        seed,
        kind: TaskKind::Copy,
        input,
        target: source,
        retrieval_distance: distractor_count + 1,
        distractor_count,
    }
}

fn main() {
    let ar = generate_associative_recall(7_100_000_000);
    let copy = generate_copy(7_100_000_001);
    println!("TDI-7.1 task-generator preflight: PASS");
    println!(
        "associative-recall input_len={} target_len={}",
        ar.input.len(),
        ar.target.len()
    );
    println!(
        "copy input_len={} target_len={}",
        copy.input.len(),
        copy.target.len()
    );
    println!("TDI-7.2 final holdout: NOT ACCESSED");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn associative_recall_is_seed_deterministic() {
        let left = generate_associative_recall(1234);
        let right = generate_associative_recall(1234);
        assert_eq!(left, right);
    }

    #[test]
    fn associative_recall_changes_across_seeds() {
        assert_ne!(
            generate_associative_recall(1234),
            generate_associative_recall(1235)
        );
    }

    #[test]
    fn associative_target_is_bound_to_a_present_query_key() {
        let example = generate_associative_recall(42);
        assert_eq!(example.kind, TaskKind::AssociativeRecall);
        assert_eq!(example.target.len(), 1);
        assert_eq!(example.input[example.input.len() - 2], 250);

        let query_key = example.input[example.input.len() - 1];
        let mut recovered = None;
        for pair in example.input[..example.input.len() - 2].chunks_exact(2) {
            if pair[0] == query_key {
                recovered = Some(pair[1]);
            }
        }
        assert_eq!(recovered, Some(example.target[0]));
    }

    #[test]
    fn associative_keys_are_unique() {
        let example = generate_associative_recall(99);
        let prefix = &example.input[..example.input.len() - 2];
        let keys: Vec<_> = prefix.chunks_exact(2).map(|pair| pair[0]).collect();
        for (left_index, left) in keys.iter().enumerate() {
            for right in &keys[(left_index + 1)..] {
                assert_ne!(left, right);
            }
        }
    }

    #[test]
    fn copy_is_seed_deterministic() {
        let left = generate_copy(555);
        let right = generate_copy(555);
        assert_eq!(left, right);
    }

    #[test]
    fn copy_target_is_exact_source_prefix() {
        let example = generate_copy(777);
        assert_eq!(example.kind, TaskKind::Copy);
        assert_eq!(example.target.len(), 4);
        assert_eq!(&example.input[..4], example.target.as_slice());
        assert_eq!(example.input.last(), Some(&251));
    }

    #[test]
    fn task_local_controls_are_nonzero_and_consistent() {
        for seed in 0..64 {
            let ar = generate_associative_recall(seed);
            assert_eq!(ar.distractor_count, 3);
            assert!(ar.retrieval_distance > 0);

            let copy = generate_copy(seed);
            assert!((1..=4).contains(&copy.distractor_count));
            assert_eq!(copy.retrieval_distance, copy.distractor_count + 1);
        }
    }

    #[test]
    fn generators_do_not_embed_final_holdout_authorization() {
        let source = include_str!("tdi-attention-v71-tasks.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
