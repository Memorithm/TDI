# TDI-8.1 task-adapter foundation

This document records the bounded evaluator-side adapter that connects the merged symbolic T1/T2/T3 generators to the merged A0/A1/A2/A3 reference-arm APIs.

It is implementation infrastructure only. It does not run a primary experiment, select development/validation/final populations, emit an H8-A/H8-B verdict, or create a TDI-8.2 surface.

## Placement

The adapter lives in `tdi-bench`, not `tdi-ai`.

`tdi-ai` owns architecture/reference primitives. `tdi-bench` owns evaluation scheduling, task encodings, projection diagnostics, operation accounting, statistical classification and provenance. Keeping the adapter in the benchmark layer prevents task-policy choices from becoming hidden architecture semantics.

## Lossless symbolic encoding

Every symbolic `u64` is represented by two finite binary64 coordinates:

- high 32-bit limb divided by `2^32`;
- low 32-bit limb divided by `2^32`.

Both coordinates are exact binary fractions in `[0,1)`. The mapping is injective and exactly decodable for the complete `u64` domain. The decoder rejects non-finite values, values outside `[0,1)`, and values not lying on the exact `2^-32` grid.

This avoids lossy integer-to-`f64` casts above `2^53` and avoids arbitrary bit reinterpretation that could produce NaN or infinity.

## Recurrent event frame

A1/A2/A3 receive the same event frame. The first nine coordinates are fixed:

1. exact event-kind tag;
2. two coordinates for the primary identifier;
3. two coordinates for generator collision-class metadata;
4. two coordinates for the symbolic value/target/token;
5. two coordinates for the source index or payload position.

The minimum width is therefore nine. `TaskAdapterLayout` provides no experimental default. A caller may select any width `>= 9`; extra coordinates are deterministic zero padding. Concrete bounded TDI-8.1 development remains responsible for freezing an actual input/VSA width before any future final surface exists.

## A0 mapping

A0 uses a three-coordinate key and two-coordinate value:

- coordinate 0 is an exact semantic namespace tag distinguishing associative keys, delayed-copy positions and distractor inputs;
- coordinates 1-2 are the exact `u64` identifier limbs;
- values are the exact two-limb symbolic value.

Associations, copy payloads and distractors are appended to full history. Query events perform deterministic full-history reads against the matching namespace and exact identifier. Namespace separation prevents an irrelevant distractor from becoming an exact-key alias of a task target.

## A1/A2/A3 scheduling

The same recurrent-input frame is presented to A1, A2 and A3.

For A2/A3:

- associative T1/T3 events use the generated symbolic key directly as the memory key;
- T2 payload positions use a deterministic domain-separated integer key;
- association/payload events write after the recurrent transition, matching the existing A2 software-oracle operation order;
- query events read the corresponding exact logical key and do not write;
- distractor events use a deterministic read-only key that is proven absent from the instance's logical write-key set.

For A3, association/payload events additionally expose a `vsa_store_key`. The bounded evaluator policy is to store the exact recurrent-input frame under that role after the integrated A3 step. The adapter does not itself mutate A3 and therefore cannot create a partial cross-mechanism transition.

No recurrent parameters, fusion gains, table capacities, VSA seeds or dynamic-memory budgets are selected here.

## Physical collision audit

T3 `collision_class` is generator-side metadata only. It must never be substituted for physical A2/A3 collision evidence.

`audit_associative_projection` replays only the logical read/write key schedule through the concrete `DirectMappedAssociativeMemory::address_for` projection and records separately:

- planned writes;
- distinct occupied physical addresses;
- physical replacement collisions;
- query hits;
- query collision misses;
- query reads into empty addresses;
- generator-side class reuses;
- physical replacements where old/new associations also share a generator collision class.

The audit is deterministic and does not inspect or mutate recurrent states or memory payloads. It therefore measures address pressure without manufacturing a task-success or H8 result.

## What this tranche does not freeze

This tranche does not freeze:

- short/medium/long numeric horizons;
- recurrent state width;
- a concrete recurrent input width beyond the lossless minimum constraint;
- associative slot count or payload width;
- A1/A2/A3 recurrent parameters;
- associative/VSA projection seeds or fusion gains;
- matched dynamic-memory budgets;
- train/development/validation/final generator ranges;
- sample counts;
- intervention sites;
- paired-resampling replicate counts or seeds;
- rejection taxonomy for the final bounded evaluator.

It creates no final holdout, human confirmation token, confirmatory runner or TDI-8.2 result path.

## Next implementation boundary

The next evaluator layer may consume this schedule to execute A0/A1/A2/A3 and emit typed per-generator observations. Before those observations can support a primary-cell classifier, TDI-8.1 still needs explicit task output/readout semantics for recurrent arms, exact operation accounting, typed provenance/rejection records, and the preregistered paired uncertainty implementation.
