# Arrays and Dependencies

**Scenario:** `dgx-contended`
**Estimated time:** 40 minutes
**Prerequisite:** Lab 07, Queue Contention and Pending Reasons

> Array elements, dependencies, workloads, and time are simulated locally. No
> sweep process executes on the host.

## Why this matters

Arrays express repeated independent work without copying scripts. Dependencies
express ordering without a human watching the queue. Together they turn a
manual “run four trainings, then evaluate” routine into scheduler-visible
workflow state.

## Mental model

```text
array submit
  ├─ sweep[0] ─┐
  ├─ sweep[1] ─┤
  ├─ sweep[2] ─┼─ successful prerequisite → evaluation eligible
  └─ sweep[3] ─┘
                         before that: Reason=Dependency
```

DGX Lab represents array elements as individual job records. Its teaching
subset accepts `afterok:<numeric-job-id>` for one recorded prerequisite.

## Before you start

- Complete Lab 07 and start the `dgx-contended` scenario.
- Know how to edit, preflight, and resubmit `train.sbatch`.
- Keep a small record of every ID returned for the array.
- Use `squeue` and `sacct` rather than production-only array formatting flags.

## Objectives

By the end, you can:

1. submit four sweep elements;
2. inspect each element's state;
3. submit evaluation with an `afterok` dependency; and
4. observe dependency release after successful prerequisite work.

## Worked example: supported directive pattern

For the sweep, add or confirm:

```bash
#SBATCH --job-name=sweep
#SBATCH --array=0-3
#SBATCH --gres=gpu:h200:1
#SBATCH --cpus-per-task=8
#SBATCH --mem=64G
#SBATCH --output=logs/%x-%j.out
```

For evaluation, remove `--array`, change the name, and add:

```bash
#SBATCH --job-name=evaluate
#SBATCH --dependency=afterok:<prerequisite-id>
```

Replace the placeholder with a numeric element ID recorded by the simulator.

## Guided practice

### 1. Submit four elements

Save and review the sweep directives, then run:

```console
sbatch train.sbatch
squeue
```

**What to notice:** submission reports four tasks. Queue names include array
indices, and contention may leave elements pending for `Resources`.

### 2. Inspect element states

Record the four numeric job IDs from the queue. Use
`scontrol show job <element-id>` on at least one pending and one running
element as capacity becomes available.

**Evidence:** each element has its own state and allocation even though all came
from one script.

### 3. Add the evaluation dependency

In this teaching subset, point `afterok` at the final recorded sweep element.
Remove the array directive, name the job `evaluate`, preflight, and submit.
Then run `scontrol show job <evaluation-id>`.

Look for `Reason=Dependency`. Also verify all four sweep elements complete
before treating the campaign prerequisite as satisfied.

### 4. Observe release

Advance time in bounded increments. Use `squeue` while active and `sacct` after
elements finish. Reinspect the evaluation ID until it becomes eligible and
runs.

**What to notice:** the dependency job was submitted early but did not need a
person to launch it later.

## Common mistakes and recovery

- **Leaving `--array` on evaluation:** remove it and resubmit intentionally.
- **Typing the literal placeholder:** use a numeric job ID.
- **Assuming “submitted last” creates ordering:** only the dependency encodes
  the rule.
- **Confusing `Dependency` with `Resources`:** inspect the job record; both can
  occur at different times.
- **Assuming one successful element proves the sweep succeeded:** verify all
  four terminal rows.

## Transfer challenge

One sweep element fails while the recorded prerequisite has not succeeded.
Predict the evaluation state under `afterok` and explain why automatic launch
would be unsafe.

## Check your understanding

1. What does `--array=0-3` create?
2. Why does each element need its own state?
3. What condition releases `afterok`?
4. Why verify the entire sweep in this teaching subset?

## Reflection

- What manual coordination did the scheduler replace?
- Which evidence showed dependency rather than resource waiting?
- How would you summarize the campaign using job IDs?
