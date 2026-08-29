# ADR 0001: Tauri envelope around a WASM-first application

**Status:** Accepted

## Decision
Use Tauri 2 as the primary desktop package while retaining UI and simulation logic in WebAssembly. Tauri supplies a dedicated window and platform bundles, not an application server.

## Consequences
- Better standalone launch and native packaging.
- Per-platform build/signing effort.
- Static web target remains feasible.
- Native commands require explicit future review.
