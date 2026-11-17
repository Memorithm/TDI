# TDI-22 — Public Boolean event-task diagnostic campaigns

**Status: non-final Development-only reference code.** Not an extension
of the frozen TDI-22.1 P1–P5 experiment, not protected/confirmatory data,
not a discovery of a new computational law, and not a physical cost study.

## Why this exists

TDI-22 already contains a deterministic P1–P5 torsor episode generator.
This independent diagnostic asks a different question: when the target
is an externally defined Boolean temporal property rather than a maximum
torsor score, do four previously merged interaction rules behave
differently? It is deliberately an exploratory falsification instrument.

## Generated task families

| Family | Independent Boolean label |
|---|---|
| `lag3-match` | Current input symbol equals the symbol three events earlier (first three labels false) |
| `rising-bit` | Current input is 1 and the preceding input was 0 |
| `parity4` | XOR of the four most recent bits (first three labels false) |
| `permuted-lag3` | Lag-three labels circularly rotated by 11, with the input stream unchanged; negative control |
| `no-signal-lag3` | Lag-three labels unchanged but the source torsor resultant is zero; negative control |

A fresh, versioned SplitMix64 stream draws one source-symbol bit and
three independent nuisance-coordinate words per event. Task selection
**does not change the stream**. Labels are computed exclusively from
the source-symbol history *before* any interaction scoring. A nuisance
point P is encoded with M(P) = -P x R, leaving the origin-reduced moment
C identically zero. No source includes a task label or its history.

One keyed source is replaced at each event. Every T3/T4 observer gets
the same symbol-carrier torsor and prediction interval. The underlying
Boolean oracle has privileged access to the symbol sequence, including
on the `no-signal` negative control: that oracle is an upper-bound
reference, **not a deployable fifth predictor**.

## Fixed comparison and confounders

The four arms come from `PairedEventProtocol`: instantaneous T3/T4 and
impulse T3/T4. Hyperparameters are hard-coded in the diagnostic:
`retention=0.5`, `lower=0.25`, `upper=1.5`, `v=(1,0,0)`,
`omega=(0,0,1)`, `Q=(0,1,0)`. Each campaign contains 32–256 events
(the shipped CSV executable uses 64), with two explicit seed values
(1 and 17) per family on each of two fully public diagnostic partitions.

**Critical confounder:** with C=0 and this fixed query, T3 score is
`2*R.x` and T4 score is `R.x`. Any difference between their thresholds
can be attributable solely to this twofold score scale. A result must
**not** be interpreted as an effect of torseur transport, let alone
superiority over a matched normalized six-component control.
Neither arm is trained or calibrated on a separate split here.

### Analytically rescaled T4 controls

Two **additional** negative controls rerun T4 on the identical stream and
memory recurrence, using half the instantaneous/impulse hysteresis thresholds
(`lower=0.125`, `upper=0.75`) with `retention=0.5`. Their scale adjustment
is derived **before observation** from the algebraic factor `T3 = 2*T4`;
it is not optimized against task labels.

The diagnostic tests require the original T3 instantaneous and impulse
predictions, edges, and confusion matrices to match their respective scaled
T4 controls exactly for all five task families. The CSV includes rows
`scaled-instant-t4` and `scaled-impulse-t4`. This is a falsification of
unqualified T3-superiority claims on this *specific* C=0 content carrier,
not a global equivalence theorem for every torsor configuration.

The generator and tasks have fixed published rules. The partitions
`development` and `stress` are public and use different stream-domain
constants; **neither is a blind holdout**. The program does not select
hyperparameters from outcomes. Seed replay and label permutation
provide reproducibility and a leakage check, not statistical
independence across sampled traces.

## Reproducible executable and outputs

```bash
cargo run -p tdi-ai --features experimental --example tdi22_event_tasks
```

The program prints one CSV record per task, seed, partition and arm,
including a deterministic source/label fingerprint, confusion counts
(TP, FP, TN, FN), and rising/falling edge counts. The fifth reported
row is the constant-negative baseline, not a neural network.

No numerical results or runtime performance numbers are preclaimed
by the source. This program is an executable campaign; see its actual
stdout and commit SHA before quoting numbers. It is not run on any
protected final corpus.

## Tests and limitations

Unit tests verify historical Boolean truth tables, byte-for-byte
deterministic replay and partition separation, nuisance-coordinate
independence, all five task families executing with matched counts,
permuted labels leaving predictions unchanged, zero-signal rejection
of informative predictions, and fail-closed invalid lengths.

The benchmark protocol clones its full aggregate for transactional
reference semantics; it is O(K) per event and **not** the sparse
incremental fast path. There is no joule, RAM traffic, latency, accuracy
generalization, calibrated novelty or energy comparison claim.
The separate TDI-28 physical-meter research must supply paired and
calibrated measurements before discussing physical costs.

## Exact-commit validation

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_event_tasks
cargo run -p tdi-ai --features experimental --example tdi22_event_tasks
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```
