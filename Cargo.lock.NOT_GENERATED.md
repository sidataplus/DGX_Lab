# Cargo.lock not generated in this pack

The artifact-generation environment did not contain Rust/Cargo and could not reach
crates.io. A lockfile was therefore not fabricated.

On the first approved build host:

```bash
cargo generate-lockfile
cargo update --dry-run
cargo check --workspace --all-targets --all-features
```

Review resolved licenses and advisories, then commit the generated `Cargo.lock`.
