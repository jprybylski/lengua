# Changelog

## [Unreleased]

## [0.2.0] - 2026-08-21

### Added

- Tags: `tag add`/`list`/`rm` name a specific revision of a template without duplicating content, stored as `refs/lengua/tags/<template>/<tag>` (not real git tags, which are repo-wide) - the same tag name can independently exist on several templates, and `tag add --rev <revision>` retroactively tags a prior revision. Tag names now work anywhere a revision is accepted (`get --rev`, `diff`).
- `init --from-dir`/`--from-repo` (with `--ref`, `--subdir`, `--force`) adopts an existing store instead of only ever starting empty, mirroring `--from-repo`'s `"[host/]owner/repo[/subdir][@ref]"` shorthand from the sibling `quartifyr`/`deckifyr` projects (host defaults to `github.com`; an explicit host reaches GitHub Enterprise).
- New `docs/consuming.md` walkthrough for pulling down and using an existing shared template library.
- The CLI now has color (green/red for diff lines, red for errors, cyan for tag names, bold for headings, dim for metadata), respecting `NO_COLOR`/`CLICOLOR`/`CLICOLOR_FORCE`. `--json` output is unaffected.

### Fixed

- `Store::open` now refuses to open a directory that isn't a lengua store (no `templates/` dir), instead of silently committing template blobs into whatever git repo happened to be at that path - including a user's own project, if lengua were run there by mistake.

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
