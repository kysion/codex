# Repository Guidelines

## Project Structure & Module Organization
- `codex-rs/` is the active Cargo workspace; key crates include `core/` for agent logic, `tui/` for the Ratatui UX, `exec/` for non-interactive runs, and `protocol/` / `common/` for shared types. Tests live alongside each crate in `src` and `tests`.
- `codex-cli/` hosts the legacy Node wrapper (ships `bin/codex.js`); touch only when adjusting packaging.
- `docs/` contains published user guides; update matching doc when behaviour shifts.
- Supporting automation lives under `scripts/` and repository-level utilities (e.g., release tooling).

## Build, Test, and Development Commands
- `cd codex-rs && just codex` runs the Rust CLI in debug mode; use `just tui` or `just exec` for specific entrypoints.
- `just fmt` enforces workspace formatting; `just fix -p codex-tui` (or another crate) applies scoped Clippy fixes.
- `just test` prefers `cargo nextest run --no-fail-fast`; fall back to `cargo test -p <crate>` for targeted runs.
- For docs and JS assets, run `pnpm format` (check) or `pnpm format:fix` from the repo root.

## Coding Style & Naming Conventions
- Rust code uses 4-space indentation and must stay `rustfmt` clean; imports are flattened per `imports_granularity=Item`.
- Crate and package names follow the `codex-*` prefix (e.g., `codex-core`); modules remain snake_case, public types PascalCase, traits CamelCase.
- Avoid custom styling helpers in TUI code—prefer Ratatui’s `.dim()`, `.cyan()`, etc.; follow `tui/styles.md` when tweaking visuals.
- JavaScript/Markdown changes should pass Prettier (`pnpm format:fix`); keep command examples in fenced blocks.

## Testing Guidelines
- Use `just test` for broad validation; for single crates run `cargo test -p codex-tui` or similar before invoking workspace-wide suites.
- Snapshot updates rely on `cargo insta pending-snapshots` followed by `cargo insta accept` once reviewed; never accept snapshots blindly.
- Respect sandbox-aware tests: leave guards built around `CODEX_SANDBOX*` and skip flows that require external network access.

## Commit & Pull Request Guidelines
- Follow the existing log style: `<area>: succinct change (#issue)` (e.g., `core: tighten command filtering (#4211)`); keep commits atomic and DCO-signed.
- PRs must explain What/Why/How, link issues, and include TUI screenshots or terminal demos when behaviour shifts.
- Before requesting review, ensure `just fmt`, `just fix -p <crate>`, and the relevant `cargo test`/`cargo nextest` invocations succeed; attach notes for tests you could not run.

## Security & Configuration Notes
- Never weaken sandbox checks or alter `CODEX_SANDBOX`/`CODEX_SANDBOX_NETWORK_DISABLED` handling; these gates protect local users.
- Configuration defaults live in `docs/config.md`; reflect any new knobs in that doc and the CLI `--help`.
- When experimenting locally, prefer `codex --sandbox workspace-write` over disabling isolation entirely.
