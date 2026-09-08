//! Deterministic generic software fixture for the Linux supervisor.
//! Not a model runner, scientific population or confirmatory experiment.
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
fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 5, "expected --tdi-seed N --tdi-plan-id SHA256");
    assert_eq!(args[1], "--tdi-seed");
    assert_eq!(args[3], "--tdi-plan-id");
    let seed: u64 = args[2].parse().expect("u64 seed");
    let plan = &args[4];
    assert!(plan.len() == 64 && plan.bytes().all(|b| b.is_ascii_hexdigit()));
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
    let values: Vec<u64> = report.profile.points().iter().map(|p| *p.overlap()).collect();
    println!(
        "{{\"seed\":{seed},\"plan_id\":\"{plan}\",\"status\":\"Evaluated\",\"scores\":{values:?}}}"
    );
}
