# TDI industrialization exact-head qualification audit — 2026-09-16

This audit is a durable software-engineering evidence snapshot. It does **not** authorize or reinterpret any scientific stage, protected/final holdout, hardware result, performance claim, novelty claim, or production actuation.

`docs/engineering/industrialization-status.md` remains the durable programme handoff and explicitly delegates current exact-head qualification state to this dated audit where older Lot-H/Q19–Q30 rows conflict. This audit reports only what is verifiable from the workflow state observed now; it does not infer what required checks were or were not green at an earlier merge time unless that historical evidence is explicitly available.

## Default-branch snapshot

Audited `Memorithm/TDI` default branch: `main` at `d8785a04d112e732bf92bd115d23640e7bfb60e7` (merge of PR #332, `docs(engine): record exact-head industrialization qualification audit`).

## Industrialization lots E→H

The existing qualified foundations remain unchanged:

- Lot E checkpoint/DAG semantics: PR #254.
- Lot F artifact/provenance/export/cache contracts: PR #261.
- Lot G1 portable Hub artifact binding: PR #264.
- Lot G2 authoritative publication evidence binding: PR #270.
- Lot G3 exact Hub execution-admission binding: PR #272.
- Lot H common partner-adapter foundation: PR #273, with public G3 type-fidelity hardening in PR #275.
- Lot H Forge interchange boundary: PR #277.

This audit focuses on later merged/open work whose current exact-head evidence differs from the older handoff rows.

## Exact-head audit

| Surface | PR | Final/current head | Repository state | Exact-head workflow evidence observed in this audit | Current qualification state |
| --- | ---: | --- | --- | --- | --- |
| ElasticXxx non-actuating boundary | #279 | `29d9f5555483b59c591ccb7d4871e84ac02d677c` | merged | all workflow runs returned for this exact head are `completed/success`, including Rust, Public Rust, MSRV, Hub/artifact/common-partner and dedicated ElasticXxx contract gates | **qualified for the declared non-actuating contract only** |
| ElasticXxx root-evidence hardening | #288 | `4c97fbd14c8240649865ab70e4c7e34c92dd51f9` | merged | workflow runs observed now are queued/pending, including Rust, Public Rust, MSRV, operational-engine and dedicated ElasticXxx gates | **current exact-head qualification not established by this audit**; do not promote the hardened boundary from these observations alone |
| Operational Hub execution/recovery | #284 | `7265c669e1a274ff5900443f5ece9b18302894a5` | merged | all workflow runs returned for this exact head are `completed/success`, including Rust, Public Rust, MSRV, operational-engine, Hub-edge, artifact/provenance and partner gates | **qualified for the declared Development/Validation software path** |
| External MLflow/OTLP export/recovery | #287 | `fc85d17e250efc2ba4231a2fa5da74a89c4e0e0a` | merged | all workflow runs returned for this exact head are `completed/success`, including external-tracking qualification, Rust, Public Rust and MSRV | **qualified for the declared optional software-export boundary** |
| SciRust reusable-primitive boundary | #283 | `f05351ecf082591d2a93a33628621f98f89d81aa` | merged | dedicated SciRust, operational-engine and some repository gates succeeded; other workflow runs observed now are queued/in-progress | **current exact-head qualification not established by this audit** |
| FLAT-ATTENTION qualification boundary | #285 | `735bd1fb78644ff1124844c795e4dba7f32980b3` | merged | the workflow query returned **no runs** for this exact final head | **unverified by this audit** |
| NNIS qualification boundary | #286 | `f46eae12dae981de2057768ab64ff953da939e8e` | merged | Rust/Public Rust/MSRV, operational-engine, dedicated NNIS and multiple repository runs observed now are queued/pending | **current exact-head qualification not established by this audit**; upstream NNIS hardware qualification remains independently blocking |
| Shared replay SDK + real finite/Jacobi libraries | #331 | `58f399b139834efeb255c9e9126809762d9cb28b` | merged as `c5f6dcad6094f945af3d4df211490a76a24f8416` | the material resume-depth P1 review was resolved, but the current workflow query for this exact final head still reports the dedicated real-library, Rust/Public Rust/MSRV, operational-engine and many repository workflows queued | **merge present; current exact-head qualification not established by this audit** |
| Paired analysis consuming SciRust | #296 | `29294e801234e858e122bf32ed694acbb3190d54` | merged as `cc07d5c55bd2b5aca78b6b8074da21e508a1c735` | material review findings were resolved, but the current exact-head workflow query is queued/pending and the merged source still pins pre-final SciRust candidate `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0` | **merged but not final-integration qualified; corrective final SciRust pin still required** |
| Durable Forge scientific search through Hub | #385 | `9f1578f50ee5661545604b4e5179e8f12106c6c8` | open, non-draft, mergeable | ambiguous-submission cancellation review is resolved with a durable `cancel-requested` state; current head blocks new/already-admitted dispatch during pending cleanup and requires Hub attachment before terminal cancellation of unknown submissions; dedicated Forge-search and repository workflows remain exact-head gated | **candidate, dependency-gated by the final qualified Forge #39 revision and exact-head CI** |

## Evidence rule: merge presence is not a substitute for current qualification evidence

For #283, #285, #286, #288, #296 and #331, the current audit cannot verify a complete green exact-head suite from the workflow state it observes now. That statement is deliberately narrower than saying the PRs were merged incorrectly: queued/pending runs may be reruns or non-required workflows, and the absence of a returned run does not by itself reconstruct merge-time required-check state.

The evidence-preserving handling is therefore:

1. keep the merges and their provenance; do not rewrite history;
2. do not relabel queued, absent or unfinished workflow observations as success;
3. where exact final-head evidence is currently unavailable, collect equivalent current-main regression evidence only as **new** evidence and identify it as such;
4. keep the affected current capability state `candidate`/`unverified` in this handoff until sufficient exact-head or explicitly newer replacement evidence is recorded;
5. record negative CI outcomes and corrective PRs normally.

## Q19–Q30 implications

- **Q19 Elastic resource control:** partial. The non-actuating #279 contract has complete exact-head evidence, but the current audit does not establish qualification of merged #288 hardening and no actual ElasticXxx actuation/control effectiveness is established.
- **Q20 Forge:** candidate execution path #385 now connects a pinned Forge process to independently evaluated authoritative Hub workflows with durable checkpoints/recovery, but its final Forge dependency pin and exact-head qualification are still pending. #277 remains the qualified interchange-only foundation.
- **Q21 SciRust primitive consumption:** remains incomplete. #283 is structural and the current audit does not establish complete qualification of its final head; real primitive consumption against an independent reference remains required.
- **Q22 statistical units/exclusions/uncertainty:** code from #296 is merged, but its current exact-head evidence is incomplete and its embedded SciRust source pin is explicitly not final. Treat the integration as unqualified until the final reviewed SciRust revision is pinned and requalified in TDI.
- **Q23 sensitivity/search bounds:** active upstream candidate `Memorithm/scirust#1452`; no TDI promotion until upstream exact-head qualification and final pinning are complete.
- **Q24/Q25 operational CLI/API/viewer:** #284 exact-head workflows are fully green for its declared Development/Validation software scope. This does not extend cgroup/GPU/sandbox claims beyond the explicitly qualified paths.
- **Q26 external tracking:** #287 exact-head workflows are fully green for its declared MLflow/OTLP software boundary. Retry may duplicate remote metrics; exactly-once delivery is not claimed.
- **Q28 FLAT/NNIS hardware:** blocked/unqualified. No exact-head FLAT final-head workflow evidence was returned for #285; the current audit does not establish complete #286 qualification, and NNIS upstream hardware qualification remains separate.
- **Q30 final-head discipline:** current evidence is incomplete for #283/#285/#286/#288/#296/#331 and active candidates remain unqualified until every applicable final-head gate completes successfully. This audit records gaps without inferring historical merge-time check state that it cannot reconstruct.

## Current coding frontier

### #385 — durable Forge scientific search

The candidate adds resumable Forge ask/tell search over actual Hub-owned workflows rather than recreating scheduling in TDI. TDI persists the Forge checkpoint and the attempt-to-Hub campaign mapping before the first workflow mutation; stage evidence is consumed only through authoritative Hub artifacts and Forge retains proposal/budget/Pareto mechanics. A material cancellation review exposed an unsafe boundary when a Hub workflow had been created but its identity was not yet durably bound. Cancellation now persists a non-dispatchable `cancel-requested` state before cleanup. Unknown submissions retain explicit pending-cleanup campaign identities until the existing `runtime.attach` path reconciles the workflow. The terminal `cancelled` state requires every mapped campaign to be terminal. The regression creates a real Hub workflow, loses its response, proves pending cancellation blocks resume and already-admitted dispatch, attaches the existing workflow, then verifies cleanup completes.

The workflow currently pins Forge `0d48a91a5eed6a9ae09a5eacd7c4f18df8bdd333`, matching the current candidate of active Forge PR #39. This is therefore a candidate dependency pin, not final evidence. Merge remains forbidden until Forge #39 reaches a final reviewed exact-head-qualified revision, #385 is repinned to it, and every applicable TDI workflow is green on the resulting exact final head.

### #331 — merged replay SDK requiring evidence reconciliation

PR #331 merged as `c5f6dcad6094f945af3d4df211490a76a24f8416`. Its code adds `ReplayCodec`, bounded complete checkpoints, real `tdi_core::TableSystem` and `tdi_operator::GreenBands` adapters, and a Hub-owned four-stage replay DAG with independent modular/analytic verification. The material resume-depth finding was fixed before merge. However, the current query for the final head still reports the dedicated real-library and repository workflows queued, so this audit does not promote it to qualified solely from merge presence. New current-main regression evidence may be recorded separately when available.

### #296 / SciRust #1452 — merged TDI consumer awaiting final upstream pin

TDI #296 merged as `cc07d5c55bd2b5aca78b6b8074da21e508a1c735` while its own PR body still declared the embedded SciRust SHA `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0` candidate-only. SciRust #1452 remains open. Therefore the merged paired-analysis implementation must not be treated as finally qualified against SciRust. After #1452 is fully reviewed, exact-head green and merged, TDI needs a corrective source-pin PR to the resulting SciRust merge SHA and its own exact-head qualification before Q22/Q23 integration is promoted.

## Ownership invariants

- scirust-hub owns generic orchestration, registry resolution, leases/fencing, transport, artifact storage and authoritative publication.
- TDI owns scientific semantics, identities, admissibility, stage authorization, verdict semantics and evidence interpretation.
- ElasticXxx owns `OBSERVE→FORECAST→PLAN→VALIDATE→ACT→VERIFY→COMMIT/ROLLBACK` and physical resource semantics.
- Forge owns candidate search/synthesis/verification/measurement/selection mechanics.
- SciRust owns reusable mathematical/statistical/IR primitives.
- FLAT-ATTENTION owns attention semantics, kernels and device qualification.
- NNIS owns NVIDIA-specific runtime/kernel/device qualification.

No status in this audit grants protected-holdout access, scientific verdict authority, runtime actuation, or hardware/performance promotion.
