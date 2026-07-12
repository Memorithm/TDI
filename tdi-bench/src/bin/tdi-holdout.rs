use std::collections::BTreeMap;

use tdi_core::{
    Action, State, TableSystem, TdiSignature, analyze_recovery, explore,
    uniform_future_block_entropy_bits,
};

const WIDTH: u8 = 3;
const STATE_COUNT: usize = 1 << WIDTH;

const TRAIN_SYSTEMS: u64 = 12_000;
const TEST_SYSTEMS: u64 = 4_000;
const TEST_SEED_OFFSET: u64 = 1_000_000;

const ENTROPY_HORIZON: usize = 8;
const TDI_HORIZON: usize = 4;
const RECOVERY_LIMIT: usize = 32;

#[derive(Clone, Debug)]
struct Record {
    entropy_key: u64,
    return_profile: Vec<(u128, u128)>,
    recovered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CombinedKey {
    entropy_key: u64,
    return_profile: Vec<(u128, u128)>,
}

#[derive(Clone, Debug)]
struct BucketModel<K> {
    buckets: BTreeMap<K, (usize, usize)>,
    global_probability: f64,
}

#[derive(Clone, Copy, Debug)]
struct Metrics {
    accuracy: f64,
    balanced_accuracy: f64,
    brier: f64,
    average_precision: f64,
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);

    let mut mixed = value;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);

    mixed ^ (mixed >> 31)
}

fn generate_transitions(seed: u64) -> [u8; STATE_COUNT] {
    let mut transitions = [0_u8; STATE_COUNT];
    let mut generator = seed;

    for target in &mut transitions {
        generator = splitmix64(generator);
        *target = (generator % STATE_COUNT as u64) as u8;
    }

    transitions
}

fn build_system(transitions: &[u8; STATE_COUNT]) -> Result<TableSystem, String> {
    let mut system =
        TableSystem::new(WIDTH).map_err(|error| format!("cannot create system: {error:?}"))?;

    for (source, &target) in transitions.iter().enumerate() {
        let source_state = State::new(source as u64, WIDTH).map_err(|error| error.to_string())?;

        let target_state =
            State::new(u64::from(target), WIDTH).map_err(|error| error.to_string())?;

        system
            .insert(source_state, Action::Noop, vec![target_state])
            .map_err(|error| format!("cannot insert transition {source}->{target}: {error:?}"))?;
    }

    Ok(system)
}

fn analyze_seed(seed: u64) -> Result<Record, String> {
    let transitions = generate_transitions(seed);
    let system = build_system(&transitions)?;

    let initial = State::new(0, WIDTH).map_err(|error| error.to_string())?;

    let entropy = uniform_future_block_entropy_bits(&system, Action::Noop, ENTROPY_HORIZON)
        .map_err(|error| format!("entropy failed for seed {seed}: {error:?}"))?;

    let recovery = analyze_recovery(
        &system,
        initial,
        Action::Flip { node: WIDTH - 1 },
        RECOVERY_LIMIT,
    )
    .map_err(|error| format!("recovery failed for seed {seed}: {error:?}"))?;

    let actions = [Action::Noop; TDI_HORIZON];

    let report = explore(&system, recovery.perturbed_state(), &actions)
        .map_err(|error| format!("exploration failed for seed {seed}: {error:?}"))?;

    let signature = TdiSignature::from_report(&report)
        .map_err(|error| format!("signature failed for seed {seed}: {error:?}"))?;

    let return_profile = signature
        .return_profile()
        .iter()
        .map(|ratio| (ratio.numerator(), ratio.denominator()))
        .collect();

    Ok(Record {
        entropy_key: entropy.to_bits(),
        return_profile,
        recovered: recovery.recovered(),
    })
}

fn generate_records(start_seed: u64, count: u64) -> Result<Vec<Record>, String> {
    (start_seed..start_seed + count).map(analyze_seed).collect()
}

fn fit_model<K, F>(records: &[Record], key_fn: F) -> BucketModel<K>
where
    K: Ord + Clone,
    F: Fn(&Record) -> K,
{
    let mut buckets = BTreeMap::<K, (usize, usize)>::new();

    let positives = records.iter().filter(|record| record.recovered).count();

    for record in records {
        let bucket = buckets.entry(key_fn(record)).or_default();
        bucket.0 += 1;

        if record.recovered {
            bucket.1 += 1;
        }
    }

    BucketModel {
        buckets,
        global_probability: positives as f64 / records.len() as f64,
    }
}

fn predict<K>(model: &BucketModel<K>, key: &K) -> f64
where
    K: Ord,
{
    model
        .buckets
        .get(key)
        .map(|(total, positives)| *positives as f64 / *total as f64)
        .unwrap_or(model.global_probability)
}

