# Diagnose Failure and Resume

**Scenario:** `dgx-degraded`
**Estimated time:** 45 minutes
**Prerequisite:** Lab 08, Arrays and Dependencies

> The failed job, OOM messages, checkpoints, GPU warning, and resumed workload
> are deterministic simulated artifacts. No model state is loaded on the host.

## Why this matters

Long jobs fail. Operational skill means identifying the failure class from
evidence, preserving completed work, changing the cause, and proving a
corrected replacement ran with explicit resume intent. “Try it again” is not a
diagnosis.

## Mental model

```text
terminal state → stderr + request evidence → failure class
                                                    ↓
checkpoints → newest readable candidate → corrected resubmission
                                                    ↓
                                           running/completed proof
```

Both host-memory and GPU-memory failures can end as an OOM state. The decisive
difference is in logs and resource context.

## Before you start

- Complete Lab 08 and select `dgx-degraded`.
- Do not clear the prepared job history, logs, or checkpoint directory.
- Keep Accounting, Job Details, virtual files, and logs visible.
- Treat a filename as a candidate, not proof that a checkpoint is usable.

## Objectives

By the end, you can:

1. identify the failed job's terminal state with `sacct`;
2. distinguish host OOM from GPU OOM using stderr;
3. select the newest valid checkpoint; and
4. resume with corrected resources.

## Worked example: evidence-first recovery

```console
sacct
ls logs
tail -n 30 logs/train-llm-<failed-id>.err
ls checkpoints
cat checkpoints/<candidate>.pt
```

Then use a verified checkpoint in a corrected submission:

```console
srun --job-name=train-resume --partition=gpu --gres=gpu:h200:4 --cpus-per-task=16 --mem=64G --time=02:00:00 python train.py --batch-size 64 --epochs 5 --resume-from-checkpoint checkpoints/<valid-checkpoint>.pt
```

Replace both placeholders or checkpoint names with evidence from your run.

## Guided practice

### 1. Find the terminal job

Run `sacct` and identify the learner training job with the OOM terminal state.
Record its ID, elapsed time, and exit code.

**What to notice:** the job is absent from `squeue` because it is terminal, but
its accounting record remains.

### 2. Classify the OOM

List `logs/` and tail the failed job's stderr. Compare these signatures:

- `host-memory oom_kill` points to the `--mem`/cgroup boundary;
- `torch.OutOfMemoryError` points to GPU HBM pressure.

Use the failed request as supporting evidence. Do not classify the failure from
the word “OOM” alone.

After the OOM is terminal, run `scontrol show job <failed-id>` as the diagnosis
record. Both this inspection and the stderr `tail` are required evidence; an
otherwise successful retry does not replace either diagnostic step.

### 3. Select a checkpoint

Run `ls checkpoints` and inspect candidates with `cat`. Choose the highest
readable completed epoch that precedes the failure.

**Boundary:** checkpoint files expose an epoch marker, not checksum or full
training-state validity metadata. Record the path and epoch, and label full
validity provisional.

### 4. Correct and resume

For the prepared host-memory failure, restore the known-safe 64 GiB request
while preserving `--partition=gpu`, 4 GPUs, 16 CPUs, and the two-hour limit.
Submit with `--job-name=train-resume` and `--resume-from-checkpoint <path>`.

Inspect `scontrol show job <resume-id>` and advance the clock. The transcript
proves which checkpoint argument you selected; `sacct` and new logs prove the
corrected synthetic job ran. No real model state is restored.

## Common mistakes and recovery

- **Calling host OOM a CUDA problem:** read the exact stderr signature.
- **Selecting the numerically largest file blindly:** verify it is readable and
  positioned before the failure; disclose the missing validity metadata.
- **Changing several variables at once:** correct the evidenced cause first.
- **Reusing the old job name:** use `train-resume` so evidence stays distinct.
- **Claiming recovery at submission:** require a completed state, the corrected
  64 GiB request, and transcript evidence of the selected checkpoint argument.

## Transfer challenge

If stderr instead showed `torch.OutOfMemoryError` while host memory stayed
comfortable, propose one minimal correction and one tradeoff it introduces.

## Check your understanding

1. Why can accounting state not distinguish the two OOM classes by itself?
2. What makes a checkpoint valid for continuation?
3. Which field would you change for a host-memory OOM?
4. What evidence closes the recovery loop?

## Reflection

- Which evidence changed your diagnosis?
- How much work did the checkpoint preserve?
- Did the correction address cause, symptom, or both?
