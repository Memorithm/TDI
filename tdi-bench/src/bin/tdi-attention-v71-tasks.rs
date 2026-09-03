//! Deterministic task-generator oracle for the frozen TDI-7.0 H-AI-1 protocol.
//!
//! The mechanics now live in `tdi_bench::attention_v7` so TDI-7.3 can reuse
//! them without copying scientific code. This binary remains a bounded oracle
//! and has no final-holdout execution path.

use tdi_bench::attention_v7::{TaskKind, generate_associative_recall, generate_copy};

fn main() {
    let ar = generate_associative_recall(7_100_000_000);
    let copy = generate_copy(7_100_000_001);
    println!("TDI-7.1 task-generator preflight: PASS");
    println!(
        "associative-recall input_len={} target_len={}",
        ar.input().len(),
        ar.target().len()
    );
    println!(
        "copy input_len={} target_len={}",
        copy.input().len(),
        copy.target().len()
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
        assert_eq!(example.kind(), TaskKind::AssociativeRecall);
        assert_eq!(example.target().len(), 1);
        assert_eq!(example.input()[example.input().len() - 2], 250);

        let query_key = example.input()[example.input().len() - 1];
        let mut recovered = None;
        for pair in example.input()[..example.input().len() - 2].chunks_exact(2) {
            if pair[0] == query_key {
                recovered = Some(pair[1]);
            }
        }
        assert_eq!(recovered, Some(example.target()[0]));
    }

    #[test]
    fn associative_keys_are_unique() {
        let example = generate_associative_recall(99);
        let prefix = &example.input()[..example.input().len() - 2];
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
        assert_eq!(example.kind(), TaskKind::Copy);
        assert_eq!(example.target().len(), 4);
        assert_eq!(&example.input()[..4], example.target());
        assert_eq!(example.input().last(), Some(&251));
    }

    #[test]
    fn task_local_controls_are_nonzero_and_consistent() {
        for seed in 0..64 {
            let ar = generate_associative_recall(seed);
            assert_eq!(ar.distractor_count(), 3);
            assert!(ar.retrieval_distance() > 0);

            let copy = generate_copy(seed);
            assert!((1..=4).contains(&copy.distractor_count()));
            assert_eq!(copy.retrieval_distance(), copy.distractor_count() + 1);
        }
    }

    #[test]
    fn generator_binary_does_not_embed_final_holdout_authorization() {
        let source = include_str!("tdi-attention-v71-tasks.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