fn calculate_metrics(records: &[Record], probabilities: &[f64]) -> Metrics {
    assert_eq!(records.len(), probabilities.len());

    let mut correct = 0_usize;
    let mut positives = 0_usize;
    let mut negatives = 0_usize;
    let mut true_positive = 0_usize;
    let mut true_negative = 0_usize;
    let mut brier_sum = 0.0_f64;

    let mut score_groups = BTreeMap::<u64, (usize, usize)>::new();

    for (record, &probability) in records.iter().zip(probabilities) {
        let predicted = probability >= 0.5;

        if predicted == record.recovered {
            correct += 1;
        }

        if record.recovered {
            positives += 1;

            if predicted {
                true_positive += 1;
            }
        } else {
            negatives += 1;

            if !predicted {
                true_negative += 1;
            }
        }

        let target = if record.recovered { 1.0 } else { 0.0 };
        brier_sum += (target - probability).powi(2);

        let group = score_groups.entry(probability.to_bits()).or_default();

        group.0 += 1;

        if record.recovered {
            group.1 += 1;
        }
    }

    let accuracy = correct as f64 / records.len() as f64;

    let sensitivity = true_positive as f64 / positives as f64;

    let specificity = true_negative as f64 / negatives as f64;

    let balanced_accuracy = (sensitivity + specificity) / 2.0;

    let brier = brier_sum / records.len() as f64;

    let mut ordered_groups: Vec<(f64, usize, usize)> = score_groups
        .into_iter()
        .map(|(bits, (total, positive))| (f64::from_bits(bits), total, positive))
        .collect();

    ordered_groups.sort_by(|left, right| right.0.total_cmp(&left.0));

    let mut cumulative_true_positive = 0_usize;
    let mut cumulative_false_positive = 0_usize;
    let mut previous_recall = 0.0_f64;
    let mut average_precision = 0.0_f64;

    for (_, total, positive) in ordered_groups {
        cumulative_true_positive += positive;
        cumulative_false_positive += total - positive;

        let recall = cumulative_true_positive as f64 / positives as f64;

        let precision = cumulative_true_positive as f64
            / (cumulative_true_positive + cumulative_false_positive) as f64;

        average_precision += (recall - previous_recall) * precision;

        previous_recall = recall;
    }

    Metrics {
        accuracy,
        balanced_accuracy,
        brier,
        average_precision,
    }
}

fn print_metrics(label: &str, metrics: Metrics) {
    println!("{label}");
    println!("  accuracy          : {:.6}", metrics.accuracy);
    println!("  balanced accuracy : {:.6}", metrics.balanced_accuracy);
    println!("  Brier score       : {:.6}", metrics.brier);
    println!("  average precision : {:.6}", metrics.average_precision);
}

fn main() -> Result<(), String> {
    println!("Generating training systems...");
    let training = generate_records(0, TRAIN_SYSTEMS)?;

    println!("Generating untouched holdout systems...");
    let test = generate_records(TEST_SEED_OFFSET, TEST_SYSTEMS)?;

    let entropy_model = fit_model(&training, |record| record.entropy_key);

    let tdi_model = fit_model(&training, |record| record.return_profile.clone());

    let combined_model = fit_model(&training, |record| CombinedKey {
        entropy_key: record.entropy_key,
        return_profile: record.return_profile.clone(),
    });

    let entropy_probabilities: Vec<f64> = test
        .iter()
        .map(|record| predict(&entropy_model, &record.entropy_key))
        .collect();

    let tdi_probabilities: Vec<f64> = test
        .iter()
        .map(|record| predict(&tdi_model, &record.return_profile))
        .collect();

    let combined_probabilities: Vec<f64> = test
        .iter()
        .map(|record| {
            predict(
                &combined_model,
                &CombinedKey {
                    entropy_key: record.entropy_key,
                    return_profile: record.return_profile.clone(),
                },
            )
        })
        .collect();

    let entropy = calculate_metrics(&test, &entropy_probabilities);

    let tdi = calculate_metrics(&test, &tdi_probabilities);

    let combined = calculate_metrics(&test, &combined_probabilities);

    let training_positive = training.iter().filter(|record| record.recovered).count();

    let test_positive = test.iter().filter(|record| record.recovered).count();

    println!();
    println!("TDI-1 untouched holdout evaluation");
    println!("width             : {WIDTH}");
    println!("training systems  : {}", training.len());
    println!("holdout systems   : {}", test.len());
    println!("training recovered: {training_positive}");
    println!("holdout recovered : {test_positive}");
    println!();

    print_metrics("ENTROPY ONLY", entropy);
    println!();

    print_metrics("TDI RETURN PROFILE", tdi);
    println!();

    print_metrics("ENTROPY + TDI", combined);
    println!();

    println!(
        "TDI AUPRC gain over entropy      : {:.6}",
        tdi.average_precision - entropy.average_precision
    );

    println!(
        "combined AUPRC gain over entropy : {:.6}",
        combined.average_precision - entropy.average_precision
    );

    println!(
        "TDI Brier improvement            : {:.6}",
        entropy.brier - tdi.brier
    );

    println!(
        "combined Brier improvement       : {:.6}",
        entropy.brier - combined.brier
    );

    Ok(())
}
