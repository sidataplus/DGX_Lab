# Development Milestones

## M0 — Verified build skeleton

- install/pin Rust build environment;
- generate `Cargo.lock`;
- compile all native crates;
- resolve API/version drift;
- run fmt, clippy, unit tests;
- build Leptos WASM and Tauri window;
- record native/WASM deterministic parity;
- verify security boundary.

**Exit:** app opens; one command reaches the WASM worker and renders an authoritative snapshot.

## M1 — One-GPU vertical slice

- guided Lab 04 UI;
- terminal parser and interactive allocation;
- one-GPU device remapping;
- progress/evidence checks;
- IndexedDB autosave;
- session reset/export/import;
- macOS Apple Silicon development build.

**Exit:** complete the one-GPU lab, close/reopen, and restore exact state.

## M2 — Batch and contention

- script editor/VFS;
- `sbatch`, `.batch` step, logs;
- virtual actors and fully occupied cluster;
- pending reasons and time acceleration;
- sandbox workspace and charts.

**Exit:** submit under contention, diagnose pending reason, and observe deterministic start.

## M3 — Failure and recovery

- host/GPU OOM, timeout, script/missing input, node fault;
- checkpoint artifacts;
- resume workflow and failure evidence;
- job history/accounting views.

**Exit:** complete Lab 09 with exact replay.

## M4 — Course v1

- all twelve modules;
- arrays, dependencies, containers, multi-GPU, policy/efficiency;
- command reference and adaptive deterministic hints;
- accessibility and light theme.

**Exit:** full course content/functional acceptance.

## M5 — Certification

- question-bank selection/randomization;
- knowledge UI and practical exam flow;
- scoring gates and attempt policy;
- evidence bundle and HTML/Markdown outputs;
- local standalone trust labels.

**Exit:** pass/fail assessment is deterministic and replayable.

## M6 — Cross-platform v1.0

- Windows/Linux release builds;
- installers, signing/notarization;
- static web build;
- SBOM/license/security/release evidence;
- trademark/public release review.

**Exit:** v1.0 release candidates on macOS, Windows, and Linux.
