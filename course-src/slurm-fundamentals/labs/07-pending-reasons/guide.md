# Queue Contention and Pending Reasons

**Scenario:** `dgx-contended`
**Estimated time:** 35 minutes
**Prerequisite:** Lab 06, From Interactive Command to Batch Job

> The competing users, queue order, resources, and clock are deterministic
> simulator state. Waiting here consumes no real GPU time.

## Why this matters

Pending is not a failure. It is a job state with a reason that tells you whether
to wait, revise a request, or fix eligibility. Reading that reason before
editing the job prevents wasteful resubmission loops.

## Mental model

```text
submitted
   ↓ eligible?
PENDING ── Reason=Resources → request fits, capacity is busy
   │
   └──── Reason=Priority  → eligible capacity/queue ordering favors other work
   ↓ resources or ordering changes
RUNNING
```

The displayed primary reason is evidence about the current scheduler decision,
not a promise about an exact start time.

## Before you start

- Complete Lab 06 and know how to submit `train.sbatch`.
- Select `dgx-contended` and inspect the GPU grid before changing anything.
- Keep plain `squeue` available so competing users remain visible.
- Know how to advance the simulator by a bounded increment.

## Objectives

By the end, you can:

1. submit a one-GPU job while all GPUs are occupied;
2. observe `PENDING` in `squeue`;
3. distinguish `Resources` from `Priority`; and
4. watch the same job transition to `RUNNING`.

## Worked example: diagnose before acting

```console
sinfo
squeue
sbatch train.sbatch
squeue -u learner
scontrol show job <job-id>
```

Record the ID returned by `sbatch`. Use the UI clock controls for time; there
is no terminal “sleep” or host process in this simulator.

## Guided practice

### 1. Confirm contention

Run `squeue` before submitting. Identify the simulated jobs owned by other
users and confirm the GPU grid has no free device.

**What to notice:** a user-filtered queue would hide the evidence that explains
the resource shortage.

### 2. Submit one GPU

Confirm the saved `train.sbatch` still requests one H200, then run
`sbatch train.sbatch`.
Run `squeue -u learner`.

**Evidence:** the learner job is accepted but shown as `PD`/`PENDING` rather
than rejected.

### 3. Read the reason

Run `scontrol show job <job-id>`.

Look for `JobState=Pending`, `Reason=Resources`, a null node list, and the
one-GPU request. `Resources` means the request is feasible but not free now.
`Priority` would instead mean queue ordering is the primary blocker. This
scenario demonstrates `Resources` live; `Priority` is the conceptual contrast,
not a second reachable state in this run.

### 4. Advance and re-check

Advance by one simulated minute, then rerun `squeue` and `scontrol`. Repeat in
bounded increments until the same job becomes `RUNNING`.

**What to notice:** do not resubmit. The original ID moves from pending to
running when a competing job releases GPUs.

## Common mistakes and recovery

- **Treating pending as failed:** inspect `Reason` before editing or cancelling.
- **Using only `squeue -u learner`:** run plain `squeue` to see contention.
- **Submitting duplicates while waiting:** cancel only unintended duplicates
  with `scancel <job-id>`.
- **Changing memory for `Reason=Resources` without evidence:** first determine
  which requested resource is unavailable.
- **Advancing a large interval blindly:** step time and observe the transition.

## Transfer challenge

Suppose the job showed `Reason=Priority` while one GPU appeared free. Explain
why “request fewer GPUs” would not directly address that evidence.

## Check your understanding

1. What is the difference between rejection and pending?
2. Which command exposes the primary pending reason?
3. Why can `Resources` resolve without a resubmission?
4. What evidence proves the original job eventually started?

## Reflection

- What observation ruled out an invalid request?
- How did the queue change before the learner job started?
- Which response to pending would have created unnecessary work?
