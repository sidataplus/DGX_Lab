# Multi-GPU Training

**Scenario:** `dgx-h200-8`
**Estimated time:** 45 minutes
**Prerequisite:** Lab 09, Diagnose Failure and Resume

> GPUs, ranks, communication, elapsed time, and throughput are simulated.
> `torchrun` launches a registered model, never a real distributed process.

## Why this matters

More GPUs change both scheduling cost and program structure. Each worker rank
needs one local device, synchronization adds overhead, and input work may not
scale. A correct launch therefore includes placement evidence and an honest
speedup calculation.

## Mental model

```text
four-GPU allocation → CUDA_VISIBLE_DEVICES=0,1,2,3
                         ↓
torchrun --nproc_per_node=4
                         ↓
rank 0→GPU 0, rank 1→GPU 1, rank 2→GPU 2, rank 3→GPU 3
                         ↓
throughput gain < ideal 4× when overhead exists
```

Ranks use job-local indices, not fixed physical chassis numbers.

## Before you start

- Complete Lab 09 and ensure no prior allocation is active.
- Start from the idle `dgx-h200-8` scenario.
- Keep GPU visibility and accounting open, plus a scratch table for rank
  placement and elapsed-time calculations.
- Use the same synthetic workload settings for a fair scaling comparison.

## Objectives

1. request four H200 GPUs;
2. launch a four-rank synthetic `torchrun` workload;
3. inspect rank-to-GPU placement; and
4. compare one- and four-GPU throughput without assuming 4× scaling.

## Worked example: four ranks inside four GPUs

```console
salloc --partition=gpu --gres=gpu:h200:4 --cpus-per-task=32 --mem=256G --time=00:30:00
echo $CUDA_VISIBLE_DEVICES
nvidia-smi -L
torchrun --nproc_per_node=4 train.py --batch-size 128 --epochs 3
exit
```

## Guided practice

### 1. Request four GPUs

Run the `salloc` command and record the allocation ID.

**What to notice:** Job Details shows four requested GPUs, while
`CUDA_VISIBLE_DEVICES` exposes four local indices.

### 2. Verify visibility before launch

Run:

```console
echo $CUDA_VISIBLE_DEVICES
nvidia-smi -L
```

Count four entries. If the counts differ, stop and diagnose the allocation
instead of launching.

### 3. Launch and inspect placement

Run the supported synthetic `torchrun` command. From
`--nproc_per_node=4` and visible devices `0,1,2,3`, construct a four-row
rank-placement table: each local rank maps to the matching local GPU index.

DGX Lab rejects a missing, nonnumeric, or mismatched process count. For
example, `--nproc_per_node=2` inside this four-GPU allocation returns an error
instead of pretending the launch contract is valid.

The runtime returns a generic acceptance message and stores no rank objects;
placement is inferred from the launch contract rather than emitted telemetry.

**Evidence:** the transcript requests four workers, the allocation exposes four
devices, and your table gives each rank one unique device. Duplicate or missing
mappings indicate an invalid launch model.

### 4. Compare scaling

Exit the interactive allocation, then submit comparable registered workloads:

```console
srun --job-name=scale-1 --partition=gpu --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:10:00 python train.py --batch-size 128 --epochs 3
srun --job-name=scale-4 --partition=gpu --gres=gpu:h200:4 --cpus-per-task=32 --mem=256G --time=00:10:00 torchrun --nproc_per_node=4 train.py --batch-size 128 --epochs 3
```

Advance the clock, use `sacct -j <job-id>` for each run, and calculate:

```text
speedup = single-GPU elapsed / four-GPU elapsed
efficiency = speedup / 4
```

Report the measured values. Do not write “4× faster” unless evidence is
actually 4×; synchronization, communication, input, and serial work reduce
scaling.

### 5. Close the evidence

Verify the released interactive allocation and both comparison jobs in `sacct`.

## Common mistakes and recovery

- **Setting four ranks with one visible GPU:** align rank count with allocation.
- **Treating local GPU numbers as physical IDs:** use job-local placement.
- **Changing batch size between comparisons:** hold the scientific workload
  constant or disclose the change.
- **Inferring utilization from GPU count:** compare completed work per unit
  time instead.
- **Keeping four GPUs for interactive thinking time:** exit promptly.

## Transfer challenge

Given a measured 2.8× speedup on four GPUs, compute scaling efficiency and name
two plausible sources of the gap from ideal scaling.

## Check your understanding

1. Why must rank count match visible device count?
2. What does a local rank-to-GPU map describe?
3. How do speedup and scaling efficiency differ?
4. Why is perfect scaling an empirical result, not a default assumption?

## Reflection

- Did the placement evidence match your prediction?
- Which overhead best explains the observed gap?
- Was four GPUs the most efficient choice for this workload?
