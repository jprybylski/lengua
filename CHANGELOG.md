# Changelog

## [Unreleased]

### Added

- `lengua skills [DIRECTORY] [--force]` exports lengua's bundled coding-agent skill files
  (`SKILL.md`) to a target directory (default: current directory) — point it at
  `.claude/skills` for Claude Code to auto-discover them, or anywhere else for a different
  tool. Closes #5.
- A rustdoc templating guide (`lengua_core::template`'s module docs) covering minijinja's
  actual supported syntax: interpolation, filters, conditionals, loops, arithmetic, and
  escaping.

### Fixed

- Every literal `{{ variable }}` example in the Jekyll docs site was being silently swallowed
  by GitHub Pages' Liquid processor (which interprets `{{ }}` as its own template syntax) —
  wrapped in `{% raw %}` blocks so the examples actually render. Closes #2.

## [0.3.0] - 2026-08-21

### Changed

- **Breaking**: the on-disk store layout moves from `<store>/{.git,templates/}` to
  `<store>/.lengua/<source>/{.git,templates/}`, with a new `.lengua/sources.toml` manifest.
  A `--store` now points at a *library* that can hold one or more named *sources* instead of
  a single git repo — the unit lengua has always managed (`.git` + `templates/`) is unchanged,
  it's just nested one level deeper. Old-layout stores are not migrated; `lengua` (any command
  but `init`) now fails with a clear error pointing at the new layout instead of guessing.
  Closes #3.
- `Store::open`'s old heuristic ("does `templates/` exist next to `.git`") is replaced by an
  unambiguous check: does `.lengua/sources.toml` exist.

### Added

- `lengua fetch` adds another source to an already-initialized library, using the same
  `--from-dir`/`--from-repo`/`--ref`/`--subdir`/`--force` flags as `init`. This is how a
  project pools templates from several existing stores without merging their git history —
  including a project that never runs plain `init`/`add` at all, only ever consuming shared
  sources.
- `lengua update [--source <NAME>]` refreshes one or every source from its recorded origin via
  a fast-forward-only git fetch (never a hard reset) — a diverged source fails loudly rather
  than losing anything, and updating "all" reports every source's outcome even after one
  fails. There's no separate lockfile: each source's own git history is the record of what
  commit it's at.
- `--source <NAME>` on `add`/`get`/`list`/`search`/`log`/`diff`/`tag`: required to disambiguate
  a write or single-source read once a library has more than one source; optional on
  `get`/`list`/`search` to read one source directly instead of the merged view.
- Merged `get`/`list`/`search` (no `--source`) resolve a name across every source with
  last-fetched-wins precedence. A name defined in more than one source always prints a
  warning — at `fetch` time when the collision is created, and again whenever a shadowed name
  is resolved — never a silent choice.
- `init --name <NAME>` / `fetch --name <NAME>` name a source explicitly; otherwise it's
  derived from `--from-dir`'s basename or `--from-repo`'s last path segment (plain `init` with
  no adoption flags defaults to `local`). A name collision on auto-derivation is an error
  asking for `--name`, not a silent rename.

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
