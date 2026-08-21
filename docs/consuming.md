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

## See also

- [`init`]({{ '/commands.html#init' | relative_url }}) — full flag reference
- [`tag`]({{ '/commands.html#tag' | relative_url }}) — what gets preserved by a plain clone
- [FAQ: What happens if I run lengua inside my own project's git repo?]({{ '/faq.html#what-happens-if-i-run-lengua-inside-my-own-projects-git-repo' | relative_url }}) — worth reading before picking where `./my-copy` lives
