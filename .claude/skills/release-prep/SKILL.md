---
name: release-prep
description: Bump version in package.json and src-tauri/Cargo.toml, run all tests, then create and push a version tag to trigger CI
disable-model-invocation: true
---

Steps to prepare and publish a new release:

1. If no version argument was provided, ask the user for the target version (e.g. 1.2.0).
2. Update the `version` field in `package.json`.
3. Update the `version` field in `src-tauri/Cargo.toml`.
4. Run `npm test` — abort and report if any tests fail.
5. Run `cargo test --manifest-path src-tauri/Cargo.toml` — abort and report if any tests fail.
6. Commit both files: `git commit -am "chore: bump version to vX.Y.Z"`
7. Create an annotated tag: `git tag vX.Y.Z`
8. Push with tags: `git push origin main --tags`
9. Confirm that the GitHub Actions release workflow has been triggered (the tag push triggers `.github/workflows/release.yml`).
