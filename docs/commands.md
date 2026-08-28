---
layout: default
title: Commands
nav_order: 6
---

# Commands
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Global options

Every subcommand accepts these two flags:

| Flag | Default | Meaning |
|---|---|---|
| `--store <STORE>` | `.` | Path to the template library (a directory containing `.lengua/`) |
| `--json` | off | Emit structured JSON instead of human-readable output |

`--json` output is always a stable, documented shape (shown per-command below) — safe to pipe
into `jq`, an editor extension, or an AI agent.

Most subcommands also accept `--source <NAME>` — see
[Libraries, sources, and `--source`](#libraries-sources-and---source) below.

---

## Libraries, sources, and `--source`

`--store` points at a **library**: a directory containing `.lengua/`, which holds one or more
named **sources** (`.lengua/<name>/`), each its own independent git-backed store. `init`
creates the first source; [`fetch`](#fetch) adds more, for pooling templates from several
existing stores into one library without ever having to merge their git history.

- **Reads** (`get`/`list`/`search`) with no `--source` merge across every source. If two
  sources define the same template name, the **most recently fetched source wins** and a
  warning is printed — never a silent resolution. Pass `--source <NAME>` to read one source
  directly, bypassing the merge (the only way to reach a copy that's currently shadowed).
- **Writes** (`add`) and the inherently single-source `log`/`diff`/`tag` need one unambiguous
  target: pass `--source <NAME>`, or omit it if the library has exactly one source. A library
  with more than one source requires `--source` on these commands.

See [Consuming an existing library]({{ '/consuming.html' | relative_url }}) for the full
`fetch`/`update` walkthrough.

---

## `init`

Create a new template library (a `.lengua/` directory) at `--store` with its first source — a
bare `git init` plus an empty `templates/` directory at `.lengua/<name>/` — or adopt an
existing store as that first source with `--from-dir`/`--from-repo`. See
[Consuming an existing library]({{ '/consuming.html' | relative_url }}) for the full
walkthrough.

```bash
lengua init --store ./templates-repo
```

```
Initialized empty lengua library at ./templates-repo (source 'local')
```

| Flag | Meaning |
|---|---|
| `--from-dir <PATH>` | Adopt an existing store from a local directory as the first source, instead of starting empty. Mutually exclusive with `--from-repo`. |
| `--from-repo <SPEC>` | Adopt an existing store by cloning it. `SPEC` is a full git URL, or `[host/]owner/repo[/subdir][@ref]` shorthand — host defaults to `github.com`, so an explicit host is how GitHub Enterprise is supported. Mutually exclusive with `--from-dir`. |
| `--ref <REF>` | Branch or tag to check out (not a commit id). Only valid with `--from-repo`; overrides any `@ref` embedded in its shorthand. |
| `--subdir <PATH>` | Import only this subdirectory of the resolved source as the new source. Overrides any subdirectory embedded in `--from-repo`'s shorthand. **Imports current content only** — see the table in [Consuming an existing library]({{ '/consuming.html' | relative_url }}) — and can't later be refreshed with [`update`](#update). |
| `--name <NAME>` | Name for the first source. Defaults to `local` when starting empty, or is derived from `--from-dir`/`--from-repo` when adopting one. |
| `--force` | With `--subdir`, allow re-importing into a source that already exists, overwriting templates with the same name. |

`--json`:

```json
{ "status": "initialized", "path": "./templates-repo", "source": "local" }
```

`status` is `"adopted"` instead of `"initialized"` when `--from-dir`/`--from-repo` is used, and
`source` is whatever name was used or derived.

Fails if `--store` already has a `.lengua/` directory (with `--subdir`, pass `--force` to
re-import into that source instead).

**Starting empty never force-creates a source you didn't ask for**: `init --from-dir`/
`--from-repo` adopts *only* that one source — a pure-consumer library that never has its own
writable content is fully supported; nothing forces an empty `local` source alongside it.

---

## `fetch`

Add another source to an already-initialized library. Takes the same adoption flags as `init`,
but requires `.lengua/` to already exist and **appends** rather than replaces — the new source
becomes the highest-precedence one for merged reads (see
[Libraries, sources, and `--source`](#libraries-sources-and---source)).

```bash
lengua fetch --store ./templates-repo --from-repo acme-org/other-templates
```

```
warning: 'letters/thank-you.md' is now shadowed by 'other-templates' (also defined in 'local')
Fetched source 'other-templates'
```

| Flag | Meaning |
|---|---|
| `--from-dir <PATH>` | Adopt the source from a local directory. Mutually exclusive with `--from-repo`. |
| `--from-repo <SPEC>` | Adopt the source by cloning it. Same `SPEC` shorthand as `init`. Mutually exclusive with `--from-dir`. |
| `--ref <REF>` | Branch or tag to check out. Only valid with `--from-repo`. |
| `--subdir <PATH>` | Import only this subdirectory. Can't later be refreshed with `update`. |
| `--name <NAME>` | Name for the new source. Auto-derived from `--from-dir`'s basename or `--from-repo`'s last path segment if omitted — errors asking for an explicit `--name` on a collision, rather than silently picking a different one. |
| `--force` | With `--subdir`, allow re-importing into a source name that already exists. |

A `--from-dir` pointed at another local *library* (a directory with its own `.lengua/`, rather
than a bare store) adopts that library's sole source directly — if it has more than one, point
`--from-dir` at the specific `.lengua/<name>` you want instead.

`--json`:

```json
{ "status": "fetched", "source": "other-templates", "warnings": ["'letters/thank-you.md' is now shadowed by 'other-templates' (also defined in 'local')"] }
```

---

## `update`

Refresh one source (`--source`) or every source in the library from its recorded origin,
**fast-forward only** — never discards anything. A source that has locally diverged from its
origin (e.g. it was `add`ed to directly) fails loudly for that source rather than being
silently overwritten or silently skipped. `local` and `--subdir`-imported sources have no
origin to refresh from and are reported as such, not treated as a hard failure.

```bash
lengua update --store ./templates-repo
```

```
local: not-updatable (source 'local' can't be updated: a local source has no origin to update from)
other-templates: fast-forwarded (a1b2c3d4e5f6..f6e5d4c3b2a1)
```

| Flag | Meaning |
|---|---|
| `--source <NAME>` | Refresh only this source. Defaults to every source in the library. |

Updating "all" never stops at the first failure — every source's outcome is reported. The
process exits non-zero only if a source genuinely couldn't be fast-forwarded; a `not-updatable`
source is informational.

`--json`:

```json
[
  { "source": "local", "status": "not-updatable", "detail": "source 'local' can't be updated: a local source has no origin to update from" },
  { "source": "other-templates", "status": "fast-forwarded", "detail": "a1b2c3d4e5f6..f6e5d4c3b2a1" }
]
```

`status` is one of `up-to-date`, `fast-forwarded`, `not-updatable`, or `error` (a genuine
divergence).

---

## `add`

Add or update a template, committing the change. The body is read from stdin unless `--file`
is given.

{% raw %}
```bash
lengua add letters/thank-you.md --title "Thank You" --field tone=formal <<'EOF'
Dear {{ name }},

Thank you for {{ reason }}.
EOF
```
{% endraw %}

| Flag | Meaning |
|---|---|
| `<NAME>` | Relative path/id under the source's `templates/` dir, e.g. `letters/thank-you.md` |
| `--file <FILE>` | Read the body from this file instead of stdin |
| `--title <TITLE>` | Frontmatter title |
| `--field <KEY=VALUE>` | Frontmatter field; repeatable |
| `--message <MESSAGE>` | Commit message (default: `add/update template`) |
| `--source <NAME>` | Which source to write to. Required if the library has more than one source. |

If the input already carries its own YAML frontmatter (e.g. a hand-authored file, or output
re-imported from `lengua get --raw`), `add` parses it and merges `--title`/`--field` on top as
overrides, rather than nesting a second frontmatter block.

`title` is the only frontmatter field lengua knows about — it's what `list`/`search` print
alongside a template's name. Every other `--field key=value` is arbitrary: lengua doesn't
interpret `tone`, `jurisdiction`, or any other key, it just stores whatever you write and lets
[`search`](#search) filter on it later. A field can hold a list (write it directly in YAML
frontmatter rather than via `--field`, which only sets scalars); `search --field key=value`
matches if `value` equals the scalar or is one of the list's elements.

Nothing here is related to [`lengua tag`](#tag) — that command names *revisions* of a template
(a point in its git history), not frontmatter metadata. If you're tempted to add a `tags:`
field to track something like "final" or "v2", `lengua tag add` is almost certainly what you
want instead; see [FAQ: How is a tag different from a git tag?]({{ '/faq.html#how-is-a-tag-different-from-a-git-tag' | relative_url }}).

`--json`:

```json
{ "status": "added", "name": "letters/thank-you.md", "commit": "<full commit sha>" }
```

---

## `get`

Render a template with variables substituted (alias in earlier docs: "render").

```bash
lengua get letters/thank-you.md --var name=Ada --var reason="the review"
```

| Flag | Meaning |
|---|---|
| `<NAME>` | Template name |
| `--var <KEY=VALUE>` | Template variable; repeatable |
| `--raw` | Print the raw, unrendered body instead |
| `--rev <REV>` | Read the template as it existed at this revision instead of the working tree — a [tag](#tag) name, or any revspec `gix` understands (`HEAD`, `HEAD~1`, a commit sha) |
| `--source <NAME>` | Read from this source specifically, bypassing merge precedence — the only way to reach a copy shadowed by another source. |

`--json`:

```json
{ "name": "letters/thank-you.md", "source": "local", "rendered": "Dear Ada,\n\nThank you for the review.\n" }
```

`source` names whichever source the template was actually read from (the one passed to
`--source`, or the merge winner when unscoped).

{% raw %}Unset `{{ variables }}` are left as-is by minijinja's default undefined behavior unless the
template supplies a default (`{{ name | default("there") }}`).{% endraw %}

---

## `list`

List every template in the library, sorted by name — merged across every source (last-fetched
wins on a name collision) unless `--source` scopes it to one.

```bash
lengua list
```

```
letters/thank-you.md	Thank You	[local]
greetings/hello.md	Hello	[local]
```

| Flag | Meaning |
|---|---|
| `--source <NAME>` | List only this source's templates. |

`--json`:

```json
[
  { "name": "letters/thank-you.md", "title": "Thank You", "source": "local" },
  { "name": "greetings/hello.md", "title": "Hello", "source": "local" }
]
```

---

## `search`

Filter templates by frontmatter field — any key you've written via [`add --field`](#add) or by
hand in a template's YAML frontmatter, not a fixed schema. Multiple `--field` flags are ANDed
together; at least one is required.

```bash
lengua search --field tone=formal
```

| Flag | Meaning |
|---|---|
| `--source <NAME>` | Search only this source's templates. |

`--json` output has the same shape as `list`.

---

## `log` and `diff`

<div class="tape">
  <img src="{{ '/assets/img/history.gif' | relative_url }}" alt="lengua add, log, and diff demo" />
</div>

### `log`

Show the commit history for one template, newest first. Consecutive commits where the file's
content didn't change are collapsed.

```bash
lengua log letters/thank-you.md
```

```
a1b2c3d4e5f6  v2
9f8e7d6c5b4a  v1
```

| Flag | Meaning |
|---|---|
| `--source <NAME>` | Which source to read. Required if the library has more than one source. |

`--json`:

```json
[
  { "commit": "<full sha>", "message": "v2" },
  { "commit": "<full sha>", "message": "v1" }
]
```

### `diff`

Show a line-based diff of a template's content between two revisions (any revspec
`git`/`gix` understands: `HEAD`, `HEAD~1`, a commit sha, ...).

```bash
lengua diff letters/thank-you.md HEAD~1 HEAD
```

{% raw %}
```
  Dear {{ name }},
  
- Thank you for your time.
+ Thank you for {{ reason }}.
```
{% endraw %}

`FROM`/`TO` default to `HEAD~1`/`HEAD`.

| Flag | Meaning |
|---|---|
| `--source <NAME>` | Which source to read. Required if the library has more than one source. |

`--json`:

{% raw %}
```json
[
  { "tag": "equal", "line": "Dear {{ name }}," },
  { "tag": "delete", "line": "Thank you for your time." },
  { "tag": "insert", "line": "Thank you for {{ reason }}." }
]
```
{% endraw %}

`tag` is one of `equal`, `insert`, `delete`.

---

## `tag`

<div class="tape">
  <img src="{{ '/assets/img/tag.gif' | relative_url }}" alt="lengua tag add, list, and rm demo" />
</div>

Named pointers to a specific revision of a single template. **These are not git tags** —
`git tag` creates a repo-wide ref under `refs/tags/`, but lengua's tags live under
`refs/lengua/tags/<template>/<tag>`, scoped to one template, so the same tag name (e.g.
`"final"`) can exist independently on several templates without colliding. See
[FAQ: How is a tag different from a git tag?]({{ '/faq.html#how-is-a-tag-different-from-a-git-tag' | relative_url }})
for more.

A tag name anywhere `get --rev` or `diff`'s `FROM`/`TO` accept a revision works — tags are
tried first, falling back to any revspec `gix` understands.

### `tag add`

Point a tag at a template's current revision, or at `--rev` — this is how you retroactively tag
a *prior* revision, e.g. tagging the version before your latest edit:

```bash
lengua tag add letters/thank-you.md tense-future
lengua tag add letters/thank-you.md tense-past --rev HEAD~1
```

| Flag | Meaning |
|---|---|
| `<TEMPLATE>` | Template name |
| `<TAG>` | Tag name |
| `--rev <REV>` | Revision to tag instead of the current `HEAD` |
| `--force` | Overwrite the tag if it already exists |
| `--source <NAME>` | Which source to target. Required if the library has more than one source. |

Refuses to overwrite an existing tag without `--force`. Tag names can't be `HEAD` (any casing)
or look like a commit id — both would be ambiguous as a revision.

`--json`:

```json
{ "tag": "tense-future", "commit": "<full commit sha>" }
```

### `tag list`

```bash
lengua tag list letters/thank-you.md
```

```
tense-future  a1b2c3d4e5f6
tense-past    9f8e7d6c5b4a
```

| Flag | Meaning |
|---|---|
| `--source <NAME>` | Which source to read. Required if the library has more than one source. |

`--json`:

```json
[
  { "tag": "tense-future", "commit": "<full sha>" },
  { "tag": "tense-past", "commit": "<full sha>" }
]
```

### `tag rm`

```bash
lengua tag rm letters/thank-you.md tense-past
```

| Flag | Meaning |
|---|---|
| `--source <NAME>` | Which source to target. Required if the library has more than one source. |

`--json`:

```json
{ "status": "removed", "template": "letters/thank-you.md", "tag": "tense-past" }
```

---

## `skills`

Export lengua's bundled coding-agent skill files (`SKILL.md`) to a target directory. Doesn't
touch a library — no `--store`/`--source` involved. Each skill lands under its own
`<directory>/<skill-name>/SKILL.md` subdirectory, since a `SKILL.md` file's name is fixed by
the [Agent Skills](https://github.com/anthropics/skills) convention.

```bash
lengua skills .claude/skills
```

| Flag | Meaning |
|---|---|
| `<DIRECTORY>` | Target directory. Defaults to the current directory. Point this at `.claude/skills` for Claude Code to auto-discover the skill(s), or anywhere else for a different tool, or just to inspect the content. |
| `--force` | Overwrite an existing `SKILL.md` at the destination |

Refuses to overwrite an existing `SKILL.md` at the destination without `--force`.

`--json`:

```json
{ "directory": ".claude/skills", "created": [".claude/skills/lengua-template-authoring/SKILL.md"] }
```
