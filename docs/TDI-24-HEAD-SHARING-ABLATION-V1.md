# TDI-24 head-sharing ablation v1 (slice 37)

Status: Phase-D Development/Validation software ablation. Contract
`tdi24-head-sharing-ablation-v1`. No training, no protected/final access, no
confirmatory execution and no scientific claim. No configuration freeze pin is
introduced or changed; holdouts are untouched.

## Arms

Both arms are two-head C6 scorers (`HEAD_SHARING_HEAD_COUNT = 2`); the score is
the mean of the head scores, summed in head order. Each head is the C6
reference `(alpha, beta, gamma) = (1, 0, 1)` scored in one slice-35 fixed
mirror basis (`tdi24-fixed-m-sensitivity-v1`).

- **Shared**: one chiral structure for every head, the canonical basis
  (rank 0) on both heads.
- **Per-head**: head `h` uses the basis of rank `PER_HEAD_BASIS_RANKS[h] =
  [0, 1]`, i.e. the canonical basis and the next basis in the frozen
  lexicographic order (`H+ = {0,1,3}`). This is a declared rule of the
  ablation, not tuned or selected from outcomes, and not a freeze pin.

Matched capacity: identical weights, identical head count and
`TrainableCapacity::reference_c6()` on both arms (bases are fixed relabellings
with zero trainable parameters).

## Checked identities and rejections

- **Shared reproduction**: the shared arm equals the single-head Stage-C C6
  score bit for bit (`(s + s) / 2 = s` exactly) (`shared_reference_drift`).
- **Canonical head**: per-head head 0 equals the reference bit for bit
  (`canonical_head_drift`).
- **Head mean**: the per-head score is exactly the mean of its stored head
  scores (`head_mean_drift`); tests also check each head against the slice-35
  study bit for bit.
- **Regenerated evidence** (`case_evidence_drift`), contract, basis-contract,
  head-count, basis-rank, weight and capacity drift, case count, tampered
  summaries and access/training/claim flags are rejected. Protected/final
  labels never generate a case.

## Smoke-budget software diagnostics

Budget `StageCPreflightBudget::bounded(8, 0)`, Development (16 cases per
family):

| Family | Shared correct | Per-head correct | Disagreements |
| --- | ---: | ---: | ---: |
| ReflectionDiscriminative | 16/16 | 8/16 | 8 |
| ReflectionNuisance | 12/16 | 12/16 | 0 |
| DirectionReversal | 16/16 | 8/16 | 8 |
| NonChiralControl | 8/16 | 16/16 | 8 |

## Recorded degeneracies

1. **Development equals Validation on this budget**, inherited from the
   Stage-C case stream (slice 30).
2. **The shared arm is the single head.** With identical heads, head sharing
   is an exact identity at zero capacity; differences only come from the
   per-head structure.
3. **One declared per-head rule.** Only ranks `[0, 1]` are evaluated; other
   per-head assignments are not explored and nothing is selected. A complement
   pair (`[0, 19]`) would cancel `chi` exactly and is not used.
4. **Beta = 0.** Only the parity-odd channel depends on the basis.

Interpreting these counts as attribution evidence is reserved for the Stage-D
attribution audit (slice 40).
