# Next Actions

## Completed through macOS RC (2026-08-06)

M0–M5 product path and unsigned macOS `.app` are in place. See `IMPLEMENTATION_STATUS.md`.

## Remaining for a “public v1.0” polish bar

1. Fix `bundle_dmg.sh` / DMG packaging on this host (or document `.app`-only distribution).
2. Apple Developer ID signing + notarization (requires credentials).
3. Move `SimSession` into a dedicated Web Worker thread.
4. Expand IndexedDB event log + migration tests (PRD §26).
5. `cargo deny` + THIRD_PARTY_NOTICES from lockfile.
6. Windows x64 / Linux x86-64 CI packages (APP-003).
7. Deeper golden command matrix coverage and a11y audit.
8. Trademark/public naming review before open release.

## Day-to-day run

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"
cd /path/to/DGX_Lab
cargo tauri dev
# or open: target/release/bundle/macos/DGX\ Lab.app
```
