# Local Development Runbook

## Supported first host

macOS Apple Silicon is the first development target. Windows x64 and Linux x86-64 are release requirements after the walking skeleton.

## Prerequisites

- current stable Rust compatible with workspace `rust-version`;
- `wasm32-unknown-unknown` target;
- Trunk 0.21.14;
- Tauri CLI 2.11.4;
- operating-system Tauri prerequisites;
- Python 3.11+ for content validation scripts (PyYAML + jsonschema).

```bash
# If `cargo`/`rustc` from ~/.cargo/bin mis-resolve to rustup, put the toolchain first:
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"

rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
cargo install tauri-cli --version 2.11.4 --locked
```

## First verification

```bash
python3 scripts/validate_all.py
python3 scripts/check_forbidden_apis.py
cargo fmt --all -- --check
cargo clippy --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop --all-targets --all-features -- -D warnings
cargo test --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop
cargo build -p sim-worker-wasm --target wasm32-unknown-unknown
```

The generated source has not been compiled in the pack-creation environment. Expect a small compatibility-fix pass for exact current crate APIs before treating the scaffold as verified.

## Run the no-build prototype

```bash
cd prototype
python3 serve.py
```

## Run the Leptos client

```bash
cd crates/web-ui && trunk serve --port 1420 --address 127.0.0.1
```

## Run Tauri

```bash
cargo tauri dev
```

The Tauri configuration can start Trunk automatically. During debugging, two terminals often produce clearer logs.

## Build order

1. Core native crates and tests.
2. Scenario compiler against source YAML.
3. `sim-worker-wasm` build and JS smoke call.
4. Leptos client.
5. Tauri dev shell.
6. IndexedDB/persistence wiring.
7. end-to-end one-GPU lab.

## Never add

- `std::process::Command`;
- shell/SSH/Slurm/HTTP Tauri plugins;
- real cluster endpoints or credentials;
- arbitrary host file APIs;
- runtime-downloaded assets.
