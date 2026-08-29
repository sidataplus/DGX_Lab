# Contributing

DGX Lab is currently optimized for one primary developer assisted by coding agents. Contributions should preserve deterministic behavior and the no-real-infrastructure boundary.

## Change requirements

Every substantive change should include:

1. tests or golden evidence;
2. requirement IDs affected;
3. security/capability impact;
4. migration impact for sessions or packs;
5. documentation changes;
6. confirmation that the static prototype and intended Tauri behavior do not diverge silently.

Run the validation scripts before proposing changes. Avoid broad native plugins, arbitrary HTML in packs, implicit randomness, floating-point comparisons in grading, and generic backend abstractions that invite a future real-Slurm implementation.
