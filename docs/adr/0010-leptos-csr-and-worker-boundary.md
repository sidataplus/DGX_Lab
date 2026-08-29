# ADR 0010: Leptos CSR UI and separate simulation worker

**Status:** Accepted

## Decision
Compile the UI and simulation adapter as separate WASM modules. The worker owns authoritative simulation state; the main thread renders snapshots/deltas and gathers input.
