use std::{cell::Cell, rc::Rc};
use tdi_ai::{
    MemoryAccounting, RecoveryPoint, RecoveryProfile, ReferenceArm, ReferenceSnapshot, StorageBits,
    experiment::*, validated_profile::*,
};

#[derive(Clone)]
struct Counter {
    value: i32,
    fail_at: Option<usize>,
}
impl ReplayAdapter for Counter {
    type Checkpoint = i32;
    type Observation = i32;
    type Error = &'static str;
    fn fork(&self, state: &i32) -> Result<Self, Self::Error> {
        Ok(Self {
            value: *state,
            fail_at: self.fail_at,
        })
    }
    fn checkpoint(&self) -> Result<i32, Self::Error> {
        Ok(self.value)
    }
    fn advance(&mut self, context: StepContext) -> Result<i32, Self::Error> {
        if self.fail_at == Some(context.depth) {
            return Err("backend");
        }
        self.value += 1 + context.noise_stream as i32;
        Ok(self.value)
    }
}
fn limits(collection: Collection) -> RunLimits {
    RunLimits::new(6, 6, 6, collection).unwrap()
}
fn metric(a: &i32, b: &i32) -> Result<i32, &'static str> {
    Ok(b - a)
}
fn sink(_: usize, _: &i32) -> Result<(), &'static str> {
    Ok(())
}

