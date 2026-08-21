# Changelog

## [Unreleased]

## [0.1.3] - 2026-08-21

### Added

- Release binaries now also published for Windows (`x86_64-pc-windows-msvc`).

### Changed

- Release asset names now follow `lengua_<version>_<os>_<arch>` (e.g. `lengua_0.1.3_linux_amd64.tar.gz`), matching `datum`'s convention, instead of raw Rust target triples (`lengua-x86_64-unknown-linux-gnu.tar.gz`).

## [0.1.2] - 2026-08-21

### Fixed

- `add` no longer fails with `Author identity is not configured` when no git identity (`user.name`/`user.email`, or `GIT_AUTHOR_*`/`GIT_COMMITTER_*`) is set anywhere in the environment - found on a fresh CI runner, but affects any machine without git configured. Falls back to a fixed `lengua <lengua@localhost>` identity when none is configured, and still uses the real one when it is.

## [0.1.1] - 2026-08-21

### Fixed

- `TemplateMeta`: templates with no title no longer get a spurious `title: null` written into their frontmatter block.

## [0.1.0] - 2026-08-21

### Added

- `lengua-core`: minijinja-based rendering, YAML/TOML frontmatter parsing (`gray_matter`), git-backed storage via `gix` (init/add/get/list/log/diff), and in-memory tag/field query filtering.
- `lengua` CLI: `init`, `add`, `get`/`render`, `list`, `search`, `log`, `diff`, each with a `--json` output mode.
