//! Deterministic worker-response/v2 software fixture for TDI engine qualification.
//! Not a model runner, scientific population or confirmatory experiment.
use std::collections::BTreeMap;
use tdi_ai::bounded_recovery::analyze_bounded;
use tdi_ai::experiment::{Collection, RunLimits};
use tdi_ai::{FutureObservable, FutureOverlap, Intervention, ReferenceDynamics};

#[derive(Clone)]
struct Increment;
impl ReferenceDynamics for Increment {
    type State = u64;
    type Error = ();
    fn advance(&self, state: &u64) -> Result<u64, ()> {
        state.checked_add(1).ok_or(())
    }
}

#[derive(Clone)]
struct Identity;
impl FutureObservable<u64> for Identity {
    type Output = u64;
    type Error = ();
    fn observe(&self, state: &u64, _: usize) -> Result<u64, ()> {
        Ok(*state)
    }
}

struct Shift;
impl Intervention<u64> for Shift {
    type Error = ();
    fn apply(&self, state: &u64) -> Result<u64, ()> {
        state.checked_add(2).ok_or(())
    }
}

struct Distance;
impl FutureOverlap<u64> for Distance {
    type Score = u64;
    type Error = ();
    fn overlap(&self, a: &u64, b: &u64) -> Result<u64, ()> {
        Ok(a.abs_diff(*b))
    }
}

fn sha256_identity(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn main() {
    let mut raw = std::env::args().skip(1);
    let mut args = BTreeMap::new();
    while let Some(key) = raw.next() {
        let value = raw.next().expect("every worker flag requires a value");
        assert!(args.insert(key, value).is_none(), "duplicate worker flag");
    }
    let seed: u64 = args["--tdi-seed"].parse().expect("u64 seed");
    let plan = &args["--tdi-plan-id"];
    let experiment = &args["--tdi-experiment-id"];
    let trial = &args["--tdi-trial-id"];
    let attempt = &args["--tdi-attempt-id"];
    let domain = &args["--tdi-domain"];
    let backend = &args["--tdi-backend-identity"];
    assert_eq!(args["--tdi-worker-protocol"], "2");
    for value in [plan, experiment, trial, attempt] {
        assert!(sha256_identity(value));
    }
    assert!(matches!(domain.as_str(), "Development" | "Validation"));
    assert_eq!(backend, "linux-cgroup-v2");

    let report = analyze_bounded(
        Increment,
        &Shift,
        Identity,
        &Distance,
        &(seed % 100),
        RunLimits::new(4, 4, 4, Collection::All).unwrap(),
        |_| false,
        |_, _| Ok::<(), ()>(()),
    )
    .unwrap();
    assert!(report.failure.is_none());
    let values: Vec<u64> = report
        .profile
        .points()
        .iter()
        .map(|point| *point.overlap())
        .collect();
    let completed_observations = values.len();
    println!(
        "{{\"schema\":2,\"execution_status\":\"completed\",\"scientific_disposition\":\"evaluated\",\"experiment_id\":\"{experiment}\",\"plan_id\":\"{plan}\",\"trial_id\":\"{trial}\",\"attempt_id\":\"{attempt}\",\"backend_identity\":\"{backend}\",\"domain\":\"{domain}\",\"seed_decimal\":\"{seed}\",\"progress\":{{\"completed_steps\":4,\"completed_observations\":{completed_observations},\"costs\":{{\"fixture_steps\":4}}}},\"artifacts\":[],\"result\":{{\"scores\":{values:?}}},\"error\":null}}"
    );
}
