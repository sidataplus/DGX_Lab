# QOS, Reservations, and Efficiency

**Scenario:** `dgx-shared`
**Estimated time:** 40 minutes
**Prerequisite:** Lab 10, Multi-GPU Training

> QOS queue behavior is simulated live. Reservation and observed-use work use
> labeled evidence cards because those runtime fields are not implemented.

## Why this matters

A technically feasible job can still wait because shared systems apply policy.
Efficient users distinguish policy from capacity, then right-size requests from
observed evidence. “Use everything available” is not a scheduling strategy.

## Mental model

```text
job request
  ├─ QOS eligibility/limit ────────┐
  ├─ reservation access/window ────┼→ eligible
  └─ resources + queue order ──────┘
                                      ↓
request versus observed use → next-run right-sizing
```

QOS limits govern who/how much may run. Reservations set aside resources for
defined users or accounts during a defined window.

## Before you start

- Complete Lab 10 and select `dgx-shared`.
- Preserve the scenario-prepared jobs and policy state.
- Keep Queue, Job Details, policy explanation, and accounting open.
- Diagnose read-only first; do not cancel prepared jobs to make the reason
  disappear.

## Objectives

By the end, you can:

1. interpret a QOS per-user pending reason;
2. inspect a reservation-constrained job;
3. propose CPU and memory right-sizing from evidence; and
4. explain why maximum GPU count may reduce shared efficiency.

## Worked example: join command and UI evidence

```console
squeue
scontrol show job <job-id>
sacct -j <job-id>
```

Reservation and efficiency use this supplied evidence card:

```text
reservation case: JobState=Pending, Reason=Reservation, NodeList=(null)
requested: 16 CPUs, 512 GiB
observed peak: 7 CPUs-equivalent, 36 GiB
next-run proposal: smaller request + justified headroom
```

These are teaching inputs, not output from `scontrol` or `sacct`.

## Guided practice

### 1. Interpret the QOS limit

Find the prepared learner job with
`Reason=QOSMaxJobsPerUserLimit` using `squeue` and
`scontrol show job <job-id>`.

**What to notice:** the request may fit the node, but a per-user concurrent-job
limit blocks it. More memory or fewer CPUs does not directly remove that
policy reason.

### 2. Inspect the reservation constraint

Inspect the reservation line in the supplied evidence card. It describes an
accepted job that is outside its eligible reservation access or window.

**Boundary:** the current runtime has no reservation policy object or prepared
reservation job. Do not invent a command or claim this line came from the live
queue.

### 3. Right-size from observed use

Use the supplied 16-CPU/512-GiB request and 7-CPU-equivalent/36-GiB peak.
Propose smaller values that retain explicit headroom and explain the margin.

Label the result provisional because it comes from one supplied sample. Current
accounting does not report observed CPU or memory peaks.

### 4. Evaluate GPU efficiency

Compare useful throughput and elapsed time against GPU count. A run using eight
GPUs can be less efficient than one using four if speedup is small, inputs
starve devices, or communication dominates.

**What to notice:** availability answers “can I request it?”; efficiency asks
“does this additional resource materially improve completed work?”

## Common mistakes and recovery

- **Calling every pending reason `Resources`:** read the typed reason.
- **Trying to fix QOS with larger hardware requests:** wait for policy capacity
  or revise campaign concurrency.
- **Treating a reservation as a faster partition:** it is scoped access over a
  defined window.
- **Right-sizing to the exact observed peak:** preserve defensible headroom.
- **Claiming the evidence card is live output:** preserve its supplied label.
- **Using GPU count alone:** pair it with throughput or elapsed work.

## Transfer challenge

A four-GPU run is 2.8× faster than one GPU, while an eight-GPU run is 3.1×
faster. Recommend a GPU count for a shared queue and defend it quantitatively.

## Check your understanding

1. What does `QOSMaxJobsPerUserLimit` constrain?
2. How does `Reservation` differ from `Resources`?
3. What evidence supports a smaller memory request?
4. Why can using fewer GPUs improve system-wide throughput?

## Reflection

- Which wait was policy-driven and which was resource-driven?
- What headroom did you preserve in your right-sized request?
- What evidence would change your GPU-count recommendation?
