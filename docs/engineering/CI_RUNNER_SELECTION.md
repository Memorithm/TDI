# Software CI runner selection

The 36 contemporary software-gate workflows that previously selected the
local `tdi` runner for same-repository PRs now default to their existing
`ubuntu-24.04` hosted execution path. This avoids indefinite queueing
when the local executor is unavailable.

Set repository variable `TDI_LOCAL_RUNNERS_ENABLED` to the literal
string `true` only when the local runner is qualified and available.
Fork PRs still use hosted execution even when that opt-in is enabled.
This is explicit runner selection, not automatic liveness detection.

Commands, job names, dependencies, timeouts, checkout semantics,
permissions, protected/final boundaries and scientific gates are
unchanged. No failing test is skipped and no status is fabricated.
The change was prompted by PR #689 jobs remaining queued with label
`tdi`, runner_id 0, while other software gates completed.

Frozen historical workflows are excluded. In particular, `tdi71-ci.yml`
retains its original blob `b943f2887242536bd53924abdf0647cd0fc002fb`
and its retired Actions state; it is not a contemporary software gate.
TDI-1 through TDI-6.x historical definitions are likewise unchanged.
Read-only post-holdout integrity gates are distinct from retired workers.

Hardware-specific measurements and protected/final jobs are not
qualified or enabled by this setting. RemoteOps remains the host
control plane; hosted runs establish software evidence only.
