use tdi_ai::adapter_sdk::{ReplayCodec, check_codec_conformance};
use tdi_ai::experiment::{
    Collection, NoiseCoupling, ReplayAdapter, RunLimits, StepContext, run_paired,
};
use tdi_bench::engine_adapters::{AdapterError, FiniteCycle, JacobiSweep};

fn contexts() -> Vec<StepContext> {
    (1..=6)
        .map(|depth| StepContext {
            depth,
            noise_stream: 0,
        })
        .collect()
}

#[test]
fn finite_library_codec_branches_oracle_and_intervention() {
    let source = FiniteCycle::new(3).unwrap();
    check_codec_conformance(&source, &contexts()).unwrap();
    let original = source.encode_checkpoint().unwrap();
    let mut left = source.fork(&source.checkpoint().unwrap()).unwrap();
    for context in contexts() {
        assert_eq!(
            left.advance(context).unwrap(),
            (3 + context.depth as u64) % 4
        );
        let raw = left.encode_checkpoint().unwrap();
        let restored = source
            .fork(&source.decode_checkpoint(&raw).unwrap())
            .unwrap();
        assert_eq!(restored.encode_checkpoint().unwrap(), raw);
    }
    assert_eq!(source.encode_checkpoint().unwrap(), original);
    assert!(source.flipped(2).is_err());
    assert_ne!(source.flipped(0).unwrap(), source.checkpoint().unwrap());
    let bad = StepContext {
        depth: 7,
        noise_stream: 1,
    };
    let before = left.encode_checkpoint().unwrap();
    assert_eq!(left.advance(bad), Err(AdapterError::InvalidContext));
    assert_eq!(left.encode_checkpoint().unwrap(), before);
}

#[test]
fn jacobi_library_matches_independent_two_by_two_inverse() {
    let source = JacobiSweep::new(vec![4.0, 4.0], vec![1.0], 0.0).unwrap();
    check_codec_conformance(&source, &contexts()).unwrap();
    let original = source.encode_checkpoint().unwrap();
    let mut left = source.clone();
    for context in contexts() {
        let result = left.advance(context).unwrap();
        // GreenBands evaluates (J + shift I)^-1; direct 2x2 inverse oracle.
        let diagonal = 4.0 + 0.25 * context.depth as f64;
        let expected = diagonal / (diagonal * diagonal - 1.0);
        assert!(result.iter().all(|x| (x - expected).abs() <= 2e-15));
        let raw = left.encode_checkpoint().unwrap();
        let replay = source
            .fork(&source.decode_checkpoint(&raw).unwrap())
            .unwrap();
        assert_eq!(replay.encode_checkpoint().unwrap(), raw);
    }
    assert_eq!(source.encode_checkpoint().unwrap(), original);
    let mut other = source.fork(&source.shifted(0.5).unwrap()).unwrap();
    assert_ne!(
        other.advance(contexts()[0]).unwrap(),
        source.clone().advance(contexts()[0]).unwrap()
    );
    assert!(source.shifted(f64::NAN).is_err());
    let mut invalid = JacobiSweep::new(vec![-10.0], vec![], 0.0).unwrap();
    let before = invalid.encode_checkpoint().unwrap();
    assert_eq!(
        invalid.advance(contexts()[0]),
        Err(AdapterError::LibraryRejected)
    );
    assert_eq!(invalid.encode_checkpoint().unwrap(), before);
}

#[test]
fn codecs_reject_truncation_trailing_wrong_matrix_and_corrupted_cache() {
    let finite = FiniteCycle::new(0).unwrap();
    let raw = finite.encode_checkpoint().unwrap();
    for size in 0..raw.len() {
        assert!(finite.decode_checkpoint(&raw[..size]).is_err());
    }
    let mut bad = raw;
    bad.push(0);
    assert!(finite.decode_checkpoint(&bad).is_err());
    let mut jacobi = JacobiSweep::new(vec![4.0; 2], vec![1.0], 0.0).unwrap();
    jacobi.advance(contexts()[0]).unwrap();
    let raw = jacobi.encode_checkpoint().unwrap();
    for size in 0..raw.len() {
        assert!(jacobi.decode_checkpoint(&raw[..size]).is_err());
    }
    let other = JacobiSweep::new(vec![5.0; 2], vec![1.0], 0.0).unwrap();
    assert!(other.decode_checkpoint(&raw).is_err());
    let mut bad = raw.clone();
    bad.push(0);
    assert!(jacobi.decode_checkpoint(&bad).is_err());
    let mut bad = raw;
    *bad.last_mut().unwrap() ^= 1;
    assert!(jacobi.decode_checkpoint(&bad).is_err());
}

#[test]
fn actual_adapter_zero_horizon_cancellation_metric_and_sink_errors() {
    let source = FiniteCycle::new(0).unwrap();
    let checkpoint = source.checkpoint().unwrap();
    let run = |horizon, cancel, metric_fail, sink_fail| {
        run_paired(
            &source,
            &checkpoint,
            &checkpoint,
            RunLimits::new(horizon, 64, 64, Collection::All).unwrap(),
            NoiseCoupling::Deterministic,
            |a: &u64, b: &u64| {
                if metric_fail {
                    Err("metric")
                } else {
                    Ok(a.abs_diff(*b))
                }
            },
            |_| cancel,
            |_, _: &u64| if sink_fail { Err("sink") } else { Ok(()) },
        )
        .unwrap()
    };
    assert!(run(0, false, false, false).failure.is_none());
    for (cancel, metric, sink) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let result = run(4, cancel, metric, sink);
        assert!(result.failure.is_some());
        assert_eq!(result.completed_depth, 0);
        assert!(result.profile.points().is_empty());
    }
    assert_eq!(source.checkpoint().unwrap(), checkpoint);
}

#[test]
fn separately_owned_parallel_sessions_match_serial_results() {
    let factory = JacobiSweep::new(vec![4.0; 2], vec![1.0], 0.0).unwrap();
    let evaluate = |mut session: JacobiSweep| {
        contexts()
            .into_iter()
            .map(|c| session.advance(c).unwrap())
            .collect::<Vec<_>>()
    };
    let expected = evaluate(factory.clone());
    let jobs: Vec<_> = (0..4)
        .map(|_| {
            let session = factory.clone();
            std::thread::spawn(move || evaluate(session))
        })
        .collect();
    for job in jobs {
        assert_eq!(job.join().unwrap(), expected);
    }
    assert_eq!(factory.progress(), 0);
}
