# From Interactive Command to Batch Job

**Scenario:** `dgx-h200-8`
**Estimated time:** 35 minutes
**Prerequisite:** Lab 05, Reproducible Container Workloads

> The editor, submission, logs, workload, and accounting are local simulated
> objects. Batch commands never reach your operating system.

## Why this matters

Interactive work is temporary; a batch script is a reviewable recipe. It keeps
resource requests, runtime setup, workload arguments, time limits, and log
locations together so another run can be explained and reproduced.

## Mental model

```text
train.sbatch
  ├─ #SBATCH directives → job request
  └─ command body       → registered workload
           ↓ preflight + save
           ↓ sbatch
queue → allocation → logs + terminal accounting
```

Submission captures the saved script. Later edits do not rewrite an existing
job.

## Before you start

- Complete Lab 05 and release any interactive allocation.
- Open `/home/learner/train.sbatch` in the Script Editor.
- Keep Terminal, Queue, virtual files, and clock controls visible.
- Create or retain the virtual `logs/` directory through the provided script.

## Objectives

By the end, you can:

1. define resources in `train.sbatch`;
2. validate and submit the script;
3. find separate output and error evidence; and
4. use accounting after the job completes.

## Worked example: a complete simulator script

```bash
#!/bin/bash
#SBATCH --job-name=train-h200
#SBATCH --partition=gpu
#SBATCH --gres=gpu:h200:1
#SBATCH --cpus-per-task=8
#SBATCH --mem=64G
#SBATCH --time=00:30:00
#SBATCH --output=logs/%x-%j.out

module load singularity/4.5.0
srun singularity exec --nv /containers/pytorch-lab.sif \
  python train.py --batch-size 64 --epochs 5
```

`%x` expands to the job name and `%j` to the job ID in the virtual output
path.

## Guided practice

### 1. Inspect and edit

Compare the open script with the worked example. Confirm each resource has a
reason: one GPU, 8 CPUs, 64 GiB, and a 30-minute ceiling.

Make one deliberate learner edit (for example, add a short comment explaining
the request), then save `train.sbatch` in the Script Editor. The bundled seed
file is a starting point, not evidence that you performed the preflight.

**What to notice:** directives begin with `#SBATCH` and appear before the
workload body. The body loads the runtime and launches the registered image.

### 2. Preflight, then submit

Review the saved file against the worked example. Submission performs the
supported directive and policy validation; if it returns an error, correct the
saved script before retrying. Then run:

```console
sbatch train.sbatch
squeue -u learner
```

Record the submitted ID. A successful submission message is not completion.

### 3. Observe progress and logs

Use the simulator clock controls to advance, checking `squeue -u learner`
between increments. Only after the job reaches `COMPLETED`, inspect:

```console
ls logs
tail -n 20 logs/train-h200-<job-id>.out
```

If `ls logs` lists a matching `.err` file, tail it too. The simulator does not
create an unused stream file, so an absent stderr artifact is normal.

### 4. Close the evidence loop

Run:

```console
sacct -j <job-id>
```

**Evidence:** match the same ID across submission, log filenames, and the
`COMPLETED` accounting row with `0:0` exit code.

## Common mistakes and recovery

- **Editing without saving:** preflight the saved content before `sbatch`.
- **Typing `#SBATCH` lines into the terminal:** put them in the script editor.
- **Using literal `<job-id>`:** replace it with the submitted number.
- **Looking only in `squeue` after completion:** switch to `sacct`.
- **Claiming success from stdout alone:** require terminal accounting too.

## Transfer challenge

Change only `--job-name` and `--epochs`, resubmit, and predict both output
filename and elapsed behavior before advancing the clock.

## Check your understanding

1. Which lines become the job request?
2. When are `%x` and `%j` expanded?
3. Why validate before submitting?
4. Which evidence proves the batch job ended successfully?

## Reflection

- What did the batch script preserve that an interactive transcript did not?
- Could another learner identify the exact resource envelope?
- Which three artifacts share the same job ID?
