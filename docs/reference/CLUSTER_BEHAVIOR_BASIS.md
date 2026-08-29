# Generic Cluster Behavior Basis

The default `DGX-H200-8` teaching profile is generalized from commissioning evidence for a single-node, eight-H200 Slurm environment. The implementation retains only pedagogically relevant behavior:

- eight whole H200-class GPUs;
- 224 logical CPUs;
- approximately 1.8 TiB scheduler-visible memory;
- cgroup-style CPU, memory, and device confinement;
- one-GPU jobs see one remapped GPU;
- batch, accounting, failure states, containers, multi-GPU communication concepts;
- GPU/CPU/RAM/job monitoring concepts;
- shared parallel-filesystem and quota failure scenarios.

Institutional hostnames, IP addresses, credentials, filesystem names, serial numbers, firmware details, and operational access points are deliberately omitted. Product-facing names use `dgx-login-01`, `dgx-h200-01`, `/shared`, `/containers`, and generic partitions.

Source documents used during planning are listed in the PRD source context and are not bundled as product content.
