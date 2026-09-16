from pathlib import Path

p = Path("docs/engineering/industrialization-status.md")
s = p.read_text()

replacements = {
'''- Lot G2 authoritative publication binding: PR #270 is candidate. Its first code/test head `72e9a58d4578b9f0a89dd7be0674d9e2b58a8c91` passed the dedicated `TDI Hub edge contracts` gate; repository-wide exact-head qualification is still required on the final PR head before merge. The contract pins Hub #51 merge `a98f77dc52caa30d055d62acc84f77b176b9bc9b`, binds exact workflow/step/attempt/generation/output lineage to the existing portable artifact binding, and keeps execution authorization false.''':
'''- Lot G2 authoritative publication binding: PR #270, final head `6c4e4d6bda4693033dc48ed405880cd609b4e5dd`, merged `7b6401e91973d4b4895d717393942aceb0cf4e6b` after every returned applicable exact-head workflow succeeded, including `TDI Hub edge contracts`, full hosted tests/Clippy/formatting, Rust/Public Rust and MSRV. The review-required output-label correction aligned publication labels with the 1..=128-byte Hub/Graph grammar while retaining the strict step-key grammar. The contract pins Hub #51 merge `a98f77dc52caa30d055d62acc84f77b176b9bc9b`, binds exact workflow/step/attempt/generation/output lineage to the existing portable artifact binding, and keeps execution authorization false.''',
'''| G2 authoritative publication binding | candidate | Hub PR #50 qualifies attempt fencing/publication and authoritative `FromStep` consumption; Hub PR #51 exposes the read-only authenticated publication DTO. TDI PR #270 validates and content-addresses exact publication lineage while keeping `execution_authorized: false`; final exact-head qualification remains required. |''':
'''| G2 authoritative publication binding | qualified | PR #270 final head `6c4e4d6bda4693033dc48ed405880cd609b4e5dd`, merge `7b6401e91973d4b4895d717393942aceb0cf4e6b`: exact-head Hub-edge, hosted full tests/Clippy/formatting, Rust/Public Rust, MSRV and all other returned gates succeeded. TDI validates and content-addresses exact authoritative publication lineage while keeping `execution_authorized: false`. |''',
'''| Q17 | candidate | Hub #48/#49/#50/#51 now qualify fence storage, orchestration use, downstream authoritative consumption and an authenticated read-only publication endpoint. TDI PR #270 binds that evidence to portable artifact identity; final exact-head TDI qualification is pending and execution admission remains separately unauthorized. |''':
'''| Q17 | qualified for authoritative publication evidence binding | Hub #48/#49/#50/#51 qualify fence storage, orchestration use, downstream authoritative consumption and the authenticated read-only publication endpoint. TDI PR #270 final head `6c4e4d6bda4693033dc48ed405880cd609b4e5dd` binds that evidence to portable artifact identity with full returned exact-head qualification. Execution admission remains separately unauthorized until Hub enforces exact component/capability pins. |''',
'''1. Finish exact-head qualification and merge of TDI PR #270, then promote Q17/G2 from candidate to qualified with its final SHA evidence.\n2. Next, bind exact component/capability registry pins at Hub execution admission; until that separate boundary is qualified, `execution_authorized` remains false even for an authoritative publication binding.\n3. Add ElasticXxx, Forge, SciRust, FLAT-ATTENTION and NNIS integrations only through qualified versioned contracts.\n4. Continue statistics/sensitivity, CLI/API/viewer, external exports and measured engine benchmarks.''':
'''1. Implement and qualify the Hub-owned exact component/capability execution-admission boundary tracked by `Memorithm/scirust-hub#52`; until that boundary is merged and consumed, `execution_authorized` remains false even for a qualified authoritative publication binding.\n2. Add the TDI-side versioned adapter for that qualified Hub admission surface without duplicating Hub registry/scheduler/lease/publication ownership.\n3. Add ElasticXxx, Forge, SciRust, FLAT-ATTENTION and NNIS integrations only through qualified versioned contracts.\n4. Continue statistics/sensitivity, CLI/API/viewer, external exports and measured engine benchmarks.''',
}

for old, new in replacements.items():
    count = s.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one status anchor, found {count}: {old[:80]!r}")
    s = s.replace(old, new, 1)

p.write_text(s)
