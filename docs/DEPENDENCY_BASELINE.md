# Dependency Baseline

This scaffold pins the application-facing libraries that materially affect architecture:

| Component | Starter pin | Role |
|---|---:|---|
| Tauri | 2.11.5 | desktop shell |
| tauri-build | 2.6.2 | Tauri build integration |
| Tauri CLI | 2.11.4 | documented developer tooling |
| Leptos | 0.8.20 | client-side Rust UI |
| wasm-bindgen | 0.2.126 | Rust/browser bridge |
| Trunk | 0.21.14 | WASM web build tooling |

The remaining crates use compatible semver ranges in the starter workspace. The first build host must resolve and commit `Cargo.lock`, run license/advisory review, and record any pin changes in an ADR. These pins were selected from current official project/crate documentation during pack generation, but compilation was unavailable in the creation environment.

## Upgrade policy

Requalify when changing:

- Tauri or its capability schema;
- Leptos reactive/mount APIs;
- wasm-bindgen/serde-wasm-bindgen message behavior;
- Rust edition/MSRV;
- serialization formats that affect state digests;
- any dependency introducing network, process, shell, filesystem, or dynamic-code capabilities.
