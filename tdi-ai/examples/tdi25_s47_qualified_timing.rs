//! TDI-25 slice 47: one qualified timing cell under the frozen T430 protocol.
//!
//! Usage: `tdi25_s47_qualified_timing <development|validation> <t6|c6>`.
//! Meant to be launched only by `scripts/tdi25-s47-qualified-timing.sh`, which
//! applies pinning, NUMA binding, governor/turbo and load/SMT checks. The
//! harness itself refuses unless the host attests to the frozen manifest.
//! Prints one JSON object on stdout. No rerun: a rejected cell is reported as
//! non qualifiee.

use std::process::ExitCode;

use tdi_ai::experimental::tdi25_eval::{
    QUALIFIED_TIMING_ENVIRONMENT_CONTRACT, StageCPreflightBudget,
    run_qualified_reference_timing_cell, summarize_timing_cell,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::ComparisonArm;

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn join<T: ToString>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (split, split_name) = match args.first().map(String::as_str) {
        Some("development") => (DataSplit::Development, "development"),
        Some("validation") => (DataSplit::Validation, "validation"),
        _ => {
            eprintln!("usage: tdi25_s47_qualified_timing <development|validation> <t6|c6>");
            return ExitCode::from(2);
        }
    };
    let (arm, arm_name) = match args.get(1).map(String::as_str) {
        Some("t6") => (ComparisonArm::T6, "T6"),
        Some("c6") => (ComparisonArm::C6, "C6"),
        _ => {
            eprintln!("usage: tdi25_s47_qualified_timing <development|validation> <t6|c6>");
            return ExitCode::from(2);
        }
    };
    let samples =
        match run_qualified_reference_timing_cell(split, StageCPreflightBudget::smoke(), arm) {
            Ok(samples) => samples,
            Err(error) => {
                eprintln!("timing refused: {error}");
                return ExitCode::from(3);
            }
        };
    let summary = match summarize_timing_cell(&samples) {
        Ok(summary) => summary,
        Err(error) => {
            eprintln!("summary refused: {error}");
            return ExitCode::from(3);
        }
    };
    let a = &samples.attestation;
    println!(
        "{{\"environment_contract\":{},\"machine_id\":{},\"split\":{},\"arm\":{},\"budget\":\"smoke\",\
\"cases\":{},\"warmup_iterations\":{},\"measured_iterations\":{},\"nanos\":[{}],\
\"median_nanos\":{},\"min_nanos\":{},\"q1_nanos\":{},\"q3_nanos\":{},\"iqr_nanos\":{},\
\"fixed_frequency_khz\":{},\"frequency_khz\":[{}],\"swap_in\":[{},{}],\"swap_out\":[{},{}],\
\"peak_rss_kib_informational\":{},\"iqr_ok\":{},\"frequency_ok\":{},\"swap_ok\":{},\
\"qualified_in_process\":{},\"attestation\":{{\"cpu_model\":{},\"kernel_release\":{},\
\"bios_version\":{},\"cpus_allowed\":{},\"governor\":{},\"sibling_governor\":{},\"no_turbo\":{},\
\"build_rustc\":{}}}}}",
        json_string(QUALIFIED_TIMING_ENVIRONMENT_CONTRACT),
        json_string(samples.environment.machine_id),
        json_string(split_name),
        json_string(arm_name),
        samples.cases,
        samples.environment.warmup_iterations,
        samples.environment.measured_iterations,
        join(&samples.nanos),
        summary.median_nanos,
        summary.min_nanos,
        summary.q1_nanos,
        summary.q3_nanos,
        summary.iqr_nanos,
        samples.fixed_frequency_khz,
        join(&samples.frequency_khz),
        samples.swap_in_start,
        samples.swap_in_end,
        samples.swap_out_start,
        samples.swap_out_end,
        samples
            .peak_rss_kib
            .map_or_else(|| "null".to_string(), |value| value.to_string()),
        summary.iqr_ok,
        summary.frequency_ok,
        summary.swap_ok,
        summary.qualified_in_process,
        json_string(&a.cpu_model),
        json_string(&a.kernel_release),
        json_string(&a.bios_version),
        json_string(&a.cpus_allowed),
        json_string(&a.governor),
        json_string(&a.sibling_governor),
        json_string(&a.no_turbo),
        json_string(a.build_rustc),
    );
    ExitCode::SUCCESS
}
