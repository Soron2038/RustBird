---
name: rust-check
description: Run cargo fmt --check, clippy, and tests on the Rust backend to verify correctness after backend changes
user-invocable: false
---

Run the following checks in sequence on the Rust backend. Report any failures immediately and stop — do not proceed to the next check if one fails.

1. `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
   - Verifies formatting without modifying files. If it fails, run `cargo fmt --manifest-path src-tauri/Cargo.toml` to auto-fix.

2. `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
   - All clippy lints are treated as errors. Pay attention to warnings about lock ordering, unused Results, and threading issues.

3. `cargo test --manifest-path src-tauri/Cargo.toml`
   - Runs all unit tests in crossfade.rs, error.rs, and state.rs.

If all three pass, report success. If any fail, show the relevant output and wait for the user to decide how to proceed.
