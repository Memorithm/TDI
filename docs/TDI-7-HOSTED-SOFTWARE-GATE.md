# TDI-7 hosted software validation gate

This document defines an additive GitHub-hosted software gate for the TDI-7.x programme.

It does **not** replace or rewrite `.github/workflows/ci.yml`, historical TDI-3/TDI-4 scientific manifests, or any TDI-5/TDI-6 workflow. It does not authorize TDI-7.2 and it does not access the final holdout.

## Purpose

The historical `Rust validation` workflow executes on the repository's self-hosted ARM64/Jetson runner. Its software jobs are useful, but runner availability can leave a commit queued without producing new software evidence.

The TDI-7 hosted gate independently executes the same three workspace-level software commands on a GitHub-hosted Ubuntu 24.04 x86_64 runner with Rust 1.97.1 pinned:

```text
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The `--locked` additions make dependency resolution explicit for the hosted reproduction and do not change the workspace dependency graph.

## Evidence semantics

A green hosted gate establishes only that the exact checked-out commit passes these three software checks on the declared hosted toolchain/platform.

It does not establish:

- that the historical self-hosted ARM64 workflow has completed;
- cross-architecture floating-point identity;
- final-holdout authorization;
- a scientific TDI-7 verdict;
- validity of historical scientific-code manifests whose current-tree relationship is handled by their own frozen protocols.

A queued or failed historical workflow must not be described as green merely because this hosted gate succeeds.

## TDI-7.2 use

For future TDI-7.2 arming, this gate is additional exact-head software evidence. The existing arming protocol remains fail-closed: every precondition named there still has to be satisfied or explicitly revised in a separately reviewed protocol change before final-holdout execution.

The hosted workflow runs on pull requests and on pushes to `main`, allowing its success to be tied to the exact merged commit rather than only a pull-request merge ref.
