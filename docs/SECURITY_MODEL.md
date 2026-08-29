# Security Model

## Primary invariant

**DGX Lab cannot touch a real cluster.** This is an architectural property, not a configuration checkbox.

## Forbidden capability set

- OS process spawning or PTY access;
- SSH libraries or executables;
- Slurm client libraries/binaries or REST adapters;
- arbitrary host filesystem access;
- unrestricted Tauri commands/plugins;
- outbound HTTP/WebSocket/network access;
- native/plugin code in imported content;
- dynamic JavaScript fetched from a URL.

## Controls in this pack

1. Pure Rust simulator crates use `#![forbid(unsafe_code)]`.
2. Tauri exposes `core:default` only and no custom commands.
3. CSP restricts `connect-src` to the application origin (`'self'`) for bundled WASM loading, and uses `object-src 'none'` and `frame-src 'none'`; no external origin is allowed.
4. The constrained shell dispatches typed virtual commands.
5. VFS path normalization rejects `..` traversal.
6. Content is YAML/JSON/Markdown validated against schemas and size/reference rules.
7. CI scans source/manifests for forbidden APIs and dependencies.
8. All prototype assets are local; it performs no fetch/XHR/WebSocket.
9. Official packs are digestable/signable; unsigned packs are explicitly labeled.
10. There is no `SchedulerBackend` interface with a dormant real implementation.

## Threat table

| Threat | Control |
|---|---|
| Learner enters `ssh`, `bash`, or malicious shell text | unsupported-command path; never reaches OS |
| Imported scenario embeds traversal path | schema + semantic validator + VFS normalization |
| Imported pack carries executable JavaScript/native code | pack allowlist of declarative extensions only |
| Tauri update adds broad plugin | capability diff and security ADR required |
| UI fabricates a pass | canonical grading in worker, evidence digest, replay checks |
| Learner edits local evidence | standalone trust limitation displayed; instructor/institution signatures separate |
| Dependency introduces network/process feature | cargo-deny review, dependency audit, forbidden API scan |
| CSP weakened | security test asserts application-origin-only `connect-src`, no external origins, and restricted worker sources |
| Public product implies vendor affiliation | original brand and trademark disclaimer/review |

## Content-pack limits

Recommended defaults:

- compressed pack ≤ 100 MiB;
- decompressed pack ≤ 500 MiB;
- ≤ 1,000 files;
- paths UTF-8, relative, normalized, no symlinks;
- allowed extensions: `.yaml`, `.yml`, `.json`, `.md`, `.txt`, `.svg`, `.png`, `.webp`;
- no HTML/JS/WASM/native library in imported packs;
- image dimensions and decompression ratio bounded.

## Release security gate

Before a public desktop release:

- Rust/Tauri compilation and tests on every target;
- dependency lockfile and license inventory;
- SBOM and vulnerability scan;
- Tauri capability snapshot review;
- offline packet-capture test;
- strings scan for institutional names, IPs, secrets, and forbidden endpoints;
- signed/notarized bundles;
- replay parity between native tests and release WASM.
