# DGX Lab Architecture

## Decision summary

DGX Lab is a single-user, fully offline desktop application. Tauri 2 supplies only the native window and packaging envelope. The interface and simulation execute in WebAssembly. The product contains no real scheduler adapter and no abstraction intended to acquire one later.

```text
Tauri shell
  └── system WebView
       ├── Leptos CSR UI WASM
       ├── Simulation worker WASM
       │    ├── deterministic clock and event queue
       │    ├── virtual Slurm scheduler
       │    ├── virtual users and infrastructure actors
       │    ├── virtual shell and filesystem
       │    ├── synthetic workloads and telemetry
       │    └── grading and assessment
       └── browser-local persistence

No server · No network · No native shell · No SSH · No real Slurm
```

## Trust boundaries

| Boundary | Trusted for | Explicitly not trusted for |
|---|---|---|
| Tauri shell | window lifecycle and packaged assets | simulation policy, grading, arbitrary filesystem, process execution |
| Web UI | presenting state and gathering learner input | deciding scheduler truth |
| Simulation worker | canonical world state and deterministic transitions | host access |
| Imported content pack | declarative scenarios/questions after validation | code, scripts, plugins, native extensions |
| Learner-entered command | virtual parser input | host command execution |

## Workspace dependency direction

```text
web-ui ────────────────► worker protocol
sim-worker-wasm ───────► sim-core + virtual-shell
scenario-compiler ─────► scenarios
report-renderer ───────► assessment
persistence-codec ─────► world + shell + evidence

grad­ing ──────────────► world + shell + virtual files
sim-core ───────────────► scheduler + model + workloads + actors + VFS
scheduler ──────────────► slurm-model

sim-core ───────X──────► web APIs
sim-core ───────X──────► Tauri
all crates ─────X──────► Slurm commands, SSH, OS process APIs
```

## State authority

The `SimulationWorld` is authoritative for clock, cluster, jobs, accounting, event history, virtual files, deterministic RNG state, and future events. The UI consumes snapshots/deltas. It must never infer a job terminal state merely because a chart finished animating, a tradition web dashboards have practiced with regrettable enthusiasm.

## Determinism contract

Given identical:

1. application/model schema version;
2. scenario revision;
3. seed;
4. ordered learner commands and UI actions;
5. imported content digests;

the world digest, job identifiers, allocation choices, actor actions, faults, score, and evidence bundle must match.

## Persistence

The target persistence design is event log plus snapshots in IndexedDB. The starter implements a complete serializable world and integrity-protected `.dgxlab` codec in Rust; browser storage wiring remains a build milestone.

```text
snapshot N
   + later command/system events
   = exact restored world
```

## Desktop shell

The Tauri crate has no custom commands and no plugins. Its CSP limits `connect-src` to `'self'` so the Trunk bootstrap can load bundled WASM from the application origin, while no external origin is permitted. `worker-src` is restricted to bundled and blob workers. Browser file import/export is the first implementation route. Any future native file-dialog bridge requires an ADR, restricted scope, path/extension/size checks, and static-web fallback.

## Build targets

- Native Rust tests: fast domain/property/golden tests.
- `wasm32-unknown-unknown`: simulator worker and Leptos client.
- Tauri desktop: macOS Apple Silicon first; Windows x64 and Linux x86-64 for v1.
- Static web build: secondary target after desktop walking skeleton.
