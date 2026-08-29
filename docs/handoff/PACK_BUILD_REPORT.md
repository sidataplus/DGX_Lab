# DGX Lab Dev Pack Build Report

**Pack version:** 0.1.0  
**Prepared:** 5 August 2026  
**Artifact class:** source, executable prototype, declarative content, and implementation documentation  
**Compiled desktop release:** not included

## Executive result

This pack advances DGX Lab from a PRD and UI concept into a coherent implementation starter. It contains a deterministic Rust simulation kernel, constrained virtual shell, virtual filesystem, scheduler and workload models, scenario and grading engines, assessment contracts, a WASM adapter, a Leptos CSR shell, a deliberately minimal Tauri 2 envelope, complete starter course content, certification material, security tooling, and a functional browser prototype.

The application source is intentionally unable to invoke a real scheduler, spawn a process, open SSH, or call an external network service. The native Tauri shell contains no custom commands or plugins. Learner commands are data interpreted by the simulator, not a particularly adventurous route to the host operating system.

## Pack inventory

| Item | Included |
|---|---:|
| Cargo workspace members | 17 |
| Rust crates under `crates/` | 16 |
| Rust source files | 18 |
| Rust source lines | approximately 4,400 |
| Generic simulation scenarios | 6 |
| Guided labs with YAML + Markdown | 12 |
| Certification questions | 36 |
| Question mix | 18 single-choice, 8 multi-select, 10 fill-in-the-blank |
| JSON Schemas | 5 |
| Requirement IDs traced | 241 |
| Architecture decision records | 10 |
| High-fidelity UI mockups | 4 |
| Functional no-build prototype | 1 |
| Deterministic `.dgxlabpack` course bundle | 1 |

## Implemented source surfaces

### Simulation and scheduler

- deterministic event queue and simulation clock;
- generic `DGX-H200-8` cluster profile;
- whole-GPU, CPU, and memory allocation;
- FIFO/resource scheduling foundation;
- typed pending and terminal states;
- job submission, cancellation, completion, timeout, and failure paths;
- virtual users and scenario actors;
- resource release and accounting records;
- state digests for replay/evidence.

### Virtual learner environment

- constrained command tokenizer and parser;
- `sinfo`, `squeue`, `sbatch`, `srun`, `salloc`, `scancel`, `scontrol`, and `sacct` starter behavior;
- simulated `nvidia-smi`, modules, Singularity, Python, and `torchrun` entry points;
- virtual filesystem and starter files;
- realistic synthetic training logs, checkpoints, utilization, and failure conditions;
- Slurm-compatible duration parsing for minutes, `minutes:seconds`, `hours:minutes:seconds`, and day-prefixed forms.

### Learning and certification

- 12-module SLURM Fundamentals course;
- guided objectives, steps, hints, and evidence checks;
- practical state-based grading;
- multiple-choice, multi-select, and fill-in-the-blank scoring;
- approved 60/25/15 certification weighting;
- 80% overall and 70% knowledge pass policy;
- critical competency gate and two-attempt policy;
- report and evidence-bundle starter code.

### Application and distribution

- Leptos CSR component scaffold;
- WASM simulator adapter;
- Tauri 2 shell with `core:default` only;
- no shell, process, HTTP, SSH, or filesystem plugin;
- bundled app icons for macOS, Windows, and Linux;
- original branding assets;
- deterministic development-pack and course-pack builders;
- CI blueprints for validation, security boundaries, Rust tests, WASM, and Tauri packaging.

## Executable prototype

The `prototype/` directory is a standalone local demonstration requiring only a browser and a static local server:

```bash
cd prototype
python3 serve.py
```

It demonstrates:

- the guided one-GPU allocation lab;
- a contended sandbox with simulated jobs;
- basic queue and job inspection;
- certification knowledge questions;
- local-only state and a no-network CSP.

The prototype does not execute entered commands and is deliberately separate from the uncompiled Rust/WASM application.

## Validation performed in the pack-creation environment

The following checks completed successfully:

```text
YAML and JSON Schema validation
cross-reference validation for scenarios, course modules, labs, and question banks
approved certification-weight and threshold validation
Tauri CSP and capability validation
Cargo/TOML parse and workspace-member validation
241-ID PRD traceability equality check
forbidden process/shell/SSH/network dependency and API scan
static Rust delimiter and unsafe-code sanity check
Node syntax check for prototype/app.js
Python bytecode compilation for helper scripts
content-pack build
SHA-256 manifest generation and verification
local HTTP smoke test of the prototype
```

The static Rust check is not a compiler substitute. It verifies balanced delimiters and the project-owned `unsafe_code` policy, not Rust type correctness or dependency APIs.

## Build limitations

The pack-generation environment did not provide `cargo`, `rustc`, Trunk, Tauri CLI, or dependency registry access. Consequently:

- no Rust crate was compiled;
- no native or WASM Rust test was executed;
- no `Cargo.lock` was resolved;
- no Tauri desktop installer was built;
- no code signing or notarization was attempted;
- exact dependency API compatibility remains a first-build-host gate.

These omissions are recorded in `BUILD_GAPS.md`; the repository does not dress a source scaffold in a fake moustache and call it a release binary.

## First build-host acceptance sequence

1. Install the pinned Rust/WASM/Tauri toolchain.
2. Resolve and commit `Cargo.lock`.
3. Run formatting, Clippy, unit, property, and native replay tests.
4. Compile `sim-worker-wasm` and `web-ui`.
5. Connect the Leptos UI to the worker messaging contract.
6. Run native-versus-WASM deterministic parity fixtures.
7. Launch `cargo tauri dev` and verify the restrictive CSP.
8. Prove by automated tests that the release has no real network, process, SSH, or Slurm path.
9. Build and smoke-test the macOS Apple Silicon bundle.
10. Expand the release matrix to Windows x86-64 and Linux x86-64.

## Recommended development priority

The next useful vertical slice is not more documentation or another glowing GPU tile. It is:

```text
Leptos terminal input
        ↓
WASM worker execute(command)
        ↓
pure Rust simulation state transition
        ↓
state delta rendered in terminal + cluster view
        ↓
IndexedDB event append
        ↓
reload and deterministic replay
```

Once this works for Lab 04, the same path can support sandbox, failures, and certification without creating three almost-identical application cores, an old and beloved software tradition best declined.
