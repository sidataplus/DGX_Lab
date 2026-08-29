# ADR 0002: No real scheduler backend

**Status:** Accepted

## Decision
DGX Lab will not define or ship a real Slurm/SSH/REST backend, feature flag, adapter, or plugin slot. The command surface terminates in the simulator.

## Rationale
This prevents environment mistakes from turning an educational command into a production action. It also keeps content portable and grading deterministic.
