# TDI-9.3 — Checked C3 predicate carrier

The complete C3 calibration introduced in PR #236 filters invalid encodings and
unrecoverable violations before evaluating actions. The raw action-only
`BooleanPolicy` intentionally does not perform that filtering. This increment
adds a checked entry point without changing the raw IR or reference policy.

## API and ordering

Experimental module: `tdi_ai::experimental::boolean_policy_carrier`.
Schema label: `tdi9.3.reference-c3-carrier.v1`.

```rust
use tdi_ai::experimental::boolean_policy_carrier::decide_checked_c3;
use tdi_ai::experimental::boolean_policy_synthesis::reference_c3_policy;

let policy = reference_c3_policy()?;
// Existing abstract order: BASE_STOP, VERIFY_BEFORE_STOP, CADENCE_DUE,
// CHECKPOINT_AVAILABLE, REMAINING_WORK, VIOLATED, SATISFIED,
// INDETERMINATE, ABSENT.
let row = [false, false, false, false, true, false, false, false, true];
let decision = decide_checked_c3(&policy, &row)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Validation precedes policy evaluation:

1. Require exactly nine supplied Boolean values. Missing values are not false.
2. Require exactly one of the four verifier-state flags.
3. Reject VIOLATED with neither an available checkpoint nor remaining work.
4. Admit the candidate through the existing C3 synthesis envelope.
5. Evaluate its unchanged ordered rules.

`ValidatedC3PredicateRow` owns a private copy of the validated bits and exposes
only an immutable view. A caller may validate once and evaluate several admitted
candidates on that same snapshot. Carrier failures and policy admission failures
remain typed and distinct. No invalid row produces STOP, CONTINUE, VERIFY or
BACKTRACK merely because that is a candidate's fallback.

## Qualification

The exhaustive test covers all 512 abstract assignments: 120 action-bearing
rows, eight unrecoverable violations and 384 invalid verifier encodings. Every
accepted reference decision, including the existing rule-operation counts, must
match the previous raw evaluator. Other tests cover unconditional fallback,
missing/extra predicates, C2 rejection, invalid predicate indices, validation
precedence and mutation of the original input after snapshot construction.

```bash
cargo test --locked -p tdi-ai --features experimental --lib boolean_policy_carrier
bash scripts/check-tdi9-bootstrap.sh
bash scripts/check-tdi9.3-representation-calibration.sh
```

The existing experimental-contract CI gate runs these tests. No test, protected
manifest, reference oracle or freeze requirement is disabled.

## Limits and next work

The schema describes the existing abstract calibration carrier. It does not
freeze TDI-9.1 observations, measure trajectory reachability, infer Unknown from
missing data, prove a candidate correct, or authorize TDI-9.2. The raw IR remains
available and unchecked callers still bear responsibility for their input domain.
This is not yet a production solver/runtime adapter.

Returned BooleanDecision counts cover only the original expression/rule
execution. Carrier and envelope checks cost additional work and are not silently
counted as free end-to-end execution. Add separate measurements before comparing
runtime efficiency.

BooleanLab's early sparse dispatch and this C3 guard share the same validation-
before-action requirement. No BooleanLab dependency or production runtime
semantics are copied into TDI. Future SciRust/ElasticXxx adapters require a pinned
versioned contract and differential tests; TDI-specific one-hot verifier states,
ordered actions and scientific boundaries remain in TDI.
