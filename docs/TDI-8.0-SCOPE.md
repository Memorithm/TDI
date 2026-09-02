# TDI-8.0 scope lock

This file is a compact scope guard for TDI-8.0.

## In scope

- deterministic bounded A0/A1/A2/A3 reference semantics;
- associative recall, delayed copy and interference recall;
- matched dynamic-memory accounting for A1/A2/A3;
- intervention-conditioned recovery diagnostics;
- exact metadata-inclusive memory accounting;
- deterministic operation accounting;
- preregistered paired scientific contrasts A2 vs A1 and A3 vs A2;
- preservation of negative, null, harmful and inconclusive results.

## Out of scope for TDI-8.0

- real LLM training;
- claims of Transformer replacement;
- claims of superiority to Mamba or any named external architecture;
- real GPU/Jetson latency, bandwidth, energy or occupancy claims;
- DA-LUC KV codec implementation;
- Forge-driven candidate optimization;
- a new standalone ASSR repository;
- claims of strict O(N) end-to-end runtime without a separately stated cost model;
- claims of constant total memory when external memory grows with history;
- claims that VSA dimensions are individually interpretable;
- claims that tokenization or probabilistic decoding has been eliminated.

## Holdout isolation

TDI-8.0 and TDI-8.1 must remain structurally isolated from the armed TDI-7.2
final holdout. TDI-8.2, when and only when separately frozen, reviewed and
explicitly authorized by a human at execution time, must use its own disjoint
final-holdout surface.
