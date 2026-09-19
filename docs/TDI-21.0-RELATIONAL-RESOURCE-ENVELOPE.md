# TDI-21.0 — Relational declared-component resource envelope

Status: **development-only accounting control; stacked after relational B0/B1 controls; not frozen; not confirmatory**.

This slice prepares a later fair comparison without pretending that the current software representations already have matched total cost.

## Scope

The envelope equalizes only the representation components already reported by the current reference implementations:

- B0/B1 attention history component bits;
- B0/B1 score-buffer component bits;
- B4 two-way Boolean memory entry/replacement bits;
- B4 relational address-program semantic bits.

It explicitly excludes allocator bookkeeping, temporary stack state, executable code, evaluator/oracle state, search/training cost, tuning, bandwidth, wall-clock latency and energy.

Therefore:

```text
equal declared component ceiling != equal total system budget
```

## Construction

For one valid attention configuration, its declared component ceiling is:

```text
history_component_bits + score_component_bits
```

The Boolean side reserves the supplied ANF address-program bits first, then chooses the largest even two-way B3 slot count whose current semantic memory footprint remains below that ceiling.

The algorithm uses the actual `BooleanStream::footprint()` contract rather than hard-coding an external estimate of B3 storage.

## Current reference numbers

For the existing capacity-16 attention references:

| Control | Declared attention components | B4 sparse-address program | Max B4 slots | B4 declared components | Slack |
| --- | ---: | ---: | ---: | ---: | ---: |
| B1 BinaryQk | 67,648 bits | 456 bits | 518 | 67,537 bits | 111 bits |
| B1 BinaryQk | 67,648 bits | 584-bit identity | 516 | 67,406 bits | 242 bits |
| B0 DenseQk | 132,160 bits | 456 bits | 1,016 | 132,028 bits | 132 bits |
| B0 DenseQk | 132,160 bits | 584-bit identity | 1,016 | 132,156 bits | 4 bits |

The v1 relational task family needs at most four stored facts. Under a task-sized attention history capacity of four:

| Control | Declared attention components | Address program | Max B4 slots | B4 declared components | Slack |
| --- | ---: | ---: | ---: | ---: | ---: |
| B1 BinaryQk | 16,960 bits | 456 bits | 126 | 16,773 bits | 187 bits |
| B1 BinaryQk | 16,960 bits | 584 bits | 126 | 16,901 bits | 59 bits |
| B0 DenseQk | 33,088 bits | 456 bits | 250 | 32,831 bits | 257 bits |
| B0 DenseQk | 33,088 bits | 584 bits | 250 | 32,959 bits | 129 bits |

These values expose the current representation disparity; they are **not** evidence that B4 is faster or more memory-efficient as a trained model.

## Why this matters

Earlier development comparisons deliberately used different storage representations. The new envelope makes that mismatch numerically explicit and gives future experiments a deterministic way to allocate B4 memory under the same declared component ceiling.

A future matched experiment must still add search/training cost, temporary memory, model parameters outside these components and physical measurements before any total-efficiency claim.

## Same-episode comparison report

A companion development report now executes the same relational episode through:

- B0 DenseQk attention;
- B1 BinaryQk attention;
- the current learned B4 relational address encoder over two-way Boolean memory.

For each episode, attention history capacity is set to exactly the number of declared facts rather than the historical capacity-16 control. B4 receives the maximal even slot count admitted by the same declared component ceiling after charging its ANF address-program bits.

The report keeps separate:

- correctness against the same evaluator-owned answer;
- attention pairwise scores and native attention work;
- B4 memory reads/writes and zero token-pair comparisons;
- B4 address predictions and ANF term evaluations;
- Development search work and fit error;
- declared component ceiling and unused slack.

Search work is reported but is not folded into the component ceiling. Consequently this remains a **matched declared-component** comparison only, never a total-budget comparison.

The distractor control is particularly important: adding irrelevant retained facts increases B0/B1 pairwise-score work while the B4 path still performs one addressed read per hop. This is a semantic work property of the current references, not a wall-clock performance result.

## Fail-closed behavior

If the address program alone exceeds the attention component ceiling, the envelope rejects the configuration rather than reporting zero slots or inventing a partial match.

Arithmetic overflow and invalid underlying attention/Boolean configurations also fail explicitly.

## Next use

After the learned-address control and relational B0/B1 controls are merged, this envelope can be used to construct matched-component relational campaigns. Those campaigns must report both the matched declared component ceiling and the remaining unmatched cost categories.