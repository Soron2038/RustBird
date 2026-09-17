# RustBird — Claude Instructions

## Project Reference

See [AGENTS.md](AGENTS.md) for architecture, commands, file structure, and conventions. Everything there applies. This file adds Claude-specific rules.

## Git Workflow

Auto-commit to `main` is the standard workflow for this project. No feature branches needed.

## Releases

Always bump the version via `npm version` — never edit `package.json`, `Cargo.toml`, `tauri.conf.json`, or `Cargo.lock` by hand. The `version` lifecycle hook syncs all four files, commits them, and tags.

```bash
npm version patch        # 1.1.12 → 1.1.13
git push && git push --tags
```

Same pattern for `minor`, `major`, or a literal `1.2.0`.

## Testing

New features and bugfixes must include tests:

- **Frontend:** Vitest + @testing-library/svelte (see `src/lib/*.test.ts` for examples)
- **Rust:** `cargo test --manifest-path src-tauri/Cargo.toml`

No strict TDD required, but every change that touches logic should have test coverage.

## Svelte 5 Only

Use exclusively Svelte 5 runes syntax:

- `$state`, `$derived`, `$props`, `$effect`
- **Never** use Svelte 4 syntax (`$:`, `export let`, stores API)

## Tray Menu (macOS)

The right-click menu is deliberately **not** attached permanently on macOS (`tray.set_menu` is `cfg(not(target_os = "macos"))` in `lib.rs`). On macOS 27 a menu that is permanently attached to the `NSStatusItem` swallows the left click — AppKit pops the menu before `tray-icon`'s overlay view sees the event, so `TrayIconEvent::Click` never fires and the popover can't open ([tray-icon#355](https://github.com/tauri-apps/tray-icon/issues/355)). `show_tray_menu` in `lib.rs` attaches the menu on right mouse-down, pops it via `show_menu()`, and detaches it again — the same trick upstream shipped in `tray-icon` 0.25.1, which no Tauri 2.x release pulls in yet.

Don't "clean this up" by restoring an unconditional `set_menu`. Once Tauri 2.x ships `tray-icon` ≥ 0.25.1 the workaround becomes redundant but stays harmless. Requires `tray-icon` ≥ 0.23.1 (`show_menu`), i.e. Tauri ≥ 2.11 — don't downgrade below that.

## Auto-Update

`src-tauri/src/updater.rs` polls `latest.json` (written to `main` by `release.yml` on every tag) at launch and every 6 h, downloads a newer version silently and emits `update-ready`; the frontend's `UpdateBanner` then calls `install_update`, which installs and restarts. The endpoint only works while the GitHub repo is public (raw.githubusercontent.com and release assets 404 for private repos).

To test the flow in a dev build without touching `tauri.conf.json`, point it at a local manifest: `RUSTBIRD_UPDATER_ENDPOINT=http://127.0.0.1:8765/latest.json npm run tauri dev` (debug builds only). The manifest's `url` must reference a genuinely signed release asset — the minisign signature is verified before the download is accepted. Installing must NOT be triggered in dev: the plugin derives the install path from the executable, which is `target/debug` there, and would replace that directory.

A local `npm run tauri build -- --bundles app` needs `TAURI_SIGNING_PRIVATE_KEY` because `createUpdaterArtifacts` is on; a throwaway key from `npx tauri signer generate` is fine when only the `.app` is needed (the `.sig` is then unusable, the embedded public key stays the real one).
