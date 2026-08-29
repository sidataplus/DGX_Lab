# Security Review Runbook

## Source review

```bash
python3 scripts/check_forbidden_apis.py
python3 scripts/validate_all.py
```

Review all changes to:

- `src-tauri/tauri.conf.json`;
- `src-tauri/capabilities/`;
- workspace dependencies;
- WASM/browser APIs;
- content-pack parsing/decompression;
- import/export paths;
- CSP;
- update/signing behavior.

## Runtime review

1. Launch packaged app with network capture/proxy denying all egress.
2. Complete a course and certification flow.
3. Enter `ssh`, `curl`, `bash`, `sbatch`, traversal paths, oversized content, malformed packs.
4. Confirm all are simulator-handled or rejected.
5. Confirm no child process is created.
6. Confirm no listener or outbound socket appears.
7. Confirm imported data remains within app storage/user-selected export.

## Capability diff gate

Any added Tauri permission/plugin is consequential. Require:

- user value;
- least-privilege scope;
- abuse case;
- denial test;
- static web fallback assessment;
- ADR;
- reviewer sign-off.

## Release scan

Search bundled strings for real hostnames, IP ranges, usernames, tokens, secrets, and institutional paths. The PRD/source reference documents are development artifacts and should not be embedded in public binaries unless explicitly intended.
