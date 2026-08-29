# Diagnose Failure and Resume

**Scenario:** `dgx-degraded`  
**Estimated time:** 45 minutes

## Why this matters

This lab builds an operational mental model rather than rewarding memorized punctuation. The simulator evaluates the resulting scheduler state and evidence, not one sacred command string.

## Objectives

- Use `sacct` to identify the terminal state.
- Read logs and distinguish GPU OOM from host OOM.
- Select the newest valid checkpoint.
- Resubmit with corrected resources and resume.

## Reflection

- What resource or state changed?
- Which command supplied decisive evidence?
- What would be unsafe or wasteful on a shared production cluster?
