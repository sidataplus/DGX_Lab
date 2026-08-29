# Testing Strategy

## Test pyramid

| Layer | Examples |
|---|---|
| Unit | resource arithmetic, state transitions, parsing, answer normalization |
| Property | no over-allocation, path confinement, terminal-state monotonicity, determinism |
| Golden | supported `sinfo`, `squeue`, `sacct`, `scontrol` outputs |
| Scenario contract | every built-in scenario/lab/question validates and references existing content |
| Replay | same seed/transcript yields same digest; snapshot+replay equals uninterrupted run |
| Native/WASM parity | canonical command sequence produces equal normalized world/evidence |
| UI integration | command input, state delta rendering, navigation, grading flow |
| Tauri security | exact capabilities/CSP; no native commands/plugins/network |
| Persistence migration | previous two major session versions remain importable |
| Accessibility | keyboard navigation, labels, contrast, reduced motion |
| Release smoke | packaged app opens offline and completes a core lab |

## Required property tests

1. Resource allocations are never negative or above capacity.
2. A whole GPU belongs to at most one allocation.
3. Releasing twice does not increase capacity twice.
4. Jobs cannot leave a terminal state.
5. Event sequence is stable for equal inputs.
6. Different seeds affect only declared randomized dimensions.
7. Imported paths remain in virtual roots.
8. Unsupported commands cannot mutate the world.
9. Certification weights and scores remain bounded.
10. Content digests change on any material evidence change.

## Golden command contract

Golden outputs are versioned as **DGX Lab behavior**, not a promise to reproduce every Slurm release byte-for-byte. A change requires:

- reason documented;
- golden diff reviewed;
- affected course content/tests updated;
- compatibility note when learners may notice.

## Performance tests

Target on M1-class hardware:

- 100 actors and 1,000 jobs remain interactive;
- process 10,000 events under two seconds in optimized native tests;
- restore ordinary snapshot under one second;
- UI command-to-state response under 100 ms excluding intentional simulation time;
- no long worker task blocks main-thread input.

## Current limitation

This generated pack has not been compiled because the creation environment had no Rust toolchain or dependency access. Python schema/security checks and JavaScript syntax checks are included and executed. Native, WASM, and Tauri gates remain mandatory on the first build host.
