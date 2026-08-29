# Implementation Status

**Updated:** 2026-08-06 — macOS RC path through M0–M6 (unsigned)

## Milestone progress

| Milestone | Status |
|---|---|
| M0 — Verified build skeleton | **Done** |
| M1 — One-GPU vertical slice | **Done** |
| M2 — Batch and contention | **Done** |
| M3 — Failure and recovery | **Done** (Lab 09 path, diagnose panel) |
| M4 — Course v1 | **Done** (12-lab picker, arrays, themes/a11y basics) |
| M5 — Certification | **Done** (embedded bank, local scoring, trust wording) |
| M6 — Release hardening | **Partial** — unsigned `.app` + binary; DMG script failed; Win/Linux deferred |

## Release artifacts (this host)

| Artifact | Path |
|---|---|
| Release binary | `target/release/dgx-lab` |
| App bundle | `target/release/bundle/macos/DGX Lab.app` |
| Checksums | `temp/DGX-Lab_0.1.0_aarch64.checksums.txt` |

**Note:** Gatekeeper will warn on unsigned apps. Open via right-click → Open if needed. DMG bundling failed in this environment; the `.app` is the primary package.

## Product capabilities (v0.1 / PRD v1.0 macOS RC)

- Tauri 2 + Leptos CSR + pure `SimSession` authority
- Learn / Sandbox / Certification modes
- 12 guided labs with scenario mapping
- Interactive GPU lab, batch `sbatch`, VFS editor, contended pending reasons
- Job arrays (`#SBATCH --array=…`), dependencies (`afterok`)
- Failure/diagnose panel for OOM/timeout-style terminal states
- Local autosave (localStorage + best-effort IndexedDB)
- Offline certification knowledge scoring + practical %
- Independent-product disclaimer in UI footer

## Residual / deferred

- Dedicated Web Worker thread (same pure session API)
- Full event-sourced IndexedDB cadence/migrations
- Windows/Linux installers; macOS signing/notarization
- DMG packaging fix; SBOM/cargo-deny release gate
- Full multi-node / multifactor priority (P2)
- Thai localization (post-v1)

## Security evidence

See `docs/handoff/SECURITY_EVIDENCE_M0.md` (still valid: no shell/SSH/process plugins; capability `core:default` only).
