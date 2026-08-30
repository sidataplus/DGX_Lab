# Capstone: Robust Training Campaign

**Scenario:** `dgx-contended`
**Estimated time:** 60 minutes
**Prerequisite:** Lab 11, QOS, Reservations, and Efficiency

> This closed, offline capstone keeps every cluster artifact inside DGX Lab.

## Why this matters

Real training is a campaign, not one command: it should be reproducible,
diagnosable, recoverable, and accountable. This lab combines those habits into
one evidence chain.

## Mental model

```text
prototype → batch → PENDING(reason) → RUNNING → failure
                                              ↓
summary ← accounting ← corrected resume ← logs + checkpoint
```

## Before you start

- Complete Labs 01–11, especially batch, pending, and checkpoint recovery.
- Select `dgx-contended` and preserve its prepared competing jobs.
- Open evidence views and record ID, state, reason, artifact, and correction.

## Objectives

1. convert an interactive prototype into a batch script;
2. submit under contention and diagnose the wait;
3. run a controlled simulated failure and recover from its checkpoint evidence; and
4. produce an accounting and efficiency summary.

## Worked example: campaign-ready script

```bash
#!/bin/bash
#SBATCH --job-name=train-h200
#SBATCH --partition=gpu
#SBATCH --gres=gpu:h200:1
#SBATCH --cpus-per-task=8
#SBATCH --mem=64G
#SBATCH --time=00:30:00
#SBATCH --output=logs/%x-%j.out
#SBATCH --error=logs/%x-%j.err

module load singularity/4.5.0
srun singularity exec --nv /containers/pytorch-lab.sif \
  python train.py --batch-size 64 --epochs 5
```

For recovery, keep the recipe but add
`--resume-from-checkpoint checkpoints/<valid-checkpoint>.pt`.

## Guided practice

### 1. Build the batch recipe

Translate the prototype into explicit directives and a containerized command.
Give stdout/stderr durable paths, save, and preflight the script. Submission
validates supported directives; resources, runtime, arguments, and logs remain
reviewable in one artifact. Keep the baseline job name `train-h200`: the
controlled failure and recovery use distinct names later.

### 2. Submit and diagnose contention

```console
squeue
sbatch train.sbatch
squeue -u learner
scontrol show job <job-id>
```

Record the initial `PENDING` state and exact reason. Use plain `squeue` to
connect that reason with competing work. Advance time in bounded increments
until the original `train-h200` ID runs and reaches `COMPLETED`. Do not inject
the controlled failure before this baseline closes.

### 3. Run and diagnose the controlled failure

After the first campaign job reaches a terminal state, launch a separate,
clearly named simulator job with an intentionally constrained 16 GiB host-memory
request:

```console
srun --job-name=capstone-failure --partition=gpu --gres=gpu:h200:1 --cpus-per-task=8 --mem=16G --time=00:30:00 python train.py --batch-size 64 --epochs 3
```

This is an in-simulator fault exercise; it never starts a host process. Advance
the simulator by one minute, then collect:

```console
sacct -j <failed-id>
ls logs
tail -n 30 logs/capstone-failure-<failed-id>.err
ls checkpoints
```

Classify the failure from state, stderr, and the recorded request. Select the
newest readable checkpoint that precedes the failure, while retaining the
guide's checkpoint-validity limitation. Reaching the named OOM state is a
critical capstone gate, not optional practice.

### 4. Correct and resume

Change only the evidenced cause, rename the job `capstone-recovery`, restore the
known-safe 64 GiB request, and add the selected checkpoint argument:

```console
srun --job-name=capstone-recovery --partition=gpu --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:30:00 python train.py --batch-size 64 --epochs 3 --resume-from-checkpoint checkpoints/<valid-checkpoint>.pt
```

Inspect its wait separately; recovery work can still contend for resources.

**Evidence:** the transcript identifies the checkpoint argument and corrected
64 GiB request; the resume job reaches `COMPLETED`. The simulator does not load
real model state.

### 5. Produce the summary

For each learner job, pair `sacct -j <job-id>` with
`scontrol show job <job-id>`. Calculate GPU-minutes as GPUs × elapsed minutes,
and CPU-minutes as CPUs × elapsed minutes.

Report IDs, transitions, failure/log/checkpoint evidence, correction, outcome,
and resource-time totals. Observed utilization is unavailable, so label
CPU/memory right-sizing as not assessable rather than inventing a conclusion.

## Common mistakes and recovery

- **Calling submission completion or duplicating it:** follow one ID to a
  terminal state.
- **Changing resources before logs:** classify first.
- **Picking the newest filename without validation:** inspect checkpoint
  metadata and chronology.
- **Reporting only the retry or inventing utilization:** preserve history and
  state the missing observed-use boundary.

## Transfer challenge

Design a follow-on evaluation job that should run only after the resumed
training succeeds. State the `afterok` relationship, required evidence, and
what should happen if resume fails.

## Check your understanding

1. What makes the batch script reproducible?
2. Which evidence explains a pending job?
3. What four facts justify a checkpoint resume?
4. What belongs in an efficiency summary beyond `COMPLETED`?

## Reflection

- Where did the campaign become robust rather than merely runnable?
- Which evidence prevented the wrong remediation?
- What one change would most improve the next campaign?
