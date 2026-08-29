# DGX Lab

**Interactive SLURM Training Simulator**

DGX Lab is a standalone, deterministic desktop simulation environment for learning SLURM and shared GPU-computing workflows. It presents a generic DGX-scale cluster, simulated concurrent users, synthetic AI workloads, failure scenarios, guided labs, and local certification. It is deliberately incapable of connecting to a real scheduler.

This development pack accompanies **DGX Lab PRD v1.0** and provides:

- a Tauri 2 + Leptos/Rust/WASM monorepo scaffold;
- a functional no-build browser prototype;
- a deterministic simulation-core walking skeleton;
- initial scheduler, virtual shell, virtual filesystem, workload, grading, and assessment crates;
- generic DGX-H200-8, contended, and degraded scenario sources;
- twelve guided course modules, each with a validated lab definition and learner guide;
- an initial certification blueprint and question bank;
- schemas, content validators, CI/release workflows, ADRs, authoring guides, and security tests;
- the approved PRD and four UI mockups.

## Current implementation status

| Area | Status in this pack |
|---|---|
| Product and architecture documentation | Substantial |
| Functional static prototype | Runnable |
| Pure Rust domain model | Walking skeleton |
| FIFO/resource scheduler | Walking skeleton |
| Virtual filesystem and shell | Walking skeleton |
| Synthetic workload planner | Walking skeleton |
| Practical grading and knowledge scoring | Walking skeleton |
| Scenario compiler and report renderer | Starter implementations |
| Leptos CSR UI | Initial component scaffold |
| WASM worker API | Initial adapter scaffold |
| Tauri 2 shell | Minimal, no real-system commands |
| Native/WASM compilation evidence | **Not produced in this environment** |
| Cargo lockfile and signed installers | Deferred until a Rust/Tauri build host is available |

The pack was generated in an environment without a Rust toolchain or dependency-network access. The source, schemas, TOML, JSON, YAML, content references, 241 requirement links, and security invariants were statically validated where possible. `docs/handoff/BUILD_GAPS.md` lists the remaining build-host verification work. Nothing is presented as compiled when it was not. A small miracle of restraint.

A detailed inventory and validation record is in `docs/handoff/PACK_BUILD_REPORT.md`.

## Fastest way to inspect the product

The prototype uses only local HTML, CSS, and JavaScript.

```bash
cd prototype
python3 serve.py
```

Then open the printed `http://127.0.0.1:...` address. It supports guided one-GPU allocation, a contended sandbox, basic job submission/inspection, and certification questions. It never executes entered commands.

You may also open `prototype/index.html` directly, although localhost gives more consistent browser behavior.

## Intended Rust/Tauri development flow

Install current stable Rust, the WASM target, Trunk, and Tauri CLI, then:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
cargo install tauri-cli --version 2.11.4 --locked

python3 scripts/validate_all.py
cargo fmt --all -- --check
cargo clippy --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop --all-targets --all-features -- -D warnings
cargo test --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop
cargo build -p sim-worker-wasm --target wasm32-unknown-unknown
cargo tauri dev
```

See `docs/runbooks/LOCAL_DEVELOPMENT.md` for OS prerequisites and the build order.

## Architectural boundary

```text
Tauri 2 desktop shell
        │
        ▼
Leptos CSR interface (WASM)
        │
        ▼
Rust simulation worker (WASM)
        │
        ├── virtual scheduler
        ├── virtual users
        ├── virtual shell/filesystem
        ├── synthetic workloads
        ├── grading and assessment
        └── deterministic event replay

NO SSH · NO SHELL · NO REAL SLURM · NO EXTERNAL NETWORK
```

The default cluster profile generalizes an eight-H200, 224-logical-CPU, cgroup-isolated Slurm environment into non-institutional names and paths. Production hostnames, IP addresses, credentials, and operational paths are intentionally absent.

## Important directories

```text
crates/                 Rust workspace crates
src-tauri/              Minimal Tauri 2 desktop shell
prototype/              No-build functional browser prototype
scenario-src/           Human-authored cluster/scenario YAML
course-src/             Course and lab Markdown/YAML
question-src/           Knowledge bank and certification blueprint
schemas/                JSON Schemas for portable content
scripts/                Validation and security tooling
docs/                   Architecture, ADRs, authoring, runbooks, handoff
assets/mockups/          Approved high-fidelity UI directions
```

## Primary next milestone

Produce the first verified build on macOS Apple Silicon, run native and WASM test parity, connect the Leptos UI to the WASM worker, and prove that the released application has no network, process, shell, SSH, or scheduler capability. See `docs/MILESTONES.md` and `docs/handoff/NEXT_ACTIONS.md`.

## Licensing

- Code: Apache License 2.0
- Built-in original course content: CC BY 4.0
- Product name and marks: see `TRADEMARKS.md`
- Third-party dependencies: to be generated from the resolved `Cargo.lock` before release
