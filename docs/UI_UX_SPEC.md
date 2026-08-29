# UI and UX Specification

## Product mental model

> Practice on a realistic virtual cluster, see why the scheduler behaves as it does, and collect evidence of competence without touching real infrastructure.

## Primary workspaces

1. **Learn**: guided objectives, progressive hints, terminal/editor, cluster visualization, evidence checklist.
2. **Sandbox**: free-play scripts, synthetic workload templates, virtual queue, timeline, resource charts.
3. **Diagnose**: job history, logs, failure cause, checkpoints, health state, resume workflow.
4. **Certification**: timed knowledge questions plus practical simulator tasks, local evidence and result.
5. **Library**: courses, scenarios, saved sessions, official/unsigned pack distinction.

## Desktop layout

```text
┌──────────────────────────────────────────────────────────┐
│ product · scenario · clock controls · offline status     │
├───────────────┬───────────────────────┬──────────────────┤
│ instructions  │ terminal / editor     │ cluster / queue  │
│ progress      │                       │ resources        │
│ competency    ├───────────────────────┼──────────────────┤
│ hints         │ job detail / timeline │ logs / charts    │
└───────────────┴───────────────────────┴──────────────────┘
```

## Interaction principles

- Every command result is visibly marked as simulated.
- Current user jobs are distinguishable from virtual actors without hiding shared contention.
- Pending reasons are clickable and explained in context.
- Resource tiles show allocation, owner, health, and local device remapping.
- Time acceleration never obscures important state transitions; the timeline records them.
- Dangerous-looking commands are rejected calmly, not executed and not dramatized.
- Hints reveal progressively and are separately recorded from correctness.

## Terminal

A constrained terminal provides history, completion, selected ANSI colors, copy, clickable job IDs/paths, and a visible supported-command reference. It is not a PTY and must not imitate unsupported shell behavior convincingly enough to mislead.

## Script editor

The editor supports plain text, line numbers, syntax highlighting for Bash/`#SBATCH`, templates, validation markers, and save into the VFS. P0 does not attempt to become a full IDE in a trench coat.

## Accessibility

- Entire core workflow is keyboard operable.
- Semantic headings/landmarks and labeled controls.
- Tables have textual alternatives to color/status icons.
- Focus indicator meets contrast requirements.
- Text scales to 200% without loss of functionality.
- Reduced-motion mode removes allocation animation.
- Screen-reader summary announces state deltas after commands.

## Visual system

The provided mockups establish a calm dark system with blue active states, green success, amber pending/warning, red failure, and purple simulation/certification context. Production must also offer a light theme and cannot rely on color alone.

See `assets/mockups/` and `docs/design/MOCKUP_INDEX.md`.
