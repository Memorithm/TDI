//! Capture the identity of the compiler that actually builds `tdi-ai`.
//!
//! TDI-24 Stage-C provenance (issue #690) must record the rustc that built the
//! evaluator rather than stamping a preferred CI version. Cargo passes the
//! compiler it uses to build scripts through `RUSTC`; its `--version` output is
//! exported to the crate as `TDI_AI_BUILD_RUSTC_VERSION`. When the compiler
//! cannot be queried, the value is `unavailable` and provenance validation
//! fails closed downstream.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| String::from("rustc"));
    let version = Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty() && !text.chars().any(char::is_control))
        .unwrap_or_else(|| String::from("unavailable"));
    println!("cargo:rustc-env=TDI_AI_BUILD_RUSTC_VERSION={version}");
}
