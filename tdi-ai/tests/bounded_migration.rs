use std::cell::Cell;
use std::rc::Rc;
use tdi_ai::bounded_recovery::{PreparationError, analyze_bounded};
use tdi_ai::experiment::{
    Collection, ConformanceError, ReplayAdapter, RunLimits, Stage, StepContext,
    check_replay_conformance,
};
use tdi_ai::{
    FutureObservable, FutureOverlap, Intervention, ReferenceDynamics, analyze_intervention_recovery,
};

#[derive(Clone)]
struct Dynamics;
impl ReferenceDynamics for Dynamics {
    type State = i32;
    type Error = ();
    fn advance(&self, state: &i32) -> Result<i32, ()> {
        Ok(state + 1)
    }
}
#[derive(Clone)]
struct Observable;
impl FutureObservable<i32> for Observable {
    type Output = i32;
    type Error = ();
    fn observe(&self, state: &i32, _: usize) -> Result<i32, ()> {
        Ok(*state)
    }
}
struct Shift(Cell<usize>);
impl Intervention<i32> for Shift {
    type Error = ();
    fn apply(&self, state: &i32) -> Result<i32, ()> {
        self.0.set(self.0.get() + 1);
        Ok(state + 3)
    }
}
struct Metric;
impl FutureOverlap<i32> for Metric {
    type Score = i32;
    type Error = ();
    fn overlap(&self, a: &i32, b: &i32) -> Result<i32, ()> {
        Ok(b - a)
    }
}

#[test]
fn bounded_migration_matches_legacy_and_applies_intervention_once() {
    for horizon in [0, 1, 5, 30] {
        let shift = Shift(Cell::new(0));
        let legacy = analyze_intervention_recovery(
            &Dynamics, &shift, &Observable, &Metric, &0, horizon,
        )
        .unwrap();
        let report = analyze_bounded(
            Dynamics,
            &shift,
            Observable,
            &Metric,
            &0,
            RunLimits::new(horizon, 30, 30, Collection::All).unwrap(),
            |_| false,
            |_, _| Ok::<(), ()>(()),
        )
        .unwrap();
        assert_eq!(legacy, report.profile);
        assert!(report.failure.is_none());
        assert_eq!(shift.0.get(), 2);
    }
}

#[test]
fn bounded_migration_retains_prefix_and_preparation_error() {
    let shift = Shift(Cell::new(0));
    let report = analyze_bounded(
        Dynamics,
        &shift,
        Observable,
        &Metric,
        &0,
        RunLimits::new(5, 5, 5, Collection::All).unwrap(),
        |depth| depth == 3,
        |_, _| Ok::<(), ()>(()),
    )
    .unwrap();
    assert_eq!(report.completed_depth, 2);
    assert_eq!(report.failure.unwrap().stage, Stage::Cancelled);
    struct Broken;
    impl Intervention<i32> for Broken {
        type Error = ();
        fn apply(&self, _: &i32) -> Result<i32, ()> {
            Err(())
        }
    }
    assert!(matches!(
        analyze_bounded(
            Dynamics,
            &Broken,
            Observable,
            &Metric,
            &0,
            RunLimits::new(0, 0, 0, Collection::None).unwrap(),
            |_| false,
            |_, _| Ok::<(), ()>(()),
        ),
        Err(PreparationError::Intervention(()))
    ));
}

struct Stateful {
    state: u64,
    failure: Option<usize>,
}
impl ReplayAdapter for Stateful {
    type Checkpoint = u64;
    type Observation = u64;
    type Error = &'static str;
    fn fork(&self, state: &u64) -> Result<Self, Self::Error> {
        if self.failure == Some(0) {
            return Err("fork");
        }
        Ok(Self { state: *state, failure: self.failure })
    }
    fn checkpoint(&self) -> Result<u64, Self::Error> {
        if self.failure == Some(usize::MAX) {
            return Err("checkpoint");
        }
        Ok(self.state)
    }
    fn advance(&mut self, context: StepContext) -> Result<u64, Self::Error> {
        if self.failure == Some(context.depth) {
            return Err("advance");
        }
        self.state = self.state.wrapping_mul(6364136223846793005)
            .wrapping_add(context.noise_stream + 1);
        Ok(self.state)
    }
}

#[test]
fn rng_checkpoint_replay_and_failure_injection() {
    let contexts: Vec<_> = (1..=8).map(|depth| StepContext { depth, noise_stream: depth as u64 % 2 }).collect();
    for seed in [0, 1, u64::MAX] {
        assert_eq!(check_replay_conformance(&Stateful { state: seed, failure: None }, &contexts), Ok(()));
    }
    for failure in [0, 1, 2, 3, 4, 5, 6, 7, 8, usize::MAX] {
        assert!(matches!(check_replay_conformance(&Stateful { state: 0, failure: Some(failure) }, &contexts), Err(ConformanceError::Adapter(_))));
    }
}

#[test]
fn equal_observations_do_not_hide_shared_cache_mutation() {
    struct Shared(Rc<Cell<u64>>);
    impl ReplayAdapter for Shared {
        type Checkpoint = u64;
        type Observation = ();
        type Error = ();
        fn fork(&self, _: &u64) -> Result<Self, ()> {
            Ok(Self(self.0.clone()))
        }
        fn checkpoint(&self) -> Result<u64, ()> {
            Ok(self.0.get())
        }
        fn advance(&mut self, _: StepContext) -> Result<(), ()> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }
    }
    assert_eq!(
        check_replay_conformance(&Shared(Rc::new(Cell::new(0))), &[StepContext { depth: 1, noise_stream: 0 }]),
        Err(ConformanceError::SourceMutated)
    );
}
