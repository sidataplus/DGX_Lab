# Queue Contention and Pending Reasons

**Scenario:** `dgx-contended`  
**Estimated time:** 35 minutes

## Why this matters

This lab builds an operational mental model rather than rewarding memorized punctuation. The simulator evaluates the resulting scheduler state and evidence, not one sacred command string.

## Objectives

- Submit a one-GPU job into a fully occupied cluster.
- Use `squeue` to observe PENDING.
- Use `scontrol show job` to distinguish Resources from Priority.
- Advance the clock until the job starts.

## Reflection

- What resource or state changed?
- Which command supplied decisive evidence?
- What would be unsafe or wasteful on a shared production cluster?
