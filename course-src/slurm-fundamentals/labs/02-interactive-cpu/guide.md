# Interactive CPU Allocation

**Scenario:** `dgx-h200-8`
**Estimated time:** 25 minutes
**Prerequisite:** Lab 01, Cluster Mental Model

> This is an offline simulation. `salloc` changes only virtual scheduler state
> and the prompt represents a simulated compute node.

## Why this matters

Interactive allocations are useful for short investigation and prototyping.
The safe habit is to request a bounded resource envelope, verify what Slurm
granted, work inside it, and release it as soon as you are done.

## Mental model

```text
login shell --request--> pending job --grant--> interactive allocation
                                                ├─ environment describes grant
                                                └─ exit releases resources
```

The prompt moving to `dgx-h200-01` is a context change, not a real SSH login.

## Before you start

- Complete Lab 01 or be comfortable with partitions and allocations.
- Make sure no earlier interactive allocation is still active.
- Keep the Queue/Job Details view open.
- Plan to record the granted job ID before exiting.

## Objectives

By the end, you can:

1. request 4 CPUs and 8 GiB of memory interactively;
2. inspect `SLURM_JOB_ID` and `SLURM_CPUS_PER_TASK`; and
3. exit cleanly and verify the terminal job in accounting.

## Worked example: bounded allocation pattern

```console
salloc --partition=gpu --cpus-per-task=4 --mem=8G --time=00:15:00
echo $SLURM_JOB_ID
echo $SLURM_CPUS_PER_TASK
exit
sacct -j <job-id>
```

Replace `<job-id>` with the number printed by `salloc`. Angle brackets are
placeholders, not literal terminal text.

## Guided practice

### 1. Request the allocation

Run the `salloc` command above. Do not request a GPU in this CPU-only lab.

**What to notice:** the grant message contains a job ID, the prompt names the
compute node, and the queue/job view shows a running allocation with 4 CPUs and
8 GiB.

### 2. Inspect the allocation environment

Run:

```console
echo $SLURM_JOB_ID
echo $SLURM_CPUS_PER_TASK
env
```

The job ID should match the grant, and `SLURM_CPUS_PER_TASK` should report `4`.
Use the job record below as an independent cross-check rather than trusting the
environment value by itself.

### 3. Cross-check the request

Copy the numeric ID into:

```console
scontrol show job <job-id>
```

Look for `JobState=Running`, `NumCPUs=4`, and `ReqMem=8192M`.

### 4. Release and account

Run `exit` once. Back at the login prompt, run `sacct -j <job-id>`.

**Evidence:** the terminal transcript should show the grant, environment
inspection, release message, and a `COMPLETED` accounting row with exit code
`0:0`.

## Common mistakes and recovery

- **Typing `exit` before recording the ID:** find the newest learner row with
  `sacct`.
- **Requesting `--mem=8`:** bare values mean MiB here; use `8G`.
- **Adding `--gres` from habit:** omit it; this exercise is CPU-only.
- **Opening a second allocation:** exit the first one, then retry.
- **Using `sacct -j $SLURM_JOB_ID` after exit:** the variable is cleared on
  release; paste the recorded number.

## Transfer challenge

Repeat with 2 CPUs and 4 GiB for five minutes. Before running it, predict which
`scontrol` fields will change and which will stay the same.

## Check your understanding

1. What makes an interactive request “bounded”?
2. Which two pieces of evidence prove the CPU count?
3. Why is `exit` an important cluster operation, not just terminal tidiness?
4. Where do you look after the job disappears from `squeue`?

## Reflection

- Did the requested and granted resources agree?
- What signal proved that resources were released?
- When would a batch job be more appropriate than `salloc`?
