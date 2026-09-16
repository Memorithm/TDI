# TDI-2.1 — Experience-Conditioned Intuition Programme

Status: preregistration programme
Date opened: 2026-09-16
Parent line: TDI-2

## 1. Purpose

TDI-2.1 studies whether a bounded representation of prior experience can support a direct, non-search decision proposal on a new but structurally related situation, while preserving an explicit route to slower analysis when evidence is insufficient.

The programme extends TDI-2 without altering its frozen historical results. TDI-2 established an incremental prospective signal on the width-3 holdout while showing poor absolute transfer to width 4. TDI-2.1 therefore treats transfer, calibration and abstention as first-class research questions rather than assuming generalization.

## 2. Working definition

For this programme, an *intuition candidate* is a deterministic or explicitly reproducible mapping

\[
(x,g,M) \mapsto (\hat y, c, e)
\]

where:

- `x` is the current observation;
- `g` is optional context available at decision time;
- `M` is experience compressed before the tested decision;
- `ŷ` is an immediate proposal produced without tree search or iterative deliberation;
- `c` is a calibrated confidence quantity;
- `e` is an evidence record sufficient to audit what experience was used.

This is an operational definition for experiments. It is not a claim about human consciousness or biological intuition.

## 3. Core hypothesis

A fixed-budget pattern-retrieval mechanism trained only on Development data can improve prediction or decision quality over matched non-memory baselines on held-out situations, and can further improve risk-adjusted performance by routing low-confidence cases to a separately defined System-2 reference.

## 4. Candidate family

The initial candidate family may contain:

1. normalized pattern projection;
2. content-addressable / associative retrieval;
3. context-conditioned feature gating;
4. hierarchical pattern representations;
5. entropy- or margin-derived confidence;
6. bounded online consolidation using only permitted feedback;
7. an explicit abstain/escalate output.

No component is considered validated merely because it appears in the candidate family.

## 5. Primary scientific questions

1. Does stored experience add information beyond matched instantaneous features?
2. Does pattern retrieval transfer to unseen structural variants?
3. Is the confidence signal calibrated enough to support selective prediction?
4. Does System-1/System-2 arbitration dominate always-fast and always-slow baselines under a frozen utility function?
5. Does online consolidation improve future performance without catastrophic interference?
6. Which components are necessary under ablation?

## 6. Hard boundaries

TDI-2.1 must not:

- rewrite TDI-2 historical measurements;
- inspect protected holdout labels during design;
- equate low latency with scientific success;
- claim `O(1)` total complexity when pattern-bank size or representation dimension grows;
- treat softmax entropy as calibrated confidence without testing;
- infer biological equivalence from functional similarity;
- promote SIMD, alignment or cache assumptions without hardware evidence;
- use System-2 outputs for training unless the feedback policy explicitly permits them.

## 7. Required comparison arms

Every main experiment must include at least:

- `B0`: matched instantaneous baseline with no stored experience;
- `B1`: same model class with shuffled or unrelated experience;
- `I1`: intuition candidate using valid experience;
- `S2`: frozen deliberate/reference procedure;
- `H`: frozen arbitration policy choosing between `I1` and `S2`.

Additional associative-memory, kernel or nearest-prototype baselines may be added before holdout opening.

## 8. Evidence policy

Each result must bind:

- exact source revision;
- exact dataset/generator identity;
- split identity and seed policy;
- candidate configuration;
- baseline configuration;
- metric implementation;
- hardware identity for performance claims;
- raw result artifact digest.

## 9. Initial external context

The programme will compare its hypotheses against recent associative-memory and fast/slow reasoning work rather than presenting the architecture as unprecedented. Initial references include:

- Santos et al., *Hopfield-Fenchel-Young Networks*, arXiv:2411.08590 (2024).
- Wu et al., *Uniform Memory Retrieval with Larger Capacity for Modern Hopfield Models*, arXiv:2404.03827 (2024).
- Christakopoulou et al., *Agents Thinking Fast and Slow: A Talker-Reasoner Architecture*, arXiv:2410.08328 (2024).
- Li et al., *From System 1 to System 2: A Survey of Reasoning Large Language Models*, arXiv:2502.17419 (2025).
- Lin et al., *Controlling Thinking Speed in Reasoning Models*, arXiv:2507.03704 (2025).

These references motivate comparison classes; they do not establish TDI-2.1 results.

## 10. Stop condition for the first research cycle

The first cycle is complete only when scope, hypotheses, data contract, baselines, calibration, arbitration, consolidation, ablations, holdout policy, statistical analysis, falsification rules, reproducibility, resource accounting and implementation gate are frozen as independent reviewable artifacts.
