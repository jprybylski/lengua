---
layout: default
title: Commands
nav_order: 4
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
| `--store <STORE>` | `.` | Path to the template store (a git repo with a `templates/` dir) |
| `--json` | off | Emit structured JSON instead of human-readable output |

`--json` output is always a stable, documented shape (shown per-command below) — safe to pipe
into `jq`, an editor extension, or an AI agent.

---

## `init`

Create a new template-library git repo at `--store` (a bare `git init` plus an empty
`templates/` directory) — or adopt an existing one with `--from-dir`/`--from-repo`. See
[Consuming an existing library]({{ '/consuming.html' | relative_url }}) for the full
walkthrough.

```bash
lengua init --store ./templates-repo
```

```
Initialized empty lengua store at ./templates-repo
```

| Flag | Meaning |
|---|---|
| `--from-dir <PATH>` | Adopt an existing store from a local directory instead of starting empty. Mutually exclusive with `--from-repo`. |
| `--from-repo <SPEC>` | Adopt an existing store by cloning it. `SPEC` is a full git URL, or `[host/]owner/repo[/subdir][@ref]` shorthand — host defaults to `github.com`, so an explicit host is how GitHub Enterprise is supported. Mutually exclusive with `--from-dir`. |
| `--ref <REF>` | Branch or tag to check out (not a commit id). Only valid with `--from-repo`; overrides any `@ref` embedded in its shorthand. |
| `--subdir <PATH>` | Import only this subdirectory of the resolved source as the new store. Overrides any subdirectory embedded in `--from-repo`'s shorthand. **Imports current content only** — see the table in [Consuming an existing library]({{ '/consuming.html' | relative_url }}). |
| `--force` | With `--subdir`, allow re-importing into a store that already exists, overwriting templates with the same name. |

`--json`:

```json
{ "status": "initialized", "path": "./templates-repo" }
```

`status` is `"adopted"` instead of `"initialized"` when `--from-dir`/`--from-repo` is used.

Fails if `--store` already contains a `.git` directory (with `--subdir`, pass `--force` to
re-import into it instead).

---

## `add`

Add or update a template, committing the change. The body is read from stdin unless `--file`
is given.

```bash
lengua add letters/thank-you.md --title "Thank You" --field tone=formal <<'EOF'
Dear {{ name }},

Thank you for {{ reason }}.
EOF
```

| Flag | Meaning |
|---|---|
| `<NAME>` | Relative path/id under `templates/`, e.g. `letters/thank-you.md` |
| `--file <FILE>` | Read the body from this file instead of stdin |
| `--title <TITLE>` | Frontmatter title |
| `--field <KEY=VALUE>` | Frontmatter field; repeatable |
| `--message <MESSAGE>` | Commit message (default: `add/update template`) |

If the input already carries its own YAML frontmatter (e.g. a hand-authored file, or output
re-imported from `lengua get --raw`), `add` parses it and merges `--title`/`--field` on top as
overrides, rather than nesting a second frontmatter block.

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

`--json`:

```json
{ "name": "letters/thank-you.md", "rendered": "Dear Ada,\n\nThank you for the review.\n" }
```

Unset `{{ variables }}` are left as-is by minijinja's default undefined behavior unless the
template supplies a default (`{{ name | default("there") }}`).

---

## `list`

List every template currently in the store, sorted by name.

```bash
lengua list
```

```
letters/thank-you.md	Thank You
greetings/hello.md	Hello
```

`--json`:

```json
[
  { "name": "letters/thank-you.md", "title": "Thank You" },
  { "name": "greetings/hello.md", "title": "Hello" }
]
```

---

## `search`

Filter templates by frontmatter field. Multiple `--field` flags are ANDed together; at least
one is required.

```bash
lengua search --field tone=formal
```

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

```
  Dear {{ name }},
  
- Thank you for your time.
+ Thank you for {{ reason }}.
```

`FROM`/`TO` default to `HEAD~1`/`HEAD`.

`--json`:

```json
[
  { "tag": "equal", "line": "Dear {{ name }}," },
  { "tag": "delete", "line": "Thank you for your time." },
  { "tag": "insert", "line": "Thank you for {{ reason }}." }
]
```

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

`--json`:

```json
{ "status": "removed", "template": "letters/thank-you.md", "tag": "tense-past" }
```
