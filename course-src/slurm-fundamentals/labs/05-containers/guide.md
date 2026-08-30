# Reproducible Container Workloads

**Scenario:** `dgx-h200-8`
**Estimated time:** 35 minutes
**Prerequisite:** Lab 04, One GPU Allocation and Isolation

> Singularity, the image, CUDA integration, and workload are simulated. DGX Lab
> never opens a real SIF or starts a container process.

## Why this matters

A container makes the software environment repeatable, but it does not grant
resources. Slurm still owns the allocation boundary, and `--nv` records the
runtime's GPU integration intent without widening that boundary. Separating
those responsibilities makes container failures much easier to diagnose.

## Mental model

```text
Slurm allocation: CPU + host memory + one GPU
        ↓
Singularity runtime: selected image + --nv
        ↓
registered workload: sees only job-local GPU visibility
```

The module selects a simulated runtime. The image path selects a registered
artifact. Neither replaces a scheduler request.

## Before you start

- Complete Lab 04 and release its allocation.
- Know the valid image path: `/containers/pytorch-lab.sif`.
- Keep the virtual filesystem, logs, and Job Details views open.
- Use only bundled virtual paths; host paths are intentionally inaccessible.

## Objectives

By the end, you can:

1. load the simulated Singularity module;
2. launch the PyTorch image with `--nv`;
3. prove the container workload sees one allocated GPU; and
4. diagnose a deliberately missing image.

## Worked example: allocation wraps runtime

```console
salloc --partition=gpu --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:20:00
module avail
module load singularity/4.5.0
module list
singularity exec --nv /containers/pytorch-lab.sif python train.py --batch-size 64 --epochs 1
echo $CUDA_VISIBLE_DEVICES
nvidia-smi -L
exit
```

## Guided practice

### 1. Allocate before loading the workload

Request the one-GPU allocation shown above.

**What to notice:** Slurm grants the resource boundary before Singularity is
involved. The job owns one visible GPU, 8 CPUs, and 64 GiB of host memory.

### 2. Load and verify the runtime

Run `module avail`, load `singularity/4.5.0`, then run `module list`.

**Evidence:** the loaded-module list contains the exact runtime version. A
silent successful `module load` is normal.

### 3. Run the registered image

Run the `singularity exec --nv` command from the worked example.

DGX Lab should acknowledge a synthetic workload and state that no host process
ran. Verify the surrounding allocation with:

```console
echo $CUDA_VISIBLE_DEVICES
nvidia-smi -L
```

Exactly one local device should be visible. The current teaching subset does
not expose a second container-only device namespace, so this allocation-level
visibility is the supported isolation evidence.

### 4. Diagnose a missing image

In the Script Editor, make a temporary `train.sbatch` variant with 64 GiB
memory, set `#SBATCH --job-name=container-missing`, and use the command below.
Save it, review the directives, and submit it with `sbatch train.sbatch`:

```bash
singularity exec --nv /missing/pytorch-lab.sif \
  python train.py --batch-size 64 --epochs 1
```

Advance the simulator clock, then use `sacct -j <job-id>` and inspect that
job's stderr path under `logs/`:

```console
tail -n 20 logs/container-missing-<job-id>.err
```

Run this inspection only after `container-missing` reaches `FAILED`. The
current stderr is the generic
`simulated workload entry point failed` message, so combine it with the
submitted `/missing/` path to diagnose the image input. Restore
`/containers/pytorch-lab.sif` and confirm a clean resubmission.

## Common mistakes and recovery

- **Running the container without an allocation:** request resources first.
- **Omitting `--nv`:** add it to record GPU-runtime intent; isolation still
  comes from the allocation.
- **Assuming module load proves the image exists:** verify the image path in
  the virtual filesystem.
- **Using a host path or downloading an image:** use only bundled virtual
  artifacts; this lab is intentionally offline.
- **Stopping at `FAILED`:** read the job's stderr and inspect the submitted
  command before changing resources.

## Transfer challenge

Explain which layer you would inspect first for each symptom: no allocation,
missing image, or zero visible GPUs. Give one piece of evidence for each.

## Check your understanding

1. Which component grants the GPU?
2. What does `--nv` do in this simulator?
3. Why can a valid module coexist with an invalid image path?
4. What proves recovery from the missing-image failure?

## Reflection

- Which evidence belonged to Slurm, Singularity, and the workload?
- Why is a versioned image path useful for reproducibility?
- What would you preserve in a real batch log?
