# Security evidence — M0 build host

**Date:** 2026-08-06  
**Host:** macOS Apple Silicon development machine  
**Scope:** First verified build skeleton (M0)

## Invariants checked

| Check | Result | Evidence |
|---|---|---|
| Forbidden Tauri capabilities / process APIs | Pass | `python3 scripts/check_forbidden_apis.py` |
| Content/schema validation | Pass | `python3 scripts/validate_all.py` |
| Application-owned crates forbid `unsafe` | Pass | `#![forbid(unsafe_code)]` on runtime crates |
| Tauri plugins | None | `src-tauri/Cargo.toml` depends only on `tauri` |
| Capability file | Minimal | `src-tauri/capabilities/main.json` → `core:default` only |
| CSP | Restrictive | `src-tauri/tauri.conf.json` `connect-src 'self'`; no external origins |
| Simulation authority | Pure Rust session | `sim-session` / `sim-worker-wasm` — no host shell, SSH, HTTP, process spawn |

## Explicit non-capabilities (by design)

- No `std::process::Command` in application runtime crates
- No shell / SSH / Slurm client / HTTP Tauri plugins
- No real scheduler backend trait
- Learner commands parse only into simulator operations (`virtual-shell`)

## Residual (later milestones)

- Automated offline network-denial smoke in CI (M6)
- SBOM / `cargo deny` release artifact (M6)
- Dedicated Web Worker thread (session currently runs in UI WASM module using the same pure `SimSession` as the worker crate)
- Signed/notarized installers (out of scope for unsigned macOS RC)

## Command log (representative)

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"
python3 scripts/validate_all.py
python3 scripts/check_forbidden_apis.py
cargo test --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop
cargo build -p sim-worker-wasm --target wasm32-unknown-unknown
```
