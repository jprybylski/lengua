---
layout: default
title: Architecture
nav_order: 5
---

# Architecture
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Crate layout

```
lengua/
├── Cargo.toml                # workspace
├── crates/
│   ├── lengua-core/           # library: templating, frontmatter, git store, query
│   └── lengua-cli/            # bin `lengua`: clap subcommands over lengua-core
├── fixtures/                  # sample template library, used by tests and docs
└── docs/                      # this site
```

`lengua-core` has no CLI dependency (no `clap`) and no I/O beyond the filesystem and git — it's
the crate a future R binding (see [lenguar](https://github.com/jprybylski/lenguar)) links
against directly, so it stays a clean library with a stable, documented `Result`/`Error` type
rather than anything CLI-shaped.

## Libraries and sources

A `--store` points at a **library**: a directory containing `.lengua/`, which holds a
`sources.toml` manifest plus one or more named **sources** at `.lengua/<name>/`. This two-level
split exists to support pooling templates from several independent stores into one library
without merging their git history — a project that doesn't want to maintain its own templates
can still `fetch` several existing ones and query across all of them.

`lengua_core::Library` owns the manifest and every source's `Store`, and is what `lengua-cli`
actually calls:

- **`init`** creates `.lengua/` with one source (empty, or adopted via `--from-dir`/
  `--from-repo`). **`fetch`** appends another source to an existing library, using the same
  adoption flags. **`update`** refreshes a source from its recorded origin via a git
  fetch-and-fast-forward — never a hard reset, so a diverged source fails loudly instead of
  losing anything.
- **Merged reads** (`get`, `list`, `search` with no `--source`) resolve a name across every
  source with **last-fetched-wins** precedence; a name defined in more than one source always
  produces a warning (printed at `fetch` time and again whenever a shadowed name is resolved),
  never a silent choice. `--source <NAME>` reads one source directly, bypassing the merge —
  the only way to reach a shadowed copy.
- **Writes** (`add`) and the inherently single-source `log`/`diff`/`tag` need one unambiguous
  target: the sole source if there's only one, otherwise an explicit `--source`.

`sources.toml` deliberately isn't a lockfile: it records each source's name and how it was
adopted (`local`, a filesystem `copied` origin, or a `cloned` remote URL/ref) so `update` knows
which fetch strategy to use, but never a pinned commit id — each source's own `.git` is the
single authority on what commit it's at, exactly the way a real git checkout works.

## The store

A `Store` (one per source, at `.lengua/<name>/`) wraps a `gix::Repository` rooted at that
directory, plus the `templates/` directory inside it — the entire on-disk shape a *bare*
lengua store has always had, unchanged since before the `Library`/multi-source layer existed.
It deliberately splits reads and writes across two different mechanisms:

- **Current-state reads** (`get`, `list`, `search`) read straight from the **working tree** on
  disk. This is the simple, obvious thing: the working tree *is* the current state, and reading
  files is cheaper and easier to reason about than walking git tree objects.
- **Writes** (`add`) write the file to the working tree, then build a git commit directly via
  [`gix`](https://github.com/GitoxideLabs/gitoxide)'s tree editor (`edit_tree` → `write_blob` →
  `upsert` → `commit`) — no shelling out to a `git` binary, no `libgit2`.
- **History** (`log`, `diff`) walks commit ancestry via `gix` (`head_id().ancestors().all()`)
  and reads blobs at arbitrary revisions via `rev_parse_single`, so they reflect real git
  history even though current-state reads don't touch git objects at all.

This is a deliberate simplification: a design that read *everything* through git tree objects
(including `list`/`search`) would be more "pure," but meaningfully more code for no behavioral
difference in the common case, since `add` is the only thing that can make the working tree and
`HEAD` diverge, and `lengua` doesn't expose any way to edit the working tree outside of `add`.

Because a bare `Store`'s shape never changed, adopting an *existing*, already-published lengua
store as a new source (`fetch --from-repo ...`) needs no git-history rewriting at all — only
where it's checked out locally (nested under `.lengua/<name>/` instead of at `--store`'s root)
is new.

## Templates on disk

Each template is one file under `templates/`, with YAML frontmatter followed by a
[minijinja](https://github.com/mitsuhiko/minijinja) body:

```markdown
---
title: Thank You
tone: formal
---

Dear {{ name }},

Thank you for {{ reason }}.
```

Frontmatter parsing/writing (`lengua_core::frontmatter`) uses
[`gray_matter`](https://github.com/the-alchemist/gray-matter) for parsing and `serde_yaml` for
writing, round-tripping through a typed `TemplateMeta { title: Option<String>, fields: BTreeMap<String, serde_yaml::Value> }`
so `title` gets first-class treatment (surfaced in `list`/`search`) while every other field is
free-form.

## Tags: `refs/lengua/tags/*`

`lengua tag` implements named revisions as custom git refs, not `git tag`. Each tag lives at
`refs/lengua/tags/<template>/<tag>`, written/read/listed/deleted directly through `gix`'s
reference API (`repo.reference(...)`, `repo.references()?.prefixed(...)`,
`repo.edit_reference(...)`) — never `refs/tags/*`, which is git's own repo-wide tag namespace
and can't be scoped per template. Keeping tags in their own namespace also means `init
--from-repo`/`--from-dir` has to fetch them explicitly: a plain clone only brings across
branches and `refs/tags/*`, so cloning adds `refs/lengua/tags/*:refs/lengua/tags/*` as an extra
fetch refspec (see [`source::clone_direct`](https://github.com/jprybylski/lengua/blob/main/crates/lengua-core/src/source.rs))
to carry tags across along with history.

Revision resolution (`read_at_revision`, used by `get --rev` and `diff`) tries a template-scoped
tag lookup first, falling through to `rev_parse_single` (any revspec `gix` understands) if no
such tag exists — so a tag name is just another valid revision selector everywhere a revspec is
accepted, with no separate CLI syntax.

## What was deliberately left out

lengua's design started from a broader research survey of templated-language and
snippet-library tooling (bkmr, Fabric, and others). Several ideas from that survey were
rejected for v1, on purpose:

- **A SQL-like query language.** `search` is a hand-rolled AND-of-equality filter over parsed
  frontmatter. It has no dependency risk and marshals trivially across the future R/FFI
  boundary; a real query language can be layered on top later if `--field` filtering turns out
  to be insufficient, but nothing here commits to that yet.
- **A second, SQLite-backed index.** Git *is* the source of truth. A cache/index database would
  duplicate it and need to be kept in sync.
- **An LSP server or semantic/embedding search.** Out of scope for a template library CLI; can
  be added later as an optional crate/feature if a real need shows up.
- **A bespoke LLM/Ollama client.** `--json` output on every command plus this documented CLI
  contract is the entire integration surface an AI agent needs. Which model or host to talk to
  is a caller's decision, not lengua's.
- **Project scaffolding.** `lengua init` means "create a new template-library git repo," full
  stop — never "bootstrap an app skeleton."

## Error handling

`lengua-core` uses [`thiserror`](https://github.com/dtolnay/thiserror) for a typed `Error` enum
(`Io`, `Git`, `Render`, `Frontmatter`, `NotFound`, `InvalidRevision`, ...) with no dependency on
any particular consumer. `lengua-cli` uses [`anyhow`](https://github.com/dtolnay/anyhow) at its
boundary, converting any `lengua_core::Error` into a clean stderr message and a non-zero exit
code. A future R binding maps the same typed `Error` enum onto R condition classes instead
(see `lenguar`'s `R/conditions.R`) — this is why the typed enum lives in `lengua-core`, not
folded into `anyhow` at the source.
