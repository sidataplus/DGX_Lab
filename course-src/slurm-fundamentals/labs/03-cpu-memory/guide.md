# CPU and Memory Requests

**Scenario:** `dgx-h200-8`
**Estimated time:** 30 minutes
**Prerequisite:** Lab 02, Interactive CPU Allocation

> Workloads, memory pressure, logs, and failures are deterministic simulations.
> No Python program runs on your Mac.

## Why this matters

Memory is a hard job boundary. Request far too little and the cgroup can end the
job; request far too much and scarce capacity sits idle. The useful workflow is
request, observe, compare, and revise from evidence.

## Mental model

```text
request (--cpus-per-task, --mem)
        ↓
allocation boundary
        ↓
registered workload rule
        ├─ known-safe request → completes
        └─ undersized request → host-memory OOM plan
```

`--mem` is host RAM, not GPU HBM. For this exercise, a request below 48 GiB
deterministically selects the host-OOM teaching path; it is not a live memory
measurement.

## Before you start

- Complete Lab 02 and release its interactive allocation.
- Know how to copy a submitted job ID.
- Keep Job Details, logs, accounting, and clock controls available.
- The current UI does not render modeled memory samples; use the explicitly
  labeled course evidence card in Step 2 for the comparison objective.

## Objectives

By the end, you can:

1. submit a CPU preprocessing workload;
2. compare requested memory with observed simulated memory; and
3. deliberately trigger and diagnose a host-memory OOM.

## Worked example: one safe run, one constrained run

```console
srun --job-name=prep-ok --partition=gpu --cpus-per-task=8 --mem=64G --time=00:10:00 python preprocess.py --epochs 2
srun --job-name=prep-oom --partition=gpu --cpus-per-task=8 --mem=16G --time=00:10:00 python preprocess.py --epochs 2
```

These are non-interactive submissions. Record each returned ID, then use the
simulator clock to advance the registered workloads.

## Guided practice

### 1. Submit the preprocessing baseline

Run the `prep-ok` command. Inspect it with:

```console
scontrol show job <baseline-id>
```

**What to notice:** `NumCPUs=8` and `ReqMem=65536M` record the request. The job
has no GPU GRES request.

### 2. Compare request with observation

Advance the clock until the baseline finishes and confirm its accounting row:

```console
sacct -j <baseline-id>
```

The course supplies this separate comparison card:

```text
baseline request: 64 GiB
given observed peak: 48 GiB
headroom in this run: 16 GiB (25% of the request)
```

The peak is not produced by the current engine or exposed in the live UI. Treat
the card as a teaching input, not as a value you observed in `sacct`.

### 3. Trigger the host-memory OOM

Run the `prep-oom` command and advance the clock until it becomes terminal.
Then inspect:

```console
sacct -j <oom-id>
ls logs
tail -n 20 logs/prep-oom-<oom-id>.err
```

**Evidence:** `sacct` renders `OUTOFMEMORY`, while stderr contains a simulated
`host-memory oom_kill` message. That wording distinguishes host RAM pressure
from a CUDA allocation failure.

### 4. Recover

Restore the known-safe 64 GiB request, use a new job name, and resubmit. Confirm
the corrected job completes before considering the diagnosis resolved.

## Common mistakes and recovery

- **Confusing GiB with MiB:** `16G` is 16 GiB; `16` is only 16 MiB.
- **Calling every OOM a GPU OOM:** this job has no GPU; read stderr.
- **Inspecting only `squeue`:** terminal jobs leave the queue; use `sacct`.
- **Claiming `sacct` reported a peak:** it reports state and elapsed time here;
  use the labeled evidence card for the comparison.
- **Raising memory without evidence:** restore the known-safe request, then
  explain what observed-use instrumentation a production decision would need.

## Transfer challenge

Choose a revised memory request for `prep-ok` that is smaller than 64 GiB but
still above the observed peak. State the headroom you preserved and why.

## Check your understanding

1. Which request controls host memory?
2. What two observations distinguish host OOM from GPU OOM?
3. Why is a terminal state alone insufficient diagnosis?
4. What would make a right-sized request defensible?

## Reflection

- How different were requested and observed memory?
- Which log line was decisive?
- What change fixed the cause rather than merely hiding the symptom?
