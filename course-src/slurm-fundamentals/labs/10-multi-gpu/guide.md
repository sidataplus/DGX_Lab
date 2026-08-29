# Multi-GPU Training

**Scenario:** `dgx-h200-8`  
**Estimated time:** 45 minutes

## Why this matters

This lab builds an operational mental model rather than rewarding memorized punctuation. The simulator evaluates the resulting scheduler state and evidence, not one sacred command string.

## Objectives

- Request four H200 GPUs.
- Launch a synthetic `torchrun` workload.
- Inspect rank-to-GPU placement.
- Compare single- and four-GPU throughput without claiming perfect scaling.

## Reflection

- What resource or state changed?
- Which command supplied decisive evidence?
- What would be unsafe or wasteful on a shared production cluster?
