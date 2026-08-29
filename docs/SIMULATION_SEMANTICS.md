# Simulation Semantics

## Fidelity level

DGX Lab is **behaviorally faithful for the supported curriculum**, not a complete reimplementation of Slurm. It models resource requests, eligibility, pending reasons, allocation, steps, accounting, dependencies, limits, reservations, failures, and synthetic utilization. Unsupported flags fail with a clear simulator message.

## Scheduler P0

1. Validate partition, node feasibility, GPU type/count, CPU, memory, account/QOS, and time limit.
2. Insert an eligible job into a deterministic FIFO order.
3. Attempt allocation in stable node and GPU index order.
4. Record a pending reason when no allocation is possible.
5. Start the job and schedule workload log/artifact/terminal events.
6. Release resources on a terminal transition.
7. immediately retry pending jobs in stable order.

The policy intentionally starts simpler than Slurm multifactor priority. P1 adds priority components and fair-share under explicit scenario configuration.

## Pending-reason precedence

The starter uses this diagnostic order:

```text
invalid partition/account/QOS
→ dependency
→ policy/QOS limit
→ reservation eligibility
→ node/partition availability
→ resources
→ priority
```

Only one primary reason is displayed, while detailed job inspection may list contributing constraints.

## Jobs and steps

- `sbatch` creates an allocation and a `.batch` step.
- `salloc` creates an interactive allocation.
- `srun` without an allocation requests one or creates an interactive job according to arguments.
- `srun` within an allocation creates a job step constrained by that allocation.
- `exit` completes the active simulated interactive allocation.

The starter shell models the allocation path; complete nested-step semantics are a near-term milestone.

## Whole GPU isolation

A job requesting one GPU receives a physical simulated GPU index. Inside its allocation, the user-facing view is remapped to a local device list. `CUDA_VISIBLE_DEVICES` and `nvidia-smi -L` therefore expose only allocated GPUs. MIG, MPS, and GPU sharding are deferred.

## Synthetic workload lifecycle

```text
command + JobSpec
        ↓
WorkloadRequest
        ↓
WorkloadPlan
├── natural duration
├── terminal state / failure point
├── deterministic stdout/stderr lines
├── checkpoint artifacts
└── telemetry curve
```

No submitted code executes. Recognized tokens such as batch size and epochs parameterize a registered workload model.

## Failure distinctions

- GPU OOM: HBM demand exceeds workload/device model.
- Host OOM: simulated cgroup memory limit is exceeded.
- TIMEOUT: workload duration exceeds requested wall time.
- FAILED: script/entry point/input validation fails.
- NODE_FAIL: infrastructure event terminates the job.
- CANCELLED: learner or actor cancels the job.

## Time control

The UI supports paused, event-step, 1×, 10×, 60×, and jump-to-next-event modes. Advancing time processes every queued event in timestamp/sequence order. It never skips scheduler transitions simply to make a chart look smooth.

## Actor semantics

Actors are data, not callbacks or scripts. Supported families:

- scripted timeline;
- background load with deterministic policy;
- policy-driven resubmission behavior;
- infrastructure operations/fault injection.

Imported packs cannot provide code. Actor actions compile into the same typed event model used by the learner.
