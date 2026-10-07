# TDI-21 Pascal P5 Development realized-cost evidence

Status: completed Development-only timing dossier. Validation has not been
executed.

## Frozen identities

- pre-execution Git commit:
  `6eb2c5339f09ba32a5565a54ea6655e4df926e5e`
- protocol SHA-256:
  `e753296e7ee50f8fb79e99bd0b8138154572574587382abbde7df2bed94e9373`
- Development case-plan SHA-256:
  `1ed390fe876fa67579beaed8bcc68440445bcb40c272afb90aab7ced7bdd695a`
- implementation-manifest SHA-256:
  `cda7fde4d69bf234db5e203d75961032f464b2fa45fbb60a53a0088bbc6b8012`
- pre-execution freeze SHA-256:
  `9edc25274d8296ae979ffd8c78201fd33121749b5c95a74c34ad17101a9ea9ab`
- release Development binary SHA-256:
  `c5facb9c8566fc4da2e03dbb5770403042bb099a248a8540e35f8a2d2191e292`
- raw Development TSV SHA-256:
  `411783d2ef63a5ccef7467f2a280b33be5de9b7857baa5ad79132f88d023e7a7`
- canonical summary SHA-256:
  `8d60257fc49c50c3146874c16f32c71bb35bc143822c93e69de3887586e33fc1`

The run used the frozen release binary on host `tarek`, Linux
`6.8.12-tegra` aarch64, machine model `NVIDIA Jetson AGX Thor Developer Kit`,
rustc 1.98.0, CPU governor `schedutil`, pinned to CPU 0. The freeze file retains
the full rustc identity.

## Gates and exactness

All 108 declared Development cells executed. All three matched arms produced
identical checksums in every cell. Pascal and generic mismatch counts were zero
in every cell. Pairwise-token comparisons summed to zero.

Every arm retained all six integer-nanosecond timing samples, the frozen rotating
arm order, and the prospectively defined six-sample median. No timing sample was
filtered or deleted.

Development gates G1-G6 therefore hold for this dossier.

## Realized-cost result

For `delta_ns_direct = median_ns_direct - median_ns_pascal`:

- 12 cells were positive (Pascal faster);
- 0 were null;
- 96 were negative (direct ANF faster).

All 12 Pascal-faster cells were exactly the `density=4, region=high` cells,
covering both Development widths, all three query schedules, and both reuse
levels. Direct ANF was faster in every other declared Development cell.

For `delta_ns_generic = median_ns_generic - median_ns_pascal`:

- 108 cells were positive (Pascal faster);
- 0 were null;
- 0 were negative.

The median direct/Pascal realized-time ratio across the complete grid was
0.4327810438305765. The median generic/Pascal ratio was
10.362963154628464. These descriptive ratios are retained without outlier
filtering.

## P4 primitive-work versus P5 realized-time direction

The frozen P4 direct contrast had 45 Pascal-favorable and 63 direct-favorable
cells. Comparing signs prospectively against realized time:

- 63 cells were direct-favorable under both P4 work and P5 time;
- 12 cells were Pascal-favorable under both;
- 33 P4 Pascal-favorable cells were direct-favorable in realized time.

This disagreement is retained as evidence; it is not repaired by changing work
accounting or selecting favorable cells.

## Boundary

This is a host/toolchain-specific Development result only. It is not a GPU,
energy, model-quality, end-to-end latency, confirmatory, or final claim. No
protected/final population was created, opened, inspected, or executed. No
promotion into SML-GENIUS-owned model primitives, SBG, MOR, Delta-KV, context
memory, or model architecture is authorized by this result.
