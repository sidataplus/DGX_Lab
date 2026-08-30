# Cluster, Nodes, Partitions, Jobs, and Steps

**Scenario:** `dgx-h200-8`  
**Estimated time:** 25 minutes

> Everything in this lab is simulated locally. Commands inspect a virtual
> scheduler; they never contact a cluster or run a host process.

## Why this matters

Slurm becomes easier when you know which object you are looking at. A partition
answers “where may work run?”, a node owns resources, a job asks for resources,
an allocation reserves them, and a step does work inside that allocation.
That model lets you diagnose state instead of guessing command flags.

## Mental model

```text
partition (eligible nodes)
  └─ node (CPU + memory + GPUs)
       └─ job allocation (reserved slice)
            └─ job step (one command or task using that slice)
```

A job can exist while pending, before it has an allocation. A step cannot use
resources outside its job's allocation.

## Before you start

- No earlier lab is required.
- Open the Learn workspace and select this lab.
- Keep the Terminal and Cluster/Node views visible.
- Treat job IDs and timestamps as run-specific evidence.

## Objectives

By the end, you can:

1. identify the default partition and node state with `sinfo`;
2. locate CPU, memory, and GPU capacity for `dgx-h200-01`; and
3. explain how a job allocation differs from a job step.

## Worked example: read from broad to specific

Start with the partition summary, then inspect its node:

```console
sinfo
scontrol show node dgx-h200-01
```

Read `gpu*` as “`gpu` is the default partition.” In the node record,
`CPUs` and `RealMemory` are capacity, while `AllocCPUs` and `AllocMem` are
currently reserved. `Gres=gpu:h200:8` describes eight simulated H200 GPUs.

## Guided practice

### 1. Map the partition

Run `sinfo`. Find the row whose partition name ends in `*`.

**What to notice:** `AVAIL` answers whether the partition accepts work;
`STATE` summarizes its node; `NODELIST` names the member node.

### 2. Inspect the node

Run:

```console
scontrol show node dgx-h200-01
```

Compare total and allocated CPU/memory values. Locate the GPU type and count.

**Evidence:** your transcript should contain both inspection commands and the
node record. The starting profile has 224 CPUs, 1,857,528 MiB of memory, and
eight H200-class GPUs.

### 3. Explain allocation versus step

Complete this sentence in your own words:

> A job allocation reserves ________; a job step is ________.

A sound answer says that the allocation is the scheduler-approved resource
boundary, while a step is work launched inside that boundary. DGX Lab focuses
on this supported allocation path; it is not a complete Slurm implementation.

## Common mistakes and recovery

- **Treating a partition as a machine:** follow `NODELIST` to the node record.
- **Reading `AllocMem` as total memory:** compare it with `RealMemory`.
- **Assuming `idle` means “no queue”:** `idle` describes the node; `squeue`
  describes jobs.
- **Guessing a node name:** copy it from `sinfo`, then rerun `scontrol`.
- **Using real-site flags from memory:** type `help` and stay within the
  simulator's documented command subset.

## Transfer challenge

Run `squeue`, then predict how the node state would change after a small
allocation starts. Check that prediction in the next lab.

## Check your understanding

1. Which symbol marks the default partition?
2. Can a pending job already have a node allocation?
3. Which node fields distinguish capacity from current use?
4. Why can several steps share one allocation but not exceed it?

## Reflection

- Which command narrowed the question from cluster-wide to node-specific?
- What evidence would you collect before requesting a GPU?
- In one sentence, describe the path from partition to running command.
