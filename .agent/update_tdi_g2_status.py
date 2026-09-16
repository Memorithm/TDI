from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


p = Path("docs/engineering/industrialization-status.md")
s = p.read_text()
s = replace_once(
    s,
    "- Hub publication-fence prerequisite progress: `Memorithm/scirust-hub` PR #48 merged `18b6a39d48c20f98c997644b9a5d835bbc437cfe` with the versioned `PublicationFence/v1` contract, and PR #49 merged `547f71f9f57fcd850deb8fe5206b7c82d1908cae` with durable SQLite fence/publication persistence and atomic fail-closed publication checks. Hub issue #46 remains open because workflow orchestration does not yet issue those fences for attempts or require authoritative publication before downstream consumption.\n",
    "- Hub authoritative-publication prerequisite: `Memorithm/scirust-hub` PR #48 merged `18b6a39d48c20f98c997644b9a5d835bbc437cfe` with `PublicationFence/v1`, PR #49 merged `547f71f9f57fcd850deb8fe5206b7c82d1908cae` with durable SQLite fence/publication persistence, PR #50 merged `dfc14fe832fd4c5e30f228dc7075b41e0bee9118` after exact-head CI qualified attempt fencing + authoritative publication + `FromStep` consumption, and PR #51 merged `a98f77dc52caa30d055d62acc84f77b176b9bc9b` after exact-head targeted and repository CI qualified the authenticated read-only publication endpoint. Hub issue #46 is closed; TDI may consume this boundary but may not mint fences or publish outputs itself.\n",
    "Hub publication prerequisite",
)
s = replace_once(
    s,
    "- Lot G1 portable artifact binding: PR #264, final head `1d5e0d3df339ab18309b400cb7b2fbe867d48218`, merged `25292997f5ee30529e8fea08c09375d8158416bf` after all returned applicable exact-head workflows succeeded, including the dedicated `TDI Hub edge contracts` gate; no unresolved review thread remained.\n",
    "- Lot G1 portable artifact binding: PR #264, final head `1d5e0d3df339ab18309b400cb7b2fbe867d48218`, merged `25292997f5ee30529e8fea08c09375d8158416bf` after all returned applicable exact-head workflows succeeded, including the dedicated `TDI Hub edge contracts` gate; no unresolved review thread remained.\n- Lot G2 authoritative publication binding: PR #270 is candidate. Its first code/test head `72e9a58d4578b9f0a89dd7be0674d9e2b58a8c91` passed the dedicated `TDI Hub edge contracts` gate; repository-wide exact-head qualification is still required on the final PR head before merge. The contract pins Hub #51 merge `a98f77dc52caa30d055d62acc84f77b176b9bc9b`, binds exact workflow/step/attempt/generation/output lineage to the existing portable artifact binding, and keeps execution authorization false.\n",
    "Lot G2 baseline",
)
s = replace_once(
    s,
    "| G authoritative publication | blocked | Hub PR #48 qualifies `PublicationFence/v1` and PR #49 its durable SQLite repository/atomic publication path, but Hub orchestration does not yet advance a fence for each workflow-step attempt or require authoritative publication before downstream consumption. Issue #46 remains open; TDI must not implement that generic authority locally. |",
    "| G2 authoritative publication binding | candidate | Hub PR #50 qualifies attempt fencing/publication and authoritative `FromStep` consumption; Hub PR #51 exposes the read-only authenticated publication DTO. TDI PR #270 validates and content-addresses exact publication lineage while keeping `execution_authorized: false`; final exact-head qualification remains required. |",
    "G2 capability row",
)
s = replace_once(
    s,
    "| Q17 | blocked | fence contract and durable SQLite publication are qualified by Hub #48/#49; authoritative publication remains unauthorized until Hub orchestration invokes that boundary for every attempt and downstream step resolution consumes only the authoritative publication (issue #46). |",
    "| Q17 | candidate | Hub #48/#49/#50/#51 now qualify fence storage, orchestration use, downstream authoritative consumption and an authenticated read-only publication endpoint. TDI PR #270 binds that evidence to portable artifact identity; final exact-head TDI qualification is pending and execution admission remains separately unauthorized. |",
    "Q17 row",
)
s = replace_once(
    s,
    "- Hub now has a qualified `PublicationFence/v1` contract (PR #48) and durable SQLite fence/publication repository (PR #49), but the orchestrator still does not issue the fence when creating workflow-step attempts nor gate downstream inputs on the authoritative publication. TDI G2 therefore remains blocked by issue #46.\n",
    "- Hub publication authority is now qualified end-to-end for the current boundary: PR #48 defines `PublicationFence/v1`, PR #49 persists it durably, PR #50 issues a fresh fence per persisted attempt and makes downstream `FromStep` consume only authoritative publication, and PR #51 exposes a read-only authenticated publication DTO. TDI G2 consumes that evidence only; it still cannot schedule, fence, publish, lease or store on Hub's behalf.\n",
    "Hub cross-repo decision",
)
s = replace_once(
    s,
    "1. Finish and qualify Hub issue #46 by wiring the PR #48/#49 contracts into Hub orchestration: advance the fence for every authoritative workflow-step attempt, publish successful outputs through the current fence, and resolve downstream step inputs only from authoritative publication. Only then may TDI advance G2 to an authoritative edge.\n2. After that Hub boundary is qualified, bind exact component/capability pins and publication authority without duplicating Hub scheduling, leases or storage in TDI.\n",
    "1. Finish exact-head qualification and merge of TDI PR #270, then promote Q17/G2 from candidate to qualified with its final SHA evidence.\n2. Next, bind exact component/capability registry pins at Hub execution admission; until that separate boundary is qualified, `execution_authorized` remains false even for an authoritative publication binding.\n",
    "Next G2 steps",
)
p.write_text(s)
