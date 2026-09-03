# TDI-8.1 exact recurrent-state readout preflight

The merged symbolic executor requires each concrete arm adapter to return one exact `TaskSymbol` at query time. A0 already exposes an exact contextual value vector, while A1/A2/A3 currently expose recurrent state but no symbolic decoder.

This tranche qualifies a deliberately minimal **target-blind exact readout candidate** before concrete arm adapters are built.

## Candidate contract

A caller supplies:

- recurrent `state_width`;
- one state coordinate for the encoded high 32-bit limb;
- one distinct state coordinate for the encoded low 32-bit limb.

No default positions are provided.

At query time the readout receives only the complete arm state. It:

1. requires exact runtime state width;
2. requires every state coordinate to be finite;
3. reads the two caller-selected coordinates;
4. passes them through the same canonical lossless two-limb decoder qualified in PR #112;
5. returns the resulting `TaskSymbol` only if both coordinates are exact canonical limbs.

The candidate has no target argument, no source-index argument, no collision-class argument, no candidate vocabulary, no nearest-neighbour lookup, no learned decoder, no tolerance and no rounding path.

## Why exact rather than rounded

The A2/A3 reference memories store binary64 recurrent payloads. An exact readout allows future bounded adapter work to test whether the architecture actually places a retrievable symbolic representation into designated state coordinates. Introducing a nearest-symbol table or tolerant decoder here would add another memory/estimation mechanism and could hide representational error.

Exact decoding can therefore fail frequently, especially for A1. Such failures are valid bounded evidence about a concrete configuration; they are not repaired by the readout layer.

This is a **candidate**, not the final TDI-8.1 readout freeze. Bounded non-final work may reject it if it makes the comparison ill-posed, but any replacement must be preregistration-compatible, target-blind, explicitly accounted and frozen before a confirmatory stage can exist.

## A0 compatibility

A0 values already use the exact two-coordinate `u64` encoding. The same target-blind helper can decode those value coordinates. This keeps the final symbolic output representation shared across A0/A1/A2/A3 without giving recurrent arms a target or vocabulary side channel.

## Qualification checks

The software preflight verifies:

- exact round-trip from caller-selected state coordinates;
- arbitrary valid coordinate placement rather than hard-coded positions;
- rejection of state widths below two coordinates;
- rejection of duplicate/out-of-range coordinate selections;
- rejection of runtime state-width drift;
- rejection of non-finite recurrent state;
- rejection of off-grid/noncanonical limbs instead of rounding;
- A0 exact value coordinates use the same decoder.

## Scientific boundary

This tranche does not:

- freeze state width or readout coordinate indices;
- freeze A1/A2/A3 recurrent parameters;
- define training/fitting or model-selection rules;
- define the late retrieval deficit;
- implement complete A0/A1/A2/A3 `SymbolicTaskAdapter`s;
- claim that exact readout is optimal;
- emit H8-A/H8-B evidence;
- create/access a TDI-8.2 surface;
- access or reinterpret TDI-7.2 evidence.

The next adapter tranche must keep this readout target-blind, account any additional state/static/temporary resources, and separately report physical A2/A3 associative occupancy/collision behavior.
