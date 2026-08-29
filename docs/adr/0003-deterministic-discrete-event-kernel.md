# ADR 0003: Deterministic discrete-event kernel

**Status:** Accepted

## Decision
All actors, workloads, scheduler transitions, faults, and clock advancement use one stable event queue ordered by simulated time and sequence. Randomness uses a serialized deterministic generator.

## Consequences
Exact replay and grading are possible. Real scheduler nondeterminism is represented only when explicitly modeled through seeded scenarios.
