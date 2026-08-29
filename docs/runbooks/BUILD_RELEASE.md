# Build and Release Runbook

## Development build

```bash
python3 scripts/validate_all.py
cargo test --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop
trunk build crates/web-ui/index.html --release --dist crates/web-ui/dist
cargo tauri build
```

## Pre-release evidence

- clean `Cargo.lock` committed;
- fmt/clippy/tests pass;
- native/WASM parity transcript passes;
- content/reference/schema validation passes;
- forbidden API/capability scan passes;
- dependency licenses reviewed;
- SBOM and vulnerability scan generated;
- CSP/capability snapshot reviewed;
- offline network test records zero outbound requests;
- institution-specific string/secret scan passes;
- bundle opens and completes Lab 04;
- certificate trust label verified;
- version/release notes/migrations complete.

## Platform matrix

| Platform | Package | Signing |
|---|---|---|
| macOS arm64 | `.dmg` / app bundle | Developer ID + notarization for public release |
| Windows x64 | installer | Authenticode recommended |
| Linux x86-64 | AppImage/deb as selected | checksums/signature |

Build on the target OS or an officially supported release workflow. Do not infer that one successful host build validates every system WebView.

## Artifact naming

```text
DGX-Lab_<version>_aarch64.dmg
DGX-Lab_<version>_x64-setup.exe
DGX-Lab_<version>_amd64.AppImage
DGX-Lab_<version>_web.zip
DGX-Lab_<version>_checksums.txt
```

## Rollback

Keep the prior signed installer and migration-compatible session reader. A release that cannot import supported prior sessions is not a minor update, no matter how optimistic the changelog sounds.
