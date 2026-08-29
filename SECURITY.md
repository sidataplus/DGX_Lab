# Security policy

## Runtime invariant

DGX Lab must remain structurally incapable of reaching a real scheduler or executing learner-entered commands.

The released application shall contain:

- no SSH client or SSH library;
- no Slurm client binaries or real scheduler adapter;
- no process-spawning or shell plugin;
- no unrestricted filesystem capability;
- no HTTP, WebSocket, upload, or updater plugin in v1;
- no remote content or CDN dependency;
- no executable imported scenario/plugin format.

Learner commands are parsed into typed simulator operations. They are never forwarded to an operating-system shell.

## Reporting a vulnerability

For a private development repository, report security issues directly to the project owner rather than opening a public issue. Include the affected version, reproduction steps, and whether the issue could cross the simulated/native boundary.

## Release checks

Run:

```bash
python3 scripts/check_forbidden_apis.py
python3 scripts/validate_all.py
cargo deny check
cargo audit
```

A stable release additionally requires platform-specific capability inspection, offline network-denial testing, imported-pack traversal/bomb tests, and a clean dependency notice bundle.
