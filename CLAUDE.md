# RustBird — Claude Instructions

## Project Reference

See [AGENTS.md](AGENTS.md) for architecture, commands, file structure, and conventions. Everything there applies. This file adds Claude-specific rules.

## Git Workflow

Auto-commit to `main` is the standard workflow for this project. No feature branches needed.

## Testing

New features and bugfixes must include tests:

- **Frontend:** Vitest + @testing-library/svelte (see `src/lib/*.test.ts` for examples)
- **Rust:** `cargo test --manifest-path src-tauri/Cargo.toml`

No strict TDD required, but every change that touches logic should have test coverage.

## Svelte 5 Only

Use exclusively Svelte 5 runes syntax:

- `$state`, `$derived`, `$props`, `$effect`
- **Never** use Svelte 4 syntax (`$:`, `export let`, stores API)
