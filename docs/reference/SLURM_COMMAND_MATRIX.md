# Supported Command Matrix

| Command | MVP | v1 | Notes |
|---|:---:|:---:|---|
| `sinfo` | ✓ | ✓ | partition/node summary |
| `squeue` | ✓ | ✓ | filters expanded in v1 |
| `sbatch` | ✓ | ✓ | parses VFS script and `#SBATCH` directives |
| `srun` | ✓ | ✓ | allocation and steps |
| `salloc` | ✓ | ✓ | interactive allocation |
| `scancel` | ✓ | ✓ | cancellation and accounting |
| `scontrol show job/node` | ✓ | ✓ | partition/reservation later |
| `sacct` | ✓ | ✓ | terminal history; format subset |
| `sstat` |  | ✓ | live synthetic metrics |
| `sprio` |  | ✓ | simplified priority components |
| job arrays |  | ✓ | `%A`, `%a`, task states |
| dependencies |  | ✓ | `afterok`, `afterany`, `afternotok` subset |
| reservations/QOS |  | ✓ | scenario-defined |
| `module` | ✓ | ✓ | simulated module registry |
| `singularity exec` | ✓ | ✓ | synthetic runtime, never a real image |
| `python`, `torchrun` | ✓ | ✓ | registered synthetic workloads only |
| `nvidia-smi` | ✓ | ✓ | inventory/visibility/telemetry subset |

Unknown commands and flags produce explicit unsupported messages. DGX Lab does not silently pass them to a host.
