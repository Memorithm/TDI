# TDI-9.3 → ElasticXxx non-final representation contract

Status: **development / non-final / non-authorizing**.

This contract exports the already-calibrated abstract C3 carrier as deterministic test data for a future ElasticXxx representation adapter. It does not authorize TDI-9.1 or TDI-9.2, does not freeze trajectory observations or thresholds, and does not expose Development, Validation, protected or final experimental material.

The source is `tdi-ai/examples/boolean_c3_elastic_interop.rs`. The versioned fixture is `interop/elasticxxx/tdi9.3-c3-carrier-v1.tsv` with schema `tdi9.3.elasticxxx-c3-carrier.v1`. Rows enumerate all 512 assignments of the existing nine abstract predicates in p0→p8 order. The checked TDI carrier classifies exactly 120 action-bearing rows, 8 well-formed unrecoverable violations and 384 invalid verifier encodings. Only checked action-bearing rows export an action; rejected rows export `-` even if the raw hand oracle would produce a branch result for a contradictory verifier encoding. Carrier validation remains authoritative.

TDI's v1 carrier is binary. It deliberately has no representation for missing predicates and therefore defines no `Unknown` value. A downstream ElasticXxx adapter may represent a missing predicate as ElasticXxx `TruthValue::Unknown`, but it must fail closed and must not reinterpret missing evidence as TDI `false`. Such a tri-state adapter is downstream representation behavior only; it cannot create a TDI action, scientific verdict, freeze pin, validation permission or actuation authority.

The fixture contains representation labels and reference actions only. It contains no trajectory samples, quality measurements, thresholds, tuning outcomes, holdout/final results or performance claims. Its purpose is differential compatibility testing. Any future replacement requires a new schema or an explicit requalification of the pinned source and fixture digest.

Run:

```bash
bash scripts/check-tdi9.3-elastic-interop.sh
```

The checker regenerates the complete fixture from the current TDI implementation, compares it byte-for-byte, verifies the full 512-row partition and action totals, and guards the non-final/non-authorizing boundary.
