---
layout: default
title: Consuming an existing library
nav_order: 3
---

# Consuming an existing library
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

The [quick start]({{ '/#quick-start' | relative_url }}) walks through building a template
library from scratch with `lengua init`. In practice, most people using lengua on a given day
aren't building a library — they're pulling down one a team already maintains and using it.
`init --from-dir`/`--from-repo` cover that.

<div class="tape">
  <img src="{{ '/assets/img/consuming.gif' | relative_url }}" alt="lengua init --from-dir demo" />
</div>

## From a local directory

```bash
lengua init --store ./my-copy --from-dir ../shared-templates
```

Clones `../shared-templates` (any existing lengua store) into `./my-copy`, full history and all
[tags](#tag) intact. There's no network involved — `--from-dir` is a local clone, not a copy, so
the result is a real independent git repo you can commit into freely.

## From a git remote

```bash
lengua init --store ./my-copy --from-repo acme-org/team-templates
```

`--from-repo` accepts either a full git URL, or the `[host/]owner/repo[/subdir][@ref]`
shorthand:

| Piece | Meaning | Example |
|---|---|---|
| `host` | Defaults to `github.com`. Give one explicitly to use GitHub Enterprise, GitLab, or any other host. | `git.acme.internal` |
| `owner/repo` | Required. | `acme-org/team-templates` |
| `subdir` | Optional — see [Importing a subdirectory](#importing-a-subdirectory) below. | `letters` |
| `@ref` | Optional branch or tag to check out (not a commit id). | `@v2` |

```bash
# GitHub, default host
lengua init --store ./my-copy --from-repo acme-org/team-templates

# GitHub Enterprise, explicit host
lengua init --store ./my-copy --from-repo git.acme.internal/acme-org/team-templates

# a specific subdirectory and tag
lengua init --store ./my-copy --from-repo acme-org/team-templates/letters@v2

# a full URL works too, including scp-style git@ URLs
lengua init --store ./my-copy --from-repo git@github.com:acme-org/team-templates.git
```

`--ref`/`--subdir` flags override anything embedded in the shorthand, so
`--from-repo acme-org/team-templates@v1 --ref v2` checks out `v2`.

## Importing a subdirectory

`--subdir` (whether given as a flag or embedded in `--from-repo`'s shorthand) selects a
subdirectory of the resolved source to use as the new store, for when several teams' stores
live as subdirectories of one shared monorepo. It works differently from a plain
`--from-dir`/`--from-repo` clone, and the difference matters:

| | Plain clone (no `--subdir`) | With `--subdir` |
|---|---|---|
| History | Full source history preserved | **Not preserved** — one fresh commit per template, at import time |
| Tags | Every `refs/lengua/tags/*` ref preserved | **Not preserved** |
| Mechanism | A real `gix` clone of the source | A clone into a temporary staging area, then each template in the subdirectory is replayed as a new `add` into a fresh store |

There's no equivalent of `git subtree split` in `gix` to extract a subdirectory's history
losslessly, so `--subdir` intentionally trades history for simplicity: you get the
subdirectory's *current* content, imported cleanly, and nothing more. lengua prints a one-line
note when this path is taken so it's never a silent surprise. If you need the full history of a
subdirectory, clone the whole repo (no `--subdir`) and treat the subdirectory as your store root
via `--store <path>/<subdir>` instead.

Re-running an `--subdir` import into a store that already exists requires `--force`, which
overwrites templates with the same name; it does not touch templates that aren't in the import.

## Pooling more than one source

`init --from-dir`/`--from-repo` adopts a store as your library's *first* source. To add a
second (or third, ...) without starting a new library, use `fetch` — same adoption flags,
appended instead of replacing:

```bash
lengua fetch --store ./my-copy --from-repo other-org/other-templates
```

The new source becomes the highest-precedence one: `get`/`list`/`search` (with no `--source`)
merge every source together, and if two sources define the same template name, **the most
recently fetched one wins** — `fetch` prints a warning when this happens, and so does any
later `get`/`list`/`search` that resolves a shadowed name, so it's never a silent surprise:

```
$ lengua fetch --store ./my-copy --from-repo other-org/other-templates
warning: 'letters/thank-you.md' is now shadowed by 'other-templates' (also defined in 'local')
Fetched source 'other-templates'
```

Pass `--source <NAME>` to any read command to bypass the merge and reach a specific source
directly — including one that's currently shadowed. `add`/`log`/`diff`/`tag` need
`--source <NAME>` too, once a library has more than one source, since there's no default
"the" source to write to or read single-source history from anymore.

A library built entirely from `fetch`ed sources, with no `local` source of its own, is a
completely normal way to use lengua — a project that only ever *consumes* shared templates
never needs to run plain `lengua init`/`add` at all.

## Keeping a source up to date

`update` refreshes one source (`--source <NAME>`) or every source in the library from its
recorded origin:

```bash
lengua update --store ./my-copy
```

```
local: not-updatable (source 'local' can't be updated: a local source has no origin to update from)
other-templates: fast-forwarded (a1b2c3d4e5f6..f6e5d4c3b2a1)
```

This is a **fast-forward-only** git fetch, the same semantics as `git pull --ff-only` — it
never rewrites or discards history. If a source has locally diverged from its origin (for
example, someone ran `add --source other-templates` directly against a fetched source), that
source fails loudly instead of silently losing those commits; updating "all" still reports
every other source's outcome rather than stopping at the first failure. A `local` source, or
one imported with `--subdir` (its history/tags weren't preserved on import, so there's nothing
to fast-forward against), always reports as not-updatable — informational, not an error.

There's no separate version-lock file to keep in sync: each source is a real git checkout, and
`update` reads/writes its ref state the same way `git fetch`/`git merge --ff-only` would.

## See also

- [End-user guide]({{ '/end-user-guide.html' | relative_url }}) — in-depth walkthrough of pure-consumer setups, 3-tier layering, and scripting
- [Storekeeper guide]({{ '/storekeeper-guide.html' | relative_url }}) — maintaining, organizing, versioning, and publishing organization stores
- [`init`]({{ '/commands.html#init' | relative_url }}) — full flag reference
- [`tag`]({{ '/commands.html#tag' | relative_url }}) — what gets preserved by a plain clone
- [FAQ: What happens if I run lengua inside my own project's git repo?]({{ '/faq.html#what-happens-if-i-run-lengua-inside-my-own-projects-git-repo' | relative_url }}) — worth reading before picking where `./my-copy` lives
