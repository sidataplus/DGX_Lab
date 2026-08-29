# Repository Map

```text
Cargo.toml                 workspace and pinned direct dependencies
crates/
  dgxlab-contracts/        cross-boundary IDs, time, terminal and worker protocol
  slurm-model/             jobs, nodes, partitions, QOS, accounting
  scheduler/               validation and deterministic allocation
  virtual-fs/              in-memory safe learner filesystem
  workloads/               synthetic AI workload plans and failures
  actors/                  declarative virtual-user/infrastructure actions
  sim-core/                authoritative world and event queue
  virtual-shell/           constrained command parser/renderer
  scenarios/               scenario contracts and built-in initializers
  grading/                 practical evidence/assertions
  assessment/              MCQ, multi-select, fill-blank, certification score
  persistence-codec/       integrity-protected session serialization
  report-renderer/         Markdown/HTML certificate output
  scenario-compiler/       YAML validation/compilation CLI
  sim-worker-wasm/         WebAssembly adapter
  web-ui/                  Leptos CSR shell
src-tauri/                 deliberately minimal desktop envelope
prototype/                 no-build interactive visual/behavioral prototype
scenario-src/              declarative scenarios
course-src/                course/lab source
question-src/              question bank and certification blueprint
schemas/                   JSON Schema contracts
scripts/                   validation, security, packaging, manifests
assets/mockups/             approved UI directions
docs/                      PRD, architecture, ADRs, runbooks, handoff
```
