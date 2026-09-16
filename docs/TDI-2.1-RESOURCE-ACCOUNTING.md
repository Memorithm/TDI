# TDI-2.1 — Resource-accounting contract

Date frozen: 2026-09-16

## Principle

Runtime cost is measured, not assumed. The TDI-2.1 scientific definition of intuition does not contain a latency threshold.

## Static accounting

For every candidate and baseline record:

- parameter count;
- persistent experience bytes;
- temporary logical allocation bytes where measurable;
- representation dimension;
- number of stored patterns;
- number of matrix/vector operations implied by dimensions;
- number of retrieval/update passes.

## Dynamic accounting

On a pinned benchmark environment report:

- median and tail latency (`p50`, `p95`, `p99`);
- throughput;
- peak resident memory;
- CPU cycles/instructions where available;
- cache-miss counters only when supported and documented;
- accelerator/device time when applicable.

Warm-up, sample count, affinity and compiler flags must be recorded.

## SIMD and alignment claims

SIMD, FMA, cache-line alignment or `repr(align(...))` optimizations are not presumed beneficial. They require compiled-code and benchmark evidence on each claimed target. No claim that alignment forces a specific instruction such as `vmovaps` is admissible without inspecting the generated code.

## Complexity language

A fixed number of forward stages may be called *constant depth*. Total work must still be expressed as a function of representation dimensions and pattern-store size. For a dense projection over `k` patterns of dimension `d`, the leading retrieval work is at least proportional to `k*d` unless a different indexed/structured mechanism is actually implemented and measured.

## Reporting boundary

Engineering efficiency is reported alongside scientific quality but cannot rescue a candidate that fails the frozen scientific endpoint.
