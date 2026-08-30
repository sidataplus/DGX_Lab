# One GPU Allocation and Isolation

**Scenario:** `guided-one-gpu`
**Estimated time:** 30 minutes
**Prerequisite:** Lab 03, CPU and Memory Requests

> The H200 devices, UUIDs, allocation, and isolation are simulated. No host GPU
> is discovered or used.

## Why this matters

On a shared GPU node, “the node has eight GPUs” does not mean your job may use
all eight. Slurm allocates whole simulated GPUs and exposes only that job-local
device set. Verifying visibility prevents accidental cross-job assumptions.

## Mental model

```text
physical virtual GPU selected by scheduler
                  ↓
job allocation boundary
                  ↓ remapped locally
CUDA_VISIBLE_DEVICES=0 and nvidia-smi -L shows one device
```

The local index `0` can represent any physical simulated GPU. It proves
isolation, not a fixed chassis slot.

## Before you start

- Complete Lab 03 and ensure no allocation remains active.
- Open the GPU grid and Queue/Job Details views.
- Start from the `guided-one-gpu` scenario.
- Record the job ID before releasing the allocation.

## Objectives

By the end, you can:

1. inspect idle GPU capacity;
2. request one H200, 8 CPUs, and 64 GiB;
3. verify device visibility in two ways; and
4. release the allocation cleanly.

## Worked example: inspect, allocate, verify, release

```console
sinfo
scontrol show node dgx-h200-01
srun --partition=gpu --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:20:00 --pty bash
echo $SLURM_JOB_ID
echo $CUDA_VISIBLE_DEVICES
nvidia-smi -L
exit
```

## Guided practice

### 1. Establish the baseline

Run `sinfo` and `scontrol show node dgx-h200-01`.

**What to notice:** the node advertises eight H200-class GPUs and starts idle
in this guided scenario.

### 2. Request exactly one GPU

Run the `srun` command from the worked example.

**Evidence:** the grant creates a running learner job whose request shows
`NumCPUs=8`, `ReqMem=65536M`, and `TresPerNode=gres/gpu:1`.

### 3. Verify the boundary

Inside the allocation, run:

```console
echo $CUDA_VISIBLE_DEVICES
nvidia-smi -L
nvidia-smi
```

`CUDA_VISIBLE_DEVICES` should contain one local index. `nvidia-smi -L` should
list exactly one logical GPU, and the summary should say `Allocated GPUs: 1`.
Use all three signals together; a changed prompt alone is not isolation proof.

### 4. Release and verify

Run `exit`. Then use your recorded ID:

```console
sacct -j <job-id>
scontrol show node dgx-h200-01
```

Look for `COMPLETED` and the return of allocated node resources to baseline.

## Common mistakes and recovery

- **Requesting `--gres=gpu:8` because eight exist:** cancel or exit and request
  the one device the task needs.
- **Treating local GPU 0 as physical GPU 0:** local numbering is remapped.
- **Running `nvidia-smi` on the login prompt:** allocate a GPU first.
- **Forgetting CPU or host memory:** include all three resource dimensions.
- **Closing the view instead of exiting:** return to the allocation terminal
  and run `exit` so accounting records a clean completion.

## Transfer challenge

Without running it, predict `CUDA_VISIBLE_DEVICES` and `nvidia-smi -L` for a
two-GPU allocation. Then test the prediction only if the lab offers spare
simulated capacity, and release it immediately.

## Check your understanding

1. What does `CUDA_VISIBLE_DEVICES=0` prove here?
2. Which command counts visible devices?
3. Why inspect node capacity before requesting a GPU?
4. What evidence proves the resource was released?

## Reflection

- Which signal gave the strongest isolation evidence?
- How are physical and job-local device identities different?
- What is the smallest complete request for this workload?
