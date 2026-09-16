# TDI industrialization exact-head qualification audit — 2026-09-16

This audit is a durable software-engineering handoff. It does **not** authorize or reinterpret any scientific stage, protected/final holdout, hardware result, performance claim, novelty claim, or production actuation.

It supplements `docs/engineering/industrialization-status.md`, whose Lot-H rows are stale relative to the current default branch. The canonical status file must not promote a merged change to `qualified` merely because it is present on `main`; exact-head workflow evidence remains mandatory.

## Default-branch snapshot

Audited `Memorithm/TDI` default branch: `main` at `1e1c9f2d30a92c3f3c835932e39b1a13a0184f3b` (merge of PR #330, `docs(tdi-2.1): close implementation cycle two`).

## Industrialization lots E→H

The existing qualified foundations remain unchanged:

- Lot E checkpoint/DAG semantics: PR #254.
- Lot F artifact/provenance/export/cache contracts: PR #261.
- Lot G1 portable Hub artifact binding: PR #264.
- Lot G2 authoritative publication evidence binding: PR #270.
- Lot G3 exact Hub execution-admission binding: PR #272.
- Lot H common partner-adapter foundation: PR #273, with public G3 type-fidelity hardening in PR #275.
- Lot H Forge interchange boundary: PR #277.

This audit focuses on later merged/open work whose exact-head state changed after those rows were written.

## Exact-head audit

| Surface | PR | Final/current head | Repository state | Exact-head workflow evidence observed in this audit | Qualification state |
| --- | ---: | --- | --- | --- | --- |
| ElasticXxx non-actuating boundary | #279 | `29d9f5555483b59c591ccb7d4871e84ac02d677c` | merged | all workflow runs returned for this exact head are `completed/success`, including Rust, Public Rust, MSRV, Hub/artifact/common-partner and dedicated ElasticXxx contract gates | **qualified for the declared non-actuating contract only** |
| ElasticXxx root-evidence hardening | #288 | `4c97fbd14c8240649865ab70e4c7e34c92dd51f9` | merged | workflow runs are still queued/pending, including Rust, Public Rust, MSRV, operational-engine and dedicated ElasticXxx gates | **not yet qualified**; current `main` must not be described as fully qualified for the hardened boundary |
| Operational Hub execution/recovery | #284 | `7265c669e1a274ff5900443f5ece9b18302894a5` | merged | all workflow runs returned for this exact head are `completed/success`, including Rust, Public Rust, MSRV, operational-engine, Hub-edge, artifact/provenance and partner gates | **qualified for the declared Development/Validation software path** |
| External MLflow/OTLP export/recovery | #287 | `fc85d17e250efc2ba4231a2fa5da74a89c4e0e0a` | merged | all workflow runs returned for this exact head are `completed/success`, including external-tracking qualification, Rust, Public Rust and MSRV | **qualified for the declared optional software-export boundary** |
| SciRust reusable-primitive boundary | #283 | `f05351ecf082591d2a93a33628621f98f89d81aa` | merged | dedicated SciRust, operational-engine and some repository gates succeeded, but Rust/Public Rust/MSRV and multiple other workflows remain queued/in-progress | **not yet qualified** |
| FLAT-ATTENTION qualification boundary | #285 | `735bd1fb78644ff1124844c795e4dba7f32980b3` | merged | the workflow query returned **no runs** for this exact final head | **unverified / not qualified** |
| NNIS qualification boundary | #286 | `f46eae12dae981de2057768ab64ff953da939e8e` | merged | Rust/Public Rust/MSRV, operational-engine, dedicated NNIS and multiple repository gates remain queued/pending | **not yet qualified**; upstream NNIS hardware qualification remains independently blocking |
| Shared replay SDK + real finite/Jacobi libraries | #331 | `f565da607bc834adf49737cdf117808fac4bc4c2` | open, non-draft, mergeable | dedicated real-library, Rust/Public Rust/MSRV, operational-engine and repository gates were queued at audit time; no unresolved review thread was returned | **candidate** |
| Paired analysis consuming SciRust | #296 | `0a917a1abf03c3a6c9aff9400ca642e80d310b3c` | open, non-draft, mergeable | dedicated analysis and repository gates were queued/pending; both material P2 review threads are resolved | **candidate, dependency-gated by SciRust #1452** |

## Procedural finding: merge presence is not qualification

PRs #283, #285, #286 and #288 are already merged even though the exact-head audit above does not show complete green qualification on their final heads. This is a process defect relative to the industrialization rule requiring complete exact-head gates before merge. It must not be repaired by relabelling queued, absent or unfinished checks as success.

The safe recovery is evidence-preserving:

1. keep the merges and their provenance; do not rewrite history;
2. allow the exact final-head workflows to complete where runs exist;
3. for an exact final head with no returned workflow runs (#285), exercise an equivalent current-main regression only as **new** evidence and do not pretend it existed before merge;
4. keep the affected Lot-H capability state `candidate`/`unverified` until the required evidence exists;
5. record any negative CI outcome and corrective PR normally.

## Q19–Q30 implications

- **Q19 Elastic resource control:** partial. The non-actuating #279 contract has complete exact-head evidence, but merged #288 hardening remains unqualified and no actual ElasticXxx actuation/control effectiveness is established.
- **Q20 Forge:** still partial. #277 qualifies interchange only; it does not establish a real Forge PROPOSE/MUTATE→COMPILE→VERIFY→MEASURE→SELECT execution path.
- **Q21 SciRust primitive consumption:** remains incomplete. #283 is structural and its final head is not fully qualified; real primitive consumption against an independent reference remains required.
- **Q22 statistical units/exclusions/uncertainty:** active candidate #296. It keeps unit weighting, explicit exclusions and non-assessed scientific verdicts, but remains dependency/CI gated.
- **Q23 sensitivity/search bounds:** active upstream candidate `Memorithm/scirust#1452`; no TDI promotion until upstream exact-head qualification and final pinning are complete.
- **Q24/Q25 operational CLI/API/viewer:** #284 exact-head workflows are now fully green for its declared Development/Validation software scope. This does not extend cgroup/GPU/sandbox claims beyond the explicitly qualified paths.
- **Q26 external tracking:** #287 exact-head workflows are fully green for its declared MLflow/OTLP software boundary. Retry may duplicate remote metrics; exactly-once delivery is not claimed.
- **Q28 FLAT/NNIS hardware:** blocked/unqualified. No exact-head FLAT final-head workflow evidence was returned for #285; #286 workflows remain incomplete and NNIS upstream hardware qualification remains separate.
- **Q30 final-head discipline:** open procedural debt because #283/#285/#286/#288 were merged without currently verifiable complete exact-head green evidence.

## Current coding frontier

### #331 — reusable real-library adapter SDK

The candidate adds `ReplayCodec` on top of the existing `ReplayAdapter`, bounded complete checkpoint codecs, actual `tdi_core::TableSystem` and `tdi_operator::GreenBands` adapters, and a Hub-owned four-stage prefix/resume/full/verify DAG. The verifier uses independent modular/analytic oracles and compares split replay with uninterrupted execution. It does not add a scheduler, protected population, GPU execution, scientific confirmation, or performance claim.

Merge remains forbidden until the dedicated real-library workflow and every applicable exact-head repository gate are complete and green.

### #296 / SciRust #1452 — general analysis

TDI #296 remains non-confirmatory and preserves explicit unit/exclusion accounting. Its two material review findings are resolved, but it depends on SciRust #1452. The SciRust PR itself remains open and its exact-head workflows were queued/pending during this audit. TDI must record the final reviewed SciRust source pin before integration; a temporary candidate SHA is not sufficient.

## Ownership invariants

- scirust-hub owns generic orchestration, registry resolution, leases/fencing, transport, artifact storage and authoritative publication.
- TDI owns scientific semantics, identities, admissibility, stage authorization, verdict semantics and evidence interpretation.
- ElasticXxx owns `OBSERVE→FORECAST→PLAN→VALIDATE→ACT→VERIFY→COMMIT/ROLLBACK` and physical resource semantics.
- Forge owns candidate search/synthesis/verification/measurement/selection mechanics.
- SciRust owns reusable mathematical/statistical/IR primitives.
- FLAT-ATTENTION owns attention semantics, kernels and device qualification.
- NNIS owns NVIDIA-specific runtime/kernel/device qualification.

No status in this audit grants protected-holdout access, scientific verdict authority, runtime actuation, or hardware/performance promotion.