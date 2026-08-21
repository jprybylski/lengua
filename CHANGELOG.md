# Changelog

## [Unreleased]

## [0.1.1] - 2026-08-21

### Fixed

- `TemplateMeta`: templates with no title no longer get a spurious `title: null` written into their frontmatter block.

## [0.1.0] - 2026-08-21

### Added

- `lengua-core`: minijinja-based rendering, YAML/TOML frontmatter parsing (`gray_matter`), git-backed storage via `gix` (init/add/get/list/log/diff), and in-memory tag/field query filtering.
- `lengua` CLI: `init`, `add`, `get`/`render`, `list`, `search`, `log`, `diff`, each with a `--json` output mode.
