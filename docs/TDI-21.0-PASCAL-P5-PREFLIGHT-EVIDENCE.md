# TDI-21 Pascal P5 preflight evidence

Status: Development/Validation-only preflight. No scientific P5 timing has been executed by this record.

## Candidate identity

- branch: `research/tdi21-pascal-p5-realized-cost-prereg`
- exact candidate commit before this evidence record: `71865a8aed29eabe335b2a78c8bc136905367492`
- protocol SHA-256: `e753296e7ee50f8fb79e99bd0b8138154572574587382abbde7df2bed94e9373`
- P4 matched-workload implementation SHA-256: `2a843c729460586a3bb2a8afb3b4b0c4c092ecd1b938f8169a67085a7480241c`
- P5 timing-semantics implementation SHA-256: `6b85b1bfbed92ab8e9039c0f2660727e027769d6f95494d3e3ffaaeeac341705`
- P5 preflight test SHA-256: `eff11f1edaa139332a4f19dfebba5afd9a91aad6f0e817e3a664997103138076`
- frozen case-plan SHA-256: `adf6564d2851112ce418a5366bacf3a0804ac62eceebc7b1a94a5f0f31637680`

The case plan is the already-frozen P4 `case-plan.tsv`, reused only because P5 prospectively declares the identical 162-cell synthetic geometry. Reuse does not count as a new P4 replication.

## Host and toolchain identity

Preflight host:

- device model: NVIDIA Jetson AGX Thor Developer Kit
- architecture: aarch64
- CPU topology: 14 online CPUs, one thread per core, one socket
- BIOS CPU description: Thor Not Specified CPU @ 2.6GHz
- kernel: Linux 6.8.12-tegra aarch64 GNU/Linux
- rustc: 1.98.0 (88d9e12ae 2026-08-18)
- rustc host: aarch64-unknown-linux-gnu
- LLVM: 22.1.8

This identity is prospective provenance for subsequent Development timing. Any scientific P5 Development timing must bind its own exact execution commit and re-check that the frozen protocol, implementation, case-plan, host, and toolchain identities have not drifted.

## Preflight qualification

A clean source snapshot of the exact branch head was downloaded from GitHub and qualified on the declared host.

- `cargo fmt --all -- --check`: PASS.
- `cargo test -p tdi-ai`: PASS.
- `cargo clippy -p tdi-ai -- -D warnings`: PASS.
- P5 preflight geometry remains Development 108 cells / Validation 54 cells.
- timing rounds remain one warm-up plus six recorded rounds with the frozen rotating arm order.
- no protected/final fixture or confirmatory path was opened or executed.

The earlier clippy failure caused by path-imported P4 items in the P5 integration test was corrected by restricting `dead_code` allowance to that preflight integration-test crate. No scientific endpoint or timing rule changed.

## Boundary

This record qualifies only the P5 preflight and provenance surface. It does not contain realized CPU timings, does not authorize Validation, and does not promote Pascal into SML-GENIUS, SBG, MOR, Delta-KV, context memory, or model architecture.
