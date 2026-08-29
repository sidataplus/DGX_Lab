# Domain Model

## Core aggregates

### SimulationWorld

Owns all state that can affect replay:

- simulated time and clock speed;
- cluster resources and policies;
- job and job-step records;
- future event queue;
- accounting history;
- actor state/actions;
- virtual filesystem;
- append-only world event log;
- deterministic RNG and ID counters.

### ClusterState

```text
ClusterState
├── partitions
├── nodes
│   ├── CPU capacity/allocation
│   ├── memory capacity/allocation
│   ├── GPU inventory/allocation/health
│   ├── node state and drain reason
│   └── running job IDs
├── QOS definitions
└── scheduling policy revision
```

### JobRecord

```text
submission
  ↓
PENDING(reason)
  ↓ allocation
RUNNING
  ├── COMPLETED
  ├── FAILED
  ├── CANCELLED
  ├── TIMEOUT
  ├── OUT_OF_MEMORY
  ├── NODE_FAIL
  └── PREEMPTED
```

A terminal accounting record preserves requested TRES, actual allocation, timestamps, state, and exit code. A future telemetry summary will add simulated peak CPU, RAM, GPU utilization, and HBM.

### Virtual filesystem

The VFS stores normalized absolute paths and bytes. It rejects traversal, enforces a simulated quota, and provides the only files visible to the constrained shell. Learner scripts never become host files.

### EvidenceLedger

Records practical evidence independent of screen layout:

- commands issued;
- job state transitions reached;
- files created/read;
- failure reasons identified;
- hints requested;
- practical assertions passed;
- assessment responses.

The grading engine evaluates state and evidence, not exact command strings unless the competency explicitly concerns syntax.

## Identifiers

- Job IDs are deterministic monotonic integers within a session.
- Session IDs and content IDs are strings at the boundary; production will use UUIDv7 or digest-derived IDs.
- Content revisions are immutable semantic versions.
- Portable artifacts carry SHA-256 digests.

## Invariants

1. Allocated resources never exceed node capacity.
2. A GPU is allocated to no more than one whole-GPU job in P0/P1 profiles.
3. Terminal jobs cannot transition back to running.
4. Released resources return to cluster availability exactly once.
5. A job step cannot outlive its allocation.
6. All visible job state derives from the world, not UI-local state.
7. File paths remain inside the virtual root.
8. Equal replay inputs yield equal world digests.
9. Certification weights sum to 100.
10. Standalone certificates never claim independent identity verification.
