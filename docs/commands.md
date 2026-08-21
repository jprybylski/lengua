---
layout: default
title: Commands
nav_order: 3
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
`templates/` directory).

```bash
lengua init --store ./templates-repo
```

```
Initialized empty lengua store at ./templates-repo
```

`--json`:

```json
{ "status": "initialized", "path": "./templates-repo" }
```

Fails if `--store` already contains a `.git` directory.

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
