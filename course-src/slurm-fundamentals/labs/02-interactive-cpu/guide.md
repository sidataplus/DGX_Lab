# Interactive CPU Allocation

**Scenario:** `dgx-h200-8`  
**Estimated time:** 25 minutes

## Why this matters

This lab builds an operational mental model rather than rewarding memorized punctuation. The simulator evaluates the resulting scheduler state and evidence, not one sacred command string.

## Objectives

- Request an interactive allocation with 4 CPUs and 8 GiB memory.
- Inspect `SLURM_JOB_ID` and `SLURM_CPUS_PER_TASK`.
- Exit cleanly and verify the job appears in accounting.

## Reflection

- What resource or state changed?
- Which command supplied decisive evidence?
- What would be unsafe or wasteful on a shared production cluster?
