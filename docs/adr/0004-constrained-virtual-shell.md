# ADR 0004: Constrained virtual shell rather than PTY

**Status:** Accepted

## Decision
Parse a curriculum-oriented command subset into typed operations. Do not emulate or embed a host shell, PTY, Python interpreter, or container runtime.

## Consequences
Less shell fidelity but dramatically smaller attack surface and clearer educational semantics. Unsupported behavior is explicit.
