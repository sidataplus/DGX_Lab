# Requirements Traceability Matrix

This matrix covers **241** unique PRD requirement IDs. “Source implementation” means code/content exists in the pack; it does **not** imply successful compilation or acceptance testing.

| ID | Priority/class | Status | Evidence / gap | Requirement |
|---|---:|---|---|---|
| `APP-001` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | DGX Lab shall launch as a standalone Tauri 2 desktop application without starting a localhost server. |
| `APP-002` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | The primary macOS build shall support Apple Silicon. |
| `APP-003` | P1 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | v1.0 shall provide qualified builds for macOS Apple Silicon, Windows x86-64, and Linux x86-64. |
| `APP-004` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | The application shall run without internet access after installation. |
| `APP-005` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | All runtime assets, fonts, icons, courses, and default profiles shall be bundled locally. |
| `APP-006` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | The application shall restore the most recent valid session after an unclean shutdown. |
| `APP-007` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | The application shall expose no automatic updater in v1. |
| `APP-008` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | The Tauri process shall expose only approved window, metadata, and import/export capabilities. |
| `APP-009` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | The application shall not include shell, process, HTTP, WebSocket, sidecar, unrestricted filesystem, or SQL Tauri plugins. |
| `APP-010` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | A release build shall fail CI when forbidden Tauri capabilities are present. |
| `APP-011` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | Native import/export commands shall validate file type, size, and user selection before reading or writing. |
| `APP-012` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | The application shall expose version, build ID, simulator compatibility version, and course-pack compatibility in About. |
| `APP-013` | P0 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | Application-owned runtime Rust crates shall forbid unsafe code unless an isolated approved exception exists. |
| `APP-014` | P1 | Scaffold + prototype | Minimal Tauri/Leptos crates and functional no-build prototype | The application shall support a future static web build without moving simulation into native-only services. |
| `UI-001` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The UI shall be implemented with Leptos client-side rendering compiled to WASM. |
| `UI-002` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The main interface shall contain learning, terminal/editor, cluster, and detail/timeline regions. |
| `UI-003` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | Users shall be able to resize or collapse major panels. |
| `UI-004` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | Panel layout shall persist locally per device. |
| `UI-005` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | Job IDs, node IDs, and virtual paths shall be clickable where they resolve to a detail view. |
| `UI-006` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | All authoritative state shall come from the simulation worker, not duplicated UI state machines. |
| `UI-007` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The UI shall detect missing/out-of-order worker deltas and request a full state snapshot. |
| `UI-008` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The cluster view shall show node, CPU, RAM, GPU, queue, and simulated-time state. |
| `UI-009` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The job detail view shall show requests, allocation, state, reason, steps, logs, telemetry, and accounting. |
| `UI-010` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The terminal shall be clearly labeled as simulated. |
| `UI-011` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The terminal shall support history, completion, clickable references, and transcript view. |
| `UI-012` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The application shall provide an integrated virtual text editor for batch scripts. |
| `UI-013` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The editor shall never open or modify host files directly. |
| `UI-014` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The application shall provide calm light and dark themes. |
| `UI-015` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The application shall honor reduced-motion preference. |
| `UI-016` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | Animations shall not be required to understand state. |
| `UI-017` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The home screen shall prioritize resume, next competency, readiness, and recovery actions. |
| `UI-018` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | Scenario Control shall be visibly distinct from learner mode. |
| `UI-019` | P0 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | Entering Scenario Control during certification shall invalidate the attempt. |
| `UI-020` | P1 | Prototype/design | Four mockups and functional no-build UI; Leptos production wiring pending | The UI shall remain usable at 125%, 150%, and 200% text scaling. |
| `SIM-001` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | The simulation shall use a deterministic discrete-event model. |
| `SIM-002` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | Equal compatibility version, profile, scenario, seed, and learner event sequence shall reproduce equal logical outcomes. |
| `SIM-003` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | The simulator shall use an explicitly versioned pseudo-random generator. |
| `SIM-004` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | Simulation decisions shall not depend on system time, operating-system randomness, or unordered map iteration. |
| `SIM-005` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | The simulator shall execute in a dedicated Web Worker in production. |
| `SIM-006` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | The pure simulation core shall compile natively for tests and benchmarks. |
| `SIM-007` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | The simulation shall support pause, single-event step, real time, ×10, and ×60. |
| `SIM-008` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | Practice mode shall support advance-to-next-relevant-event. |
| `SIM-009` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | Certification scenarios shall be able to restrict clock controls. |
| `SIM-010` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | The worker shall process bounded batches and yield to the message loop. |
| `SIM-011` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | UI metric deltas may be coalesced without removing logical events from replay. |
| `SIM-012` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | Resource and scoring arithmetic shall use deterministic integer/fixed-point representations where practical. |
| `SIM-013` | P1 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | The simulator shall support at least 100 actors and 1,000 jobs in one scenario. |
| `SIM-014` | P1 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | The simulator shall replay at least 10,000 events deterministically. |
| `SIM-015` | P0 | Partial source implementation | `sim-core`, `sim-worker-wasm`; production worker batching/UI integration unverified | The simulator shall provide state digests for snapshots and finalized assessments. |
| `SCH-001` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The default profile shall contain one generic login node and one eight-GPU H200-class compute node. |
| `SCH-002` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The default compute node shall expose 224 logical CPUs and approximately 1.86 TB allocatable memory. |
| `SCH-003` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The default profile shall use generic names and paths, with no institutional identifiers. |
| `SCH-004` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall model jobs, allocations, job steps, nodes, partitions, users, accounts, and resources. |
| `SCH-005` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall model whole-GPU GRES allocation. |
| `SCH-006` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall track physical virtual GPU allocation and job-local visibility mapping. |
| `SCH-007` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall model consumable CPU and memory. |
| `SCH-008` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall reject unsatisfiable requests according to profile policy. |
| `SCH-009` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall support `PENDING`, `CONFIGURING`, `RUNNING`, `COMPLETING`, `COMPLETED`, `FAILED`, `CANCELLED`, `TIMEOUT`, `OUT_OF_MEMORY`, `NODE_FAIL`, and `PREEMPTED`. |
| `SCH-010` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | Pending jobs shall retain a typed reason code. |
| `SCH-011` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The P0 scheduler shall implement deterministic FIFO/resource behavior with explicit overrides. |
| `SCH-012` | P1 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | P1 shall add simplified multifactor priority and fair-share. |
| `SCH-013` | P1 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall support job arrays and task concurrency limits. |
| `SCH-014` | P1 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall support core dependency types used by the curriculum. |
| `SCH-015` | P1 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall support QOS limits in advanced profiles. |
| `SCH-016` | P1 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall support time-bounded reservations in advanced profiles. |
| `SCH-017` | P1 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall support node drain, draining, down, and resume behavior. |
| `SCH-018` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall distinguish submission rejection from accepted pending jobs. |
| `SCH-019` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | Resource release shall occur deterministically at job/step termination. |
| `SCH-020` | P0 | Partial source implementation | `slurm-model`, `scheduler`, generic profiles; advanced policy/steps pending | The scheduler shall expose request, allocation, usage, and accounting as separate concepts. |
| `CMD-001` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | Every learner command shall pass through a typed simulator parser. |
| `CMD-002` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | No learner command shall be forwarded to a host shell, interpreter, or native process. |
| `CMD-003` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | P0 shall implement `sinfo`, `squeue`, `sbatch`, `srun`, `salloc`, `scancel`, `scontrol show job`, `scontrol show node`, and `sacct`. |
| `CMD-004` | P1 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | P1 shall implement `sstat`, `sprio`, `squeue --start`, partition/reservation views, and read-only accounting views. |
| `CMD-005` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | `sbatch` shall parse supported `#SBATCH` directives from virtual scripts. |
| `CMD-006` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | The parser shall stop recognizing `#SBATCH` directives after the first non-comment/non-whitespace command. |
| `CMD-007` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | Unsupported commands and options shall fail explicitly with curriculum-safe guidance. |
| `CMD-008` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | The parser shall support quoting, environment variables, line continuation, selected redirection, and selected pipelines. |
| `CMD-009` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | P0 shall support curated shell/file commands required by lessons. |
| `CMD-010` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | `module`, `singularity`, `python`, `torchrun`, and `nvidia-smi` shall map only to registered simulation behavior. |
| `CMD-011` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | Command support shall be versioned and exposed in an in-app reference. |
| `CMD-012` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | Output for common command/flag combinations shall be golden-tested. |
| `CMD-013` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | Command output shall be labeled as DGX Lab behavior, not universal Slurm output. |
| `CMD-014` | P0 | Partial source implementation | `virtual-shell`; supported P0 subset and tests authored, compilation unverified | Learner mode shall not expose simulated administrator mutation commands. |
| `VFS-001` | P0 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | Every session shall have an isolated virtual filesystem. |
| `VFS-002` | P0 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | Virtual paths shall never map to host paths. |
| `VFS-003` | P0 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | The virtual root shall include generic home, shared, dataset, container, checkpoint, scratch, and temporary paths. |
| `VFS-004` | P0 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | The filesystem shall support directories and regular text files. |
| `VFS-005` | P0 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | Logs and checkpoint metadata shall appear as virtual artifacts. |
| `VFS-006` | P0 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | Basic ownership and permission errors shall be supported. |
| `VFS-007` | P1 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | P1 shall support quota and capacity scenarios. |
| `VFS-008` | P0 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | All virtual path resolution shall normalize `.` and `..` without permitting escape. |
| `VFS-009` | P0 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | Large simulated binary artifacts shall be represented by metadata rather than full payloads. |
| `VFS-010` | P0 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | The integrated editor shall save only to the virtual filesystem. |
| `VFS-011` | P1 | Source implementation | `virtual-fs`; in-memory VFS, normalization, quota, tests authored | Content blobs shall be deduplicated by hash where practical. |
| `WRK-001` | P0 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | Workloads shall be declarative synthetic models, not executable code. |
| `WRK-002` | P0 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | P0 shall include CPU preprocessing and single-GPU training workloads. |
| `WRK-003` | P1 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | P1 shall include parameter sweeps, checkpointed training, and multi-GPU workloads. |
| `WRK-004` | P0 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | Workloads shall produce deterministic logs and artifacts. |
| `WRK-005` | P0 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | Workloads shall model time-varying CPU, RAM, GPU, HBM, and I/O. |
| `WRK-006` | P1 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | Multi-GPU workloads shall model rank startup and communication phases. |
| `WRK-007` | P0 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | Workloads shall support profile/scenario parameterization. |
| `WRK-008` | P0 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | Workloads shall declare failure rules. |
| `WRK-009` | P0 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | The simulator shall distinguish GPU OOM, host-memory OOM, timeout, and script failure. |
| `WRK-010` | P0 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | Telemetry views shall derive from the same workload state used for logs and accounting. |
| `WRK-011` | P1 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | Simulated energy or cost estimates shall be labeled estimates. |
| `WRK-012` | P0 | Partial source implementation | `workloads`; deterministic logs, artifacts, telemetry, failure planning | Imported packs shall not define executable workload plugins. |
| `ACT-001` | P0 | Starter implementation | `actors`, scenario sources; adaptive/background policies pending | The simulator shall support scripted virtual users. |
| `ACT-002` | P1 | Starter implementation | `actors`, scenario sources; adaptive/background policies pending | P1 shall support policy-driven and background-load actors. |
| `ACT-003` | P1 | Starter implementation | `actors`, scenario sources; adaptive/background policies pending | The simulator shall support infrastructure actors for fault events. |
| `ACT-004` | P0 | Starter implementation | `actors`, scenario sources; adaptive/background policies pending | Actor actions shall use the same validation/scheduling path as learner actions unless explicitly marked administrator behavior. |
| `ACT-005` | P0 | Starter implementation | `actors`, scenario sources; adaptive/background policies pending | Actor IDs, names, accounts, and actions shall be deterministic from scenario data and seed. |
| `ACT-006` | P0 | Starter implementation | `actors`, scenario sources; adaptive/background policies pending | Hidden future actor actions shall not be visible in learner mode. |
| `ACT-007` | P1 | Starter implementation | `actors`, scenario sources; adaptive/background policies pending | Scenario Control shall expose actor scripts and future events outside assessment. |
| `ACT-008` | P0 | Starter implementation | `actors`, scenario sources; adaptive/background policies pending | Ordinary lessons shall support at least 12 visible concurrent users. |
| `ACT-009` | P1 | Starter implementation | `actors`, scenario sources; adaptive/background policies pending | The engine shall support at least 100 actors for stress scenarios. |
| `FLT-001` | P0 | Partial source implementation | `workloads`, `scenarios`; core OOM/timeout/script/node concepts | P0 scenarios shall include invalid request, GPU OOM, host OOM, timeout, cancellation, script error, missing input, and permission error. |
| `FLT-002` | P1 | Partial source implementation | `workloads`, `scenarios`; core OOM/timeout/script/node concepts | P1 scenarios shall include node drain/down, GPU fault, storage outage, quota exhaustion, checkpoint corruption, and container failure. |
| `FLT-003` | P0 | Partial source implementation | `workloads`, `scenarios`; core OOM/timeout/script/node concepts | Faults shall alter scheduler, workload, logs, telemetry, and accounting consistently. |
| `FLT-004` | P0 | Partial source implementation | `workloads`, `scenarios`; core OOM/timeout/script/node concepts | Fault recovery shall be scenario-defined and deterministic. |
| `FLT-005` | P0 | Partial source implementation | `workloads`, `scenarios`; core OOM/timeout/script/node concepts | Practical grading shall distinguish diagnosis from remediation. |
| `FLT-006` | P1 | Partial source implementation | `workloads`, `scenarios`; core OOM/timeout/script/node concepts | Scenarios shall allow a correct conclusion that the learner cannot directly remediate an infrastructure fault. |
| `FLT-007` | P0 | Partial source implementation | `workloads`, `scenarios`; core OOM/timeout/script/node concepts | Fault output shall be realistic but clearly simulated and independently authored. |
| `LRN-001` | P0 | Content implemented | Twelve lab YAML/Markdown sources and course map | DGX Lab shall provide guided lessons and free-play practice. |
| `LRN-002` | P0 | Content implemented | Twelve lab YAML/Markdown sources and course map | MVP shall ship with at least four complete guided labs. |
| `LRN-003` | P1 | Content implemented | Twelve lab YAML/Markdown sources and course map | v1.0 shall ship with twelve complete labs covering competencies C1–C12. |
| `LRN-004` | P0 | Content implemented | Twelve lab YAML/Markdown sources and course map | Lessons shall include concise concept cards and command references. |
| `LRN-005` | P0 | Content implemented | Twelve lab YAML/Markdown sources and course map | Hints shall be deterministic, progressive, and recorded. |
| `LRN-006` | P0 | Content implemented | Twelve lab YAML/Markdown sources and course map | Practice completion shall distinguish independent and assisted completion. |
| `LRN-007` | P0 | Content implemented | Twelve lab YAML/Markdown sources and course map | Learning objectives shall map to stable competency IDs. |
| `LRN-008` | P0 | Content implemented | Twelve lab YAML/Markdown sources and course map | Practical grading shall be state/evidence-based rather than exact command-string matching. |
| `LRN-009` | P0 | Content implemented | Twelve lab YAML/Markdown sources and course map | Equivalent valid solution paths shall receive credit when covered by grading rules. |
| `LRN-010` | P0 | Content implemented | Twelve lab YAML/Markdown sources and course map | A course shall declare prerequisites and completion policy. |
| `LRN-011` | P1 | Content implemented | Twelve lab YAML/Markdown sources and course map | The UI shall recommend remediation after failed evidence checks. |
| `LRN-012` | P0 | Content implemented | Twelve lab YAML/Markdown sources and course map | No online LLM shall be required for instruction, hints, or scoring. |
| `QST-001` | P0 | Source/content implementation | `assessment` and 36-question deterministic bank | The question engine shall support single-answer multiple choice. |
| `QST-002` | P1 | Source/content implementation | `assessment` and 36-question deterministic bank | The question engine shall support multi-select. |
| `QST-003` | P0 | Source/content implementation | `assessment` and 36-question deterministic bank | The question engine shall support fill-in-the-blank. |
| `QST-004` | P0 | Source/content implementation | `assessment` and 36-question deterministic bank | Fill-in-the-blank shall support accepted aliases, normalized whitespace, case policy, and numeric tolerance. |
| `QST-005` | P0 | Source/content implementation | `assessment` and 36-question deterministic bank | Questions shall map to competencies and difficulty bands. |
| `QST-006` | P0 | Source/content implementation | `assessment` and 36-question deterministic bank | Option order shall be randomizable deterministically. |
| `QST-007` | P1 | Source/content implementation | `assessment` and 36-question deterministic bank | Question selection shall follow a versioned assessment blueprint. |
| `QST-008` | P0 | Source/content implementation | `assessment` and 36-question deterministic bank | Explanations shall be shown according to practice/certification policy. |
| `QST-009` | P1 | Source/content implementation | `assessment` and 36-question deterministic bank | Multi-select partial-credit policy shall be explicit and bounded. |
| `QST-010` | P0 | Source/content implementation | `assessment` and 36-question deterministic bank | Question authoring validation shall detect missing correct answers and duplicate options. |
| `QST-011` | P0 | Source/content implementation | `assessment` and 36-question deterministic bank | Runtime answer matching shall not use an LLM. |
| `QST-012` | P1 | Source/content implementation | `assessment` and 36-question deterministic bank | Regex-like accepted patterns shall be restricted and tested for pathological behavior. |
| `CERT-001` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | v1.0 shall provide a certification workflow combining knowledge and practical assessment. |
| `CERT-002` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | Default weights shall be 60% practical, 25% multiple-choice/multi-select, and 15% fill-in-the-blank. |
| `CERT-003` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | Default pass policy shall require 80% overall and 70% knowledge score. |
| `CERT-004` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | Critical practical competencies shall be mandatory regardless of aggregate score. |
| `CERT-005` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | Certification attempts shall pin app, course, blueprint, scenario, question-bank, and seed revisions. |
| `CERT-006` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | Scenario Control shall be disabled or invalidate a certification attempt. |
| `CERT-007` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | Hints shall be disabled by default in certification; any permitted use shall mark the attempt assisted. |
| `CERT-008` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | The default certification session shall allow up to two attempts. |
| `CERT-009` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | The application shall generate a locally verifiable evidence digest. |
| `CERT-010` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | The application shall generate a certificate and detailed competency report. |
| `CERT-011` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | The certificate shall state its standalone/local trust level. |
| `CERT-012` | P2 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | The system shall support later instructor countersignature metadata without claiming institutional verification in v1. |
| `CERT-013` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | Finalized assessment evidence shall be immutable in local storage. |
| `CERT-014` | P1 | Partial source/content implementation | Scoring gates, blueprint, report renderer; full UI/evidence flow pending | A finalized assessment shall be replayable or rescored under its pinned compatible rules. |
| `PER-001` | P0 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | Sessions, progress, virtual files, and assessment evidence shall persist in IndexedDB. |
| `PER-002` | P0 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | The application shall autosave after learner commands and significant state transitions. |
| `PER-003` | P0 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | The application shall create periodic snapshots. |
| `PER-004` | P0 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | Restore shall load the latest valid snapshot and replay subsequent events. |
| `PER-005` | P1 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | Restore shall fall back to an earlier valid snapshot after corruption. |
| `PER-006` | P0 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | Reset-to-scenario-start shall be available in Practice mode. |
| `PER-007` | P1 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | Full rewind and branching shall be available in P1. |
| `PER-008` | P0 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | Sessions shall export to `.dgxlab`. |
| `PER-009` | P0 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | `.dgxlab` import shall validate size, paths, schemas, and hashes. |
| `PER-010` | P1 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | The app shall support at least the previous two major session schema versions through read or migration. |
| `PER-011` | P1 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | Migration shall preserve original evidence and never silently rescore a finalized attempt. |
| `PER-012` | P0 | Starter implementation | Integrity codec; IndexedDB event sourcing/migrations pending | Storage management shall show local usage and deletion controls. |
| `PACK-001` | P0 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | Built-in content shall use compiled, validated pack data. |
| `PACK-002` | P1 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | v1.0 shall import `.dgxlabpack` files. |
| `PACK-003` | P0 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | Imported packs shall contain data only and no executable code. |
| `PACK-004` | P0 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | Pack import shall validate magic, schema, compatibility, size, entry count, paths, hashes, and references. |
| `PACK-005` | P1 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | Official packs may use an embedded-public-key signature scheme. |
| `PACK-006` | P1 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | Invalid signatures shall not be treated as unsigned trusted packs. |
| `PACK-007` | P1 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | The UI shall display trust and compatibility state. |
| `PACK-008` | P1 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | Unsigned local packs may be imported after explicit warning. |
| `PACK-009` | P0 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | Course-pack source authoring shall remain external in v1. |
| `PACK-010` | P1 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | A scenario compiler/validator CLI shall be delivered in P1. |
| `PACK-011` | P1 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | Pack licenses and attribution shall be displayed. |
| `PACK-012` | P0 | Partial source implementation | Schemas, compiler CLI, deterministic `.dgxlabpack` builder | The pack format shall be versioned independently from the session format. |
| `RPT-001` | P0 | Starter implementation | Markdown/HTML renderer; production export integration pending | The app shall provide a command transcript and job timeline. |
| `RPT-002` | P0 | Starter implementation | Markdown/HTML renderer; production export integration pending | The app shall provide a competency matrix. |
| `RPT-003` | P0 | Starter implementation | Markdown/HTML renderer; production export integration pending | Reports shall distinguish practice, assessment, assisted, and independent evidence. |
| `RPT-004` | P0 | Starter implementation | Markdown/HTML renderer; production export integration pending | The app shall export a human-readable HTML or Markdown learning report. |
| `RPT-005` | P1 | Starter implementation | Markdown/HTML renderer; production export integration pending | v1.0 shall export a certificate as PDF or deterministic print-ready HTML. |
| `RPT-006` | P1 | Starter implementation | Markdown/HTML renderer; production export integration pending | The app shall export JSON evidence and CSV competency data. |
| `RPT-007` | P0 | Starter implementation | Markdown/HTML renderer; production export integration pending | Reports shall not include machine identifiers or host paths by default. |
| `RPT-008` | P1 | Starter implementation | Markdown/HTML renderer; production export integration pending | A report shall include app/course/scenario versions and evidence digest. |
| `A11Y-001` | P0 | Design requirement | UX spec present; implementation/audit pending | The application shall be usable through keyboard-only navigation. |
| `A11Y-002` | P0 | Design requirement | UX spec present; implementation/audit pending | Interactive controls shall have semantic labels and visible focus. |
| `A11Y-003` | P0 | Design requirement | UX spec present; implementation/audit pending | Visual cluster views shall have table/text alternatives. |
| `A11Y-004` | P0 | Design requirement | UX spec present; implementation/audit pending | The terminal shall provide a screen-reader-friendly transcript mode. |
| `A11Y-005` | P0 | Design requirement | UX spec present; implementation/audit pending | Reduced-motion mode shall disable nonessential animation. |
| `A11Y-006` | P0 | Design requirement | UX spec present; implementation/audit pending | State shall not be communicated by color alone. |
| `A11Y-007` | P1 | Design requirement | UX spec present; implementation/audit pending | Certification timing policy shall support declared accommodations. |
| `I18N-001` | P0 | Deferred | English v1; localization infrastructure/Thai P1 | All UI strings shall use localization keys. |
| `I18N-002` | P0 | Deferred | English v1; localization infrastructure/Thai P1 | UI, course, and simulated-output locale shall be separately represented. |
| `I18N-003` | P0 | Deferred | English v1; localization infrastructure/Thai P1 | English UI and course content shall ship in v1. |
| `I18N-004` | P1 | Deferred | English v1; localization infrastructure/Thai P1 | Thai UI/course pack shall be supported as P1 content. |
| `I18N-005` | P0 | Deferred | English v1; localization infrastructure/Thai P1 | Practical scoring shall remain locale-neutral. |
| `SEC-001` | P0 | Source controls implemented, runtime unverified | Minimal capabilities/CSP/scanners/runbooks; packaged runtime test pending | The runtime shall contain no code path that invokes real Slurm or SSH. |
| `SEC-002` | P0 | Source controls implemented, runtime unverified | Minimal capabilities/CSP/scanners/runbooks; packaged runtime test pending | The runtime shall contain no host process-spawn API in application-owned crates. |
| `SEC-003` | P0 | Source controls implemented, runtime unverified | Minimal capabilities/CSP/scanners/runbooks; packaged runtime test pending | The runtime shall contain no arbitrary HTTP/WebSocket client capability. |
| `SEC-004` | P0 | Source controls implemented, runtime unverified | Minimal capabilities/CSP/scanners/runbooks; packaged runtime test pending | The release CSP shall deny external runtime resources and connections. |
| `SEC-005` | P0 | Source controls implemented, runtime unverified | Minimal capabilities/CSP/scanners/runbooks; packaged runtime test pending | Complete built-in-course execution shall pass with network disabled. |
| `SEC-006` | P0 | Source controls implemented, runtime unverified | Minimal capabilities/CSP/scanners/runbooks; packaged runtime test pending | Course content shall not include arbitrary HTML or script. |
| `SEC-007` | P0 | Source controls implemented, runtime unverified | Minimal capabilities/CSP/scanners/runbooks; packaged runtime test pending | Imported archives shall enforce expansion, count, and path limits. |
| `SEC-008` | P0 | Source controls implemented, runtime unverified | Minimal capabilities/CSP/scanners/runbooks; packaged runtime test pending | CI shall scan for forbidden dependencies, capabilities, APIs, and external URLs. |
| `SEC-009` | P0 | Source controls implemented, runtime unverified | Minimal capabilities/CSP/scanners/runbooks; packaged runtime test pending | Developer/Scenario Control shall not enable host execution. |
| `SEC-010` | P0 | Source controls implemented, runtime unverified | Minimal capabilities/CSP/scanners/runbooks; packaged runtime test pending | No `RealSlurmBackend` or equivalent interface shall exist in the repository. |
| `PRIV-001` | P0 | Design/source controls | Offline/no telemetry boundary; runtime persistence/privacy verification pending | No telemetry shall be transmitted in v1. |
| `PRIV-002` | P0 | Design/source controls | Offline/no telemetry boundary; runtime persistence/privacy verification pending | No cloud account or progress synchronization shall be required. |
| `PRIV-003` | P0 | Design/source controls | Offline/no telemetry boundary; runtime persistence/privacy verification pending | Learner data shall remain local unless explicitly exported. |
| `PRIV-004` | P0 | Design/source controls | Offline/no telemetry boundary; runtime persistence/privacy verification pending | Users shall be able to delete local sessions and imported packs. |
| `PRIV-005` | P0 | Design/source controls | Offline/no telemetry boundary; runtime persistence/privacy verification pending | Reports shall minimize personal data. |
| `NFR-001` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | Cold start to usable Home screen shall be under 3 seconds on an M1-class Mac after installation, excluding first OS security verification. |
| `NFR-002` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | Resume of an ordinary session shall complete in under 1 second after IndexedDB open on target hardware. |
| `NFR-003` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | Replaying 10,000 events from a valid snapshot shall complete in under 2 seconds on target hardware. |
| `NFR-004` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | Terminal command acknowledgement shall appear within 50 ms for non-advancing commands and within 100 ms for normal simulated scheduling actions. |
| `NFR-005` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | UI frame interaction shall remain responsive while 100 actors and 1,000 jobs are simulated at ×60. |
| `NFR-006` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | The simulation worker shall not block the UI main thread for more than 50 ms due to simulation processing. |
| `NFR-007` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | Cross-platform deterministic golden scenarios shall produce identical canonical state/evidence digests. |
| `NFR-008` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | Runtime network operation shall not be required or attempted. |
| `NFR-009` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | An unclean application close shall lose no more than the most recent uncommitted UI-only change; committed commands shall be recoverable. |
| `NFR-010` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | Imported pack validation shall fail safely without partially activating content. |
| `NFR-011` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | A malformed learner command shall not crash the simulation worker. |
| `NFR-012` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | A worker crash shall preserve the last valid persisted state and provide a recovery path. |
| `NFR-013` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | All P0 features shall have automated tests and documented failure behavior. |
| `NFR-014` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | Application-owned domain crates shall have no dependency on Tauri or browser APIs. |
| `NFR-015` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | The public API surface among crates shall use typed versioned contracts. |
| `NFR-016` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | Release builds shall be reproducible to the degree practical and record compiler, dependency, and build metadata. |
| `NFR-017` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | No external font, script, stylesheet, image, or course asset shall be fetched at runtime. |
| `NFR-018` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | The application shall meet keyboard-first and screen-reader acceptance criteria for core learning and certification flows. |
| `NFR-019` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | Storage use for the built-in application, excluding platform WebView/runtime, should remain below 250 MB unless documented course assets justify more. |
| `NFR-020` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | An ordinary session with 10,000 events should remain below 50 MB before optional detailed report artifacts. |
| `NFR-021` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | The simulator shall expose clear compatibility errors rather than silently altering old scenario semantics. |
| `NFR-022` | NFR | Specified, unverified | Testing/performance/reliability targets documented; build evidence pending | The system shall be maintainable and testable by one primary developer; unnecessary services and native plugins are prohibited. |