#[test]
fn bounded_streaming_and_noise_coupling() {
    let factory = Counter {
        value: 42,
        fail_at: None,
    };
    let mut streamed = Vec::new();
    let report = run_paired(
        &factory,
        &0,
        &10,
        limits(Collection::Every(2)),
        NoiseCoupling::Paired,
        metric,
        |_| false,
        |d, s| {
            streamed.push((d, *s));
            sink(d, s)
        },
    )
    .unwrap();
    assert_eq!(report.completed_depth, 6);
    assert!(report.failure.is_none());
    assert_eq!(streamed, (1..=6).map(|d| (d, 10)).collect::<Vec<_>>());
    assert_eq!(
        report
            .profile
            .points()
            .iter()
            .map(|p| p.depth())
            .collect::<Vec<_>>(),
        vec![2, 4, 6]
    );
    assert_eq!(factory.value, 42);
    let independent = run_paired(
        &factory,
        &0,
        &0,
        limits(Collection::All),
        NoiseCoupling::Independent,
        metric,
        |_| false,
        sink,
    )
    .unwrap();
    assert_eq!(
        independent
            .profile
            .points()
            .iter()
            .map(|p| *p.overlap())
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6]
    );
}
#[test]
fn failures_preserve_only_accepted_prefix() {
    let factory = Counter {
        value: 0,
        fail_at: Some(3),
    };
    let report = run_paired(
        &factory,
        &0,
        &1,
        limits(Collection::All),
        NoiseCoupling::Deterministic,
        metric,
        |_| false,
        sink,
    )
    .unwrap();
    assert_eq!(report.completed_depth, 2);
    assert_eq!(report.profile.horizon(), 2);
    let failure = report.failure.unwrap();
    assert_eq!(
        (failure.depth, failure.branch, failure.stage),
        (3, Some(Branch::Reference), Stage::Advance)
    );
    let factory = Counter {
        value: 0,
        fail_at: None,
    };
    let cancelled = run_paired(
        &factory,
        &0,
        &1,
        limits(Collection::None),
        NoiseCoupling::Paired,
        metric,
        |d| d == 2,
        sink,
    )
    .unwrap();
    assert_eq!(cancelled.completed_depth, 1);
    assert!(cancelled.profile.is_empty());
    let failed = run_paired(
        &factory,
        &0,
        &1,
        limits(Collection::All),
        NoiseCoupling::Paired,
        metric,
        |_| false,
        |d, s| if d == 3 { Err("disk") } else { sink(d, s) },
    )
    .unwrap();
    assert_eq!(failed.completed_depth, 2);
    assert_eq!(failed.profile.horizon(), 2);
    assert_eq!(failed.failure.unwrap().stage, Stage::Sink);
    let failed = run_paired(
        &factory,
        &0,
        &1,
        limits(Collection::All),
        NoiseCoupling::Paired,
        |_, _| Err::<i32, _>("metric"),
        |_| false,
        sink,
    )
    .unwrap();
    assert_eq!(failed.completed_depth, 0);
    assert_eq!(failed.failure.unwrap().stage, Stage::Metric);
}
#[test]
fn invalid_limits_fail_before_execution() {
    assert_eq!(
        RunLimits::new(2, 1, 2, Collection::All),
        Err(ConfigurationError::StepLimit)
    );
    assert_eq!(
        RunLimits::new(2, 2, 2, Collection::Every(0)),
        Err(ConfigurationError::ZeroStride)
    );
    assert_eq!(
        RunLimits::new(2, 2, 1, Collection::All),
        Err(ConfigurationError::PointLimit)
    );
    let factory = Counter {
        value: 0,
        fail_at: None,
    };
    let huge = RunLimits::new(usize::MAX, usize::MAX, usize::MAX, Collection::All).unwrap();
    assert_eq!(
        run_paired(
            &factory,
            &0,
            &0,
            huge,
            NoiseCoupling::Paired,
            metric,
            |_| false,
            sink
        ),
        Err(ConfigurationError::Capacity)
    );
    let cancelled = run_paired(
        &factory,
        &0,
        &0,
        limits(Collection::All),
        NoiseCoupling::Paired,
        metric,
        |_| true,
        sink,
    )
    .unwrap();
    assert_eq!(cancelled.failure.unwrap().depth, 0);
}
#[test]
fn qualification_detects_shared_mutable_backend() {
    struct Shared(Rc<Cell<i32>>);
    impl ReplayAdapter for Shared {
        type Checkpoint = i32;
        type Observation = i32;
        type Error = ();
        fn fork(&self, _: &i32) -> Result<Self, ()> {
            Ok(Self(self.0.clone()))
        }
        fn checkpoint(&self) -> Result<i32, ()> {
            Ok(self.0.get())
        }
        fn advance(&mut self, _: StepContext) -> Result<i32, ()> {
            self.0.set(self.0.get() + 1);
            Ok(self.0.get())
        }
    }
    let contexts = (1..=4)
        .map(|depth| StepContext {
            depth,
            noise_stream: 0,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        check_replay_conformance(
            &Counter {
                value: 0,
                fail_at: None
            },
            &contexts
        ),
        Ok(())
    );
    assert_eq!(
        check_replay_conformance(&Shared(Rc::new(Cell::new(0))), &contexts),
        Err(ConformanceError::SourceMutated)
    );
}
#[test]
fn profiles_validate_depths_and_declared_score_domain() {
    for depths in [vec![0, 1], vec![1, 1], vec![2, 1]] {
        assert!(
            ValidatedProfile::new(RecoveryProfile::new(
                depths
                    .into_iter()
                    .map(|d| RecoveryPoint::new(d, 0.5))
                    .collect()
            ))
            .is_err()
        );
    }
    let profile = ValidatedProfile::new(RecoveryProfile::new(vec![
        RecoveryPoint::new(2, 0.0),
        RecoveryPoint::new(8, 1.0),
    ]))
    .unwrap()
    .validate_scores(ScoreDomain::UnitInterval)
    .unwrap();
    assert_eq!(
        (profile.observation_count(), profile.last_depth()),
        (2, Some(8))
    );
    assert!(
        ValidatedProfile::new(RecoveryProfile::from_overlaps([f64::NAN]))
            .unwrap()
            .validate_scores(ScoreDomain::Finite)
            .is_err()
    );
    assert!(
        ValidatedProfile::new(RecoveryProfile::from_overlaps([2.0]))
            .unwrap()
            .validate_scores(ScoreDomain::UnitInterval)
            .is_err()
    );
    assert!(
        ValidatedProfile::new(RecoveryProfile::from_overlaps([2.0]))
            .unwrap()
            .validate_scores(ScoreDomain::Finite)
            .is_ok()
    );
}
#[test]
fn snapshots_reject_overflow_even_when_arm_components_are_legal() {
    for memory in [
        MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(u128::MAX))
            .with_temporary_working(StorageBits::new(1)),
        MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(1))
            .with_static_parameters(StorageBits::new(u128::MAX)),
    ] {
        assert!(memory.validate_for_arm(ReferenceArm::A1).is_ok());
        assert!(ReferenceSnapshot::new(ReferenceArm::A1, (), memory).is_err());
    }
}

#[test]
fn causal_non_query_step_ignores_memory_and_invalid_input_is_atomic() {
    use tdi_ai::{
        associative_memory::AssociativeMemoryLayout,
        assr_reference::{A2Reference, RecurrentLayout, RecurrentParameters},
    };
    let parameters = RecurrentParameters::new(
        RecurrentLayout::new(1, 1).unwrap(),
        vec![1.0],
        vec![0.0],
        vec![0.0],
    )
    .unwrap();
    let mut reference = A2Reference::new(
        parameters,
        AssociativeMemoryLayout::new(1, 1).unwrap(),
        0,
        1.0,
    )
    .unwrap();
    reference.step_without_read(&[0.75], Some(5)).unwrap();
    reference.step_without_read(&[0.25], None).unwrap();
    assert_eq!(reference.state(), &[0.25]);
    let before = reference.snapshot().unwrap();
    assert!(reference.step_without_read(&[f64::NAN], Some(5)).is_err());
    assert_eq!(before, reference.snapshot().unwrap());
}
