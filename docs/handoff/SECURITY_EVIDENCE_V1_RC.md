# Security evidence — v1.0 macOS RC (unsigned)

**Date:** 2026-08-06  
**Host:** macOS Apple Silicon  

## Checks run

| Check | Result |
|---|---|
| `scripts/check_forbidden_apis.py` | Pass |
| `scripts/validate_all.py` | Pass (content/schema) |
| Core `cargo test` workspace (excl. wasm UI desktop) | Pass |
| Tauri capabilities | `core:default` only |
| Plugins | None in `src-tauri` |
| Runtime architecture | No process/SSH/HTTP/Slurm client |

## Packaging

- Built: `target/release/dgx-lab`, `target/release/bundle/macos/DGX Lab.app`
- Not signed / not notarized
- DMG step failed in bundler; not required for RC app use

## Explicit residual risks

- Unsigned binary triggers Gatekeeper prompts
- UI WASM hosts `SimSession` on main thread (not separate Worker yet)
- Certification is standalone local evidence only (by design)
