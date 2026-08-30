# DGX Lab

**Interactive SLURM Training Simulator**

DGX Lab is a standalone, deterministic desktop simulation environment for learning SLURM and shared GPU-computing workflows. It presents a generic DGX-scale cluster, simulated concurrent users, synthetic AI workloads, failure scenarios, guided labs, and local certification. It is deliberately incapable of connecting to a real scheduler.

## Learning experience

Every module follows a consistent **Learn → Practice → Assess** rhythm. Learners receive one recommended action at a time, work entirely inside the simulated terminal, and advance only when the simulator observes the required evidence.

![Module 1 guided learning workspace with a recommended command, lab path, practice terminal, and virtual cluster](assets/screenshots/01-guided-foundations.png)

*Module 1 begins with a concrete next action. The lab path distinguishes the current step from later work, while the virtual cluster connects each command to visible scheduler state.*

![Module 9 failure-recovery workspace showing an out-of-memory job, recommended accounting command, lab path, and GPU state](assets/screenshots/02-failure-recovery.png)

*Module 9 turns an out-of-memory failure into a diagnosis-and-recovery exercise. Job state, GPU state, and the required evidence stay visible together so learners can inspect the failure before choosing a remedy.*

![Mobile capstone view showing the Learn, Practice, and Assess journey with a recommended next action](assets/screenshots/03-mobile-capstone.png)

*The same journey reflows for narrow screens: controls remain touch-friendly, the recommended action stays prominent, and the capstone opens with an explicit evidence-backed objective.*

This repository accompanies **DGX Lab PRD v1.0** and provides:

- a Tauri 2 + Leptos/Rust/WASM application workspace;
- a responsive Learn, Practice, and Assess interface;
- a deterministic simulation core with constrained virtual scheduler, shell, and filesystem;
- state-backed practical grading and an offline knowledge assessment;
- generic DGX-H200-8, contended, and degraded scenario sources;
- twelve guided course modules, each with a validated lab definition and learner guide;
- a certification blueprint and question bank;
- schemas, content validators, CI/release workflows, ADRs, authoring guides, and security tests;
- the approved PRD, four design-direction mockups, and verified runtime screenshots.

## Current implementation status

| Area | Status in this pack |
|---|---|
| Product and architecture documentation | Substantial |
| Functional static prototype | Runnable reference implementation |
| Pure Rust domain model | Implemented and tested |
| FIFO/resource scheduler | Deterministic and tested |
| Virtual filesystem and shell | Constrained and tested |
| Synthetic workload planner | Implemented and tested |
| Practical grading and knowledge scoring | State-backed and tested |
| Scenario compiler and report renderer | Implemented |
| Leptos CSR UI | Responsive release build produced |
| WASM worker API | Native/WASM boundary verified |
| Tauri 2 shell | Minimal, no real-system commands |
| Native/WASM compilation evidence | Verified in the current workspace |
| Cargo lockfile | Resolved and committed |
| Signed installers | Deferred to a signing/notarization release lane |

The current release has native Rust test evidence, a verified WebAssembly build, strict linting, browser QA at desktop and phone widths, validated course content, 241 requirement links, and a checksum-verified course pack. The browser build remains offline and never executes learner commands on the host.

## Fastest way to inspect the product

Serve the checked-in release build locally:

```bash
cd crates/web-ui/dist
python3 -m http.server 1421 --bind 127.0.0.1
```

Then open `http://127.0.0.1:1421`. The release includes all twelve labs, free practice, visual cluster evidence, failure recovery, and the capstone-gated readiness assessment. It never executes entered commands.

The no-build implementation in `prototype/` remains available as an early reference surface.

## Intended Rust/Tauri development flow

Install current stable Rust, the WASM target, Trunk, and Tauri CLI, then:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
cargo install tauri-cli --version 2.11.4 --locked

python3 scripts/validate_all.py
cargo clippy --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop --all-targets -- -A clippy::manual_checked_ops -D warnings
cargo test --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop
cargo build -p sim-worker-wasm --target wasm32-unknown-unknown
cargo check -p web-ui --target wasm32-unknown-unknown
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
assets/screenshots/      Verified runtime screenshots used in this README
```

## Primary next milestone

Produce signed and notarized desktop artifacts, add automated visual-regression coverage, and run structured usability sessions with learners while preserving the no-network, no-host-shell boundary. See `docs/MILESTONES.md` and `docs/handoff/NEXT_ACTIONS.md`.

## Licensing

- Code: Apache License 2.0
- Built-in original course content: CC BY 4.0
- Product name and marks: see `TRADEMARKS.md`
- Third-party dependencies: to be generated from the resolved `Cargo.lock` before release
