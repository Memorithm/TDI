# Merge qualification policy

This document records the repository-level merge policy for TDI engineering and
scientific work. It does not authorize a scientific stage, holdout access,
runtime actuation, or a performance claim.

## Server-enforced baseline

As verified on 2026-09-17, the `main` branch is protected with strict
up-to-date status checks and enforcement for administrators. Pull requests are
required, unresolved review conversations block merge, force-push and branch
deletion are disabled, and zero human approvals are required so the autonomous
workflow does not depend on a manual reviewer.

The globally required status contexts are:

- `Formatting`
- `Tests`
- `Clippy`
- `Preregistration integrity`
- `Public formatting`
- `Public tests`
- `Public clippy`
- `Public preregistration integrity`
- `Rust MSRV`

These contexts are the server-enforced baseline. They are not a substitute for
conditional scientific, hardware, partner, or integration gates.

## Conditional exact-head gates

A pull request may trigger additional workflows according to its changed paths,
features, experiment series, hardware requirements, or integration boundary.
Every such applicable workflow must complete successfully on the pull request's
exact final head SHA before merge. `queued`, `pending`, `in_progress`,
`failure`, `cancelled`, `timed_out`, or stale results from an older SHA are not
green evidence.

A workflow intentionally absent because its declared path filter does not match
the pull request is not applicable. A workflow that should apply according to
its declared trigger but is unexpectedly absent is a merge blocker until the
missing evidence is explained or restored.

Examples of conditional surfaces include Hub edge contracts, artifact and
provenance contracts, partner adapters, real-library integration, Linux
containment, external tracking, measured engine benchmarks, hardware-specific
qualification, and series-specific preregistration or experiment gates.

## Scientific authority boundaries

CI success qualifies only the software/evidence surface named by that gate. It
does not by itself:

- open a protected or final holdout;
- change a preregistered H0/H1 decision rule;
- grant TDI scheduling, lease, transport, or artifact-store ownership that
  belongs to scirust-hub;
- grant partner execution or actuation authority;
- establish novelty, proof, performance, or hardware generality.

Scientific verdicts remain governed by the experiment contract and the owning
research programme. ProofLab may use `PROVED` only for kernel-checked proof
artifacts under its declared proof boundary.

## Merge procedure

For each candidate:

1. start from the current default branch and use a dedicated branch;
2. resolve all material review findings and conflicts;
3. identify the exact final head SHA;
4. verify the globally required contexts on that SHA;
5. verify every additional applicable workflow on that same SHA;
6. merge only a non-draft, mergeable, conflict-free pull request;
7. after merge, use the new `main` as the base for dependent work;
8. retain negative results, provenance, holdout boundaries, and failed
   qualification evidence rather than rewriting them as success.

If a later commit changes the candidate head, prior exact-head qualification is
stale and the applicable gates must qualify the new head again.

## Auditable exception path

There is no standing branch-protection bypass: administrators are subject to the
same protected-main baseline and the repository has no bypass actor configured.
An emergency exception therefore requires an explicit, temporary administrative
policy change rather than a silent merge override.

Any such exception must be recorded before use in a dedicated issue or incident
record with the exact base/head SHAs, the blocked context, the reason normal
qualification cannot complete, the approving repository administrator, and the
bounded restoration condition. The protection change itself and its restoration
must be captured through the repository administration audit trail, and the live
protection endpoint must be re-read after restoration.

An exception cannot authorize protected/final holdout access, weaken a frozen
scientific decision rule, substitute missing hardware evidence, or relabel a
failed/cancelled/missing conditional gate as success. If those gates are material
to the change, the candidate remains scientifically or operationally unqualified
even if an emergency repository update is administratively permitted.

## Runner-load path scoping

Path filters are allowed only to remove genuinely inapplicable workflows. They
must not suppress a gate whose code, preregistration, evidence, or hardware
boundary is affected by the pull request. The server-required baseline remains
unfiltered. Engineering-document-only changes may skip unrelated experimental
series gates when those gates do not consume `docs/engineering/**`.
