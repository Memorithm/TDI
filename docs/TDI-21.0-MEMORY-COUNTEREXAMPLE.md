# TDI-21.0 — B3 does not dominate B2 on every retrieval

Status: **constructed development counterexample, not a trained-model benchmark**.

For the versioned XOR-shift route with salt zero, four total entries, and the implemented round-robin two-way replacement policy, write facts with identities and payloads `1, 3, 5, 9`, then query identity `3`.

B2's full-width tags map these identities to direct slots `1, 3, 1, 1`; it ends with facts `9` and `3`. B3 maps all four to one two-way bucket and ends with facts `5` and `9`. Thus B2 returns the exact fact for query `3`, while B3 correctly reports that the fact is missing from its bounded storage. Conversely, querying `5` succeeds in B3 but fails in B2. Both recover `9`.

This supplies both directions of a capacity/replacement tradeoff. The earlier pair fixture, in which B3 retains two facts that collide in one B2 slot, must not be read as universal B3 superiority. Equal entry counts also do not equalize semantic memory because B3 requires replacement metadata.

The independent task truth is the four input facts, not the content that happens to survive in either candidate. The regression `tdi21_memory_tradeoffs` asserts both positive and negative outcomes. `scripts/check-tdi21-development.sh` runs it in debug and release with the observed replies in its output. Execution results must be read from exact-head CI logs.

This is a test of these declared reference policies only. It neither refutes Boolean relational modeling nor establishes any advantage over attention. Learned routing, matched resource envelopes and held-out retrieval distributions remain separate work.

See [the causal reference contract](TDI-21.0-CAUSAL-STREAM.md) and [the bootstrap audit](TDI-21.0-AUDIT-20260914.md).
