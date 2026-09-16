# TDI-2.1 — System-1/System-2 arbitration contract

Date frozen: 2026-09-16

## Outputs

The arbitration policy returns exactly one of:

- `Fast`: accept `I1` proposal;
- `Slow`: invoke the frozen `S2` reference;
- `Abstain`: produce no task answer where the task permits abstention.

## Inputs

The policy may use only quantities available before the target is revealed, including the frozen confidence signal, declared ambiguity diagnostics and static resource-budget state. It cannot inspect the protected target, future trajectory or holdout aggregate performance.

## Primary policy

A single confidence threshold selected on Validation is the primary policy. Additional multi-feature routers are exploratory unless preregistered before holdout opening.

## Hybrid utility

For each case define

`U = -L(answer,target) - lambda_slow * 1[Slow] - lambda_abstain * 1[Abstain]`.

The task-specific loss and both penalties must be frozen before PrimaryHoldout opening. Results must also report raw task quality separately so utility choices cannot hide degraded accuracy.

## Mandatory arms

- always `Fast`;
- always `Slow`;
- frozen hybrid threshold;
- random router matched to the hybrid's observed slow-call rate, using a preregistered seed.

## Decision rule for H5

The hybrid supports H5 only if its mean paired utility is better than both always-fast and always-slow under the primary frozen utility, with the statistical procedure specified in the TDI-2.1 analysis contract.

## Safety boundary

TDI-2.1 arbitration is research routing only. It grants no physical-actuation, financial, medical, security-critical or production decision authority.
