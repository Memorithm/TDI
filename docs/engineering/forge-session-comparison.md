# Forge persistent-session comparison — protocol fixed 2026-09-18

This public Development extension measures the transport cost identified in the
previous adaptive comparison and tests a separately versioned earlier-feedback
policy. It does not change any frozen TDI series, historical report/profile,
default production backend, model execution permission or scientific verdict.

## Fixed comparison before execution

- Forge source: `83c8c572a692016fb786e661c47a249112b6cadd` (PR #41).
- Numerical GP dependency: SciRust `146575107005c24a47682dcaa08c4cd9464d1cc3`.
- Optuna 5.0.0 and the existing exact requirements lock.
- Same categorical grid x,y in -8..7; common (-8,-8) baseline charged as trial 1.
- No pruning, cache, parallel evaluation, omitted failed cases or automatic retry.
- Original six public tasks retained, including the disclosed non-discriminating
  categorical-interaction task. Three new fixed tasks: checkerboard-bowl
  `(x+2)^2+(y-5)^2+12*((x+y) mod 2)`; plateau-gate, with `40+abs(x-y)` outside
  `x>=1,y>=-2` and `(x-4)^2+3*(y-1)^2` inside; permuted-bowl, using independently
  permuted categorical coordinates. Separate oracles are exhaustively checked.
  All eight discriminating tasks must have baseline headroom before execution.
- Session arms: Forge random, original TPE, SciRust GP, early TPE; Optuna random,
  independent TPE, multivariate TPE, equally early multivariate TPE.
- Early Forge uses four eligible measurements in two dimensions; early Optuna
  uses four startup trials. Both original TPE comparators retain ten startup
  observations. Optuna retains 24 acquisition candidates; Forge scans admissible
  points. Equal evaluation budgets do not imply equal acquisition compute.
- `session-development`: 20 seeds (0..19), 32 calls, 9 tasks, 8 arms: 46,080 calls.
- `session-budget64`: same tasks/seeds/arms, 64 calls: 92,160 calls.
- `transport-development`: the three original tasks, 10 seeds, 32 calls, original
  Forge TPE/GP with replay and session: 3,840 calls. Paired parameter/loss/incumbent
  trajectories must match exactly; otherwise the report verifier rejects it.
- Smoke profiles use two seeds and 16 calls. Serial arm order rotates by task
  index plus seed. Each run retains all raw trajectories and observed times.
- Primary quality: final regret. Secondary: normalized incumbent AUC (including
  baseline). Time: complete adapter, verification, objective and durable IO.
  Report each task and each arm, including losses and the noninformative task.
  Do not select a winning policy, change constants/tasks or drop seeds after
  observing this execution. These known public tasks are not confirmation.

## What changes operationally

The original client launches a process and replays complete history per command.
The session client restores once, sends one command per transition and receives
a compact receipt. Its immutable hash-linked journal is fsynced before a permit
can be used. Full checkpoints remain portable and replayable. Recovery validates
sequence, byte/command bounds and hashes, then Forge reconstructs authoritative
state; an outstanding external operation still needs explicit reconciliation.

The opening record uses `tdi-forge-session-open/v2`: its identity covers the
schema version, complete specification, response and deployment binding together.
Recovery validates repository, protocol, absolute path, source-commit and binary
hash structure. The earlier candidate v1 opening identity is rejected because it
did not cover all provenance fields; it is not silently upgraded.

Tests kill an actual worker with an active permit and check that restoration and
duplicate delivery allocate no second reservation. They reject a missing/corrupt
journal entry, altered specification, changed executable, protocol/sequence
mismatch, timeout and output flood. Hashes are integrity identities, not trust
signatures. Tests do not establish power-loss durability or hostile-code isolation.

TDI's shared Linux process telemetry also uses `pidfd` exit notifications when
available. It retains `wait4` CPU/RSS readings, process-group cleanup and 5 ms
output-budget monitoring; unavailable pidfds use the original polling path.
The notification uses `poll`, supporting descriptors above `select`'s limit;
the actual process tests also run with more than 1,100 descriptors open.
Both paths are exercised with real exit, timeout and output-overflow cases.
The paired replay baseline also benefits from this common improvement; the
transport ratio therefore does not attribute all earlier polling cost to sessions.
See [Python os.pidfd_open](https://docs.python.org/3/library/os.html#os.pidfd_open)
and [Linux pidfd_open(2)](https://man7.org/linux/man-pages/man2/pidfd_open.2.html),
consulted 2026-09-18. Optuna startup settings follow its
[5.0.0 TPESampler API](https://optuna.readthedocs.io/en/stable/reference/samplers/generated/optuna.samplers.TPESampler.html).

## Reproduce

From a checkout containing this protocol, build the exact Forge revision in a
separate checkout. These commands use a new sibling directory:

```bash
git clone https://github.com/Memorithm/Forge.git ../Forge-session
git -C ../Forge-session checkout --detach 83c8c572a692016fb786e661c47a249112b6cadd
cargo +1.89.0 build --manifest-path ../Forge-session/Cargo.toml --release --locked -p forge-bridge --example scientific_search
python3 -m venv .venv-optuna
.venv-optuna/bin/python -m pip install -r scripts/requirements-optuna-benchmark.txt
PYTHONDONTWRITEBYTECODE=1 .venv-optuna/bin/python scripts/check-tdi-forge-session.py --forge-worker ../Forge-session/target/release/examples/scientific_search
PYTHONDONTWRITEBYTECODE=1 .venv-optuna/bin/python scripts/tdi_optuna_benchmark.py --forge-worker ../Forge-session/target/release/examples/scientific_search --profile transport-development --max-seconds 1800 --output session-transport
.venv-optuna/bin/python scripts/tdi_optuna_benchmark.py --verify-report session-transport/report.json
PYTHONDONTWRITEBYTECODE=1 .venv-optuna/bin/python scripts/tdi_optuna_benchmark.py --forge-worker ../Forge-session/target/release/examples/scientific_search --profile session-development --max-seconds 1800 --output session-quality32
.venv-optuna/bin/python scripts/tdi_optuna_benchmark.py --verify-report session-quality32/report.json
PYTHONDONTWRITEBYTECODE=1 .venv-optuna/bin/python scripts/tdi_optuna_benchmark.py --forge-worker ../Forge-session/target/release/examples/scientific_search --profile session-budget64 --max-seconds 1800 --output session-quality64
.venv-optuna/bin/python scripts/tdi_optuna_benchmark.py --verify-report session-quality64/report.json
```

Source commits, binary hashes, exact package versions, Python identity and source
file hashes are recorded. A declared source commit is not a build attestation.
Shared-host timings and the differing Optuna/Forge durability architectures do
not establish intrinsic optimizer speed. Preserve partial evidence on failure;
do not rerun a frozen result-conditioned subset.

## Qualification correction

The initial candidate was published as TDI `d7c0714267e4428f386f2661f31ab825fbe51552`.
Review then identified the high-descriptor `select` failure and incomplete
opening-provenance hash. Its preliminary evidence remains attributed to that
source. Qualification repeats each complete fixed profile on the corrected
source, retaining earlier completed/partial evidence. Tasks, seeds, objective
budgets, search algorithms and comparator settings do not change. This is a
robustness correction, with no result-conditioned algorithm tuning or seed retry.
