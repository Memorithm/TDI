//! Execute public non-final TDI-22 Boolean event diagnostics (CSV on stdout).
//!
//! cargo run -p tdi-ai --features experimental --example tdi22_event_tasks

use tdi_ai::experimental::tdi22_event_tasks::{
    BooleanTask, DiagnosticPartition, TaskConfig, run_campaign,
};

fn main() {
    println!("partition,task,seed,events,fingerprint,arm,tp,fp,tn,fn,rising,falling");
    for partition in [
        DiagnosticPartition::Development,
        DiagnosticPartition::Stress,
    ] {
        for task in [
            BooleanTask::LagThreeMatch,
            BooleanTask::RisingBit,
            BooleanTask::FourBitParity,
            BooleanTask::PermutedLagThree,
            BooleanTask::NoSignalLagThree,
        ] {
            for seed in [1_u64, 17_u64] {
                let result = run_campaign(TaskConfig {
                    partition,
                    task,
                    seed,
                    events: 64,
                })
                .expect("bounded deterministic diagnostic must execute");
                for (arm, metrics) in [
                    ("instant-t3", result.metrics[0]),
                    ("instant-t4", result.metrics[1]),
                    ("impulse-t3", result.metrics[2]),
                    ("impulse-t4", result.metrics[3]),
                    ("scaled-instant-t4", result.scale_matched_t4[0]),
                    ("scaled-impulse-t4", result.scale_matched_t4[1]),
                    ("constant-negative", result.always_negative),
                ] {
                    println!(
                        "{},{},{seed},64,{:016x},{arm},{},{},{},{},{},{}",
                        partition.token(),
                        task.token(),
                        result.stream_fingerprint,
                        metrics.true_positive,
                        metrics.false_positive,
                        metrics.true_negative,
                        metrics.false_negative,
                        metrics.rising_edges,
                        metrics.falling_edges,
                    );
                }
            }
        }
    }
}
