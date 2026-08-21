---
layout: default
title: Home
nav_order: 1
description: "lengua is a git-backed CLI for managing a library of templated text and snippets."
permalink: /
---

# lengua

**lengua** is a git-backed library for templated text: prose, Markdown, LaTeX, or code
snippets, annotated with YAML frontmatter and rendered with Jinja-style variable substitution.
Every `add` is a real git commit, so your template library carries full history for free —
`log` and `diff` are just `git log`/`git diff` scoped to one template.
{: .fs-6 .fw-300 }

[Get started](#quick-start){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[View on GitHub](https://github.com/jprybylski/lengua){: .btn .fs-5 .mb-4 .mb-md-0 }

---

## What does lengua do?

lengua manages a directory of small, reusable templates — think a library of letter
templates, prompt fragments, boilerplate clauses, or commit-message skeletons — with:

- **Frontmatter fields** — arbitrary YAML fields (`tone: formal`, `jurisdiction: eu`, ...)
  on every template, filterable via `search`.
- **Jinja-style rendering** — {% raw %}`{{ name }}`{% endraw %} variable interpolation via
  [minijinja](https://github.com/mitsuhiko/minijinja), substituted at `get` time.
- **Git-native history** — every `add` commits the change; `log` and `diff` read real git
  history, no separate database.
- **Scriptable `--json` output** — every command supports `--json` for use from scripts,
  editors, or AI agents.

It is deliberately *not* a project scaffolder (`lengua init` creates a template-library repo,
not an app skeleton) and does not embed a query language, a database, or an LLM client — see
[Architecture]({{ '/architecture.html' | relative_url }}) for what was left out and why.

## Quick start

### 1. Initialize a library

```bash
lengua init --store ./templates-repo
```

This creates a `.lengua/` directory holding your first source (`.lengua/local/`, a git
repository with a `templates/` directory inside it). A library can later pool templates from
more than one source — see [Consuming an existing library]({{ '/consuming.html' | relative_url }})
for `fetch`/`update`.

<div class="tape">
  <img src="{{ '/assets/img/quickstart.gif' | relative_url }}" alt="lengua init, add, and get demo" />
</div>

### 2. Add a template

{% raw %}
```bash
lengua add letters/thank-you.md --store ./templates-repo --title "Thank You" --field tone=formal <<'EOF'
Dear {{ name }},

Thank you for {{ reason }}.

Warm regards,
{{ sender }}
EOF
```
{% endraw %}

Every `add` is a git commit — `lengua log letters/thank-you.md` shows its full history.

### 3. Render it

```bash
lengua get letters/thank-you.md --store ./templates-repo \
  --var name=Ada --var reason="your thoughtful review" --var sender=Grace
```

### 4. Find it later

```bash
lengua search --field tone=formal --store ./templates-repo
```

Ready for more? See [Commands]({{ '/commands.html' | relative_url }}) for the full reference.

## Where to next

| Page | What's there |
|---|---|
| [Installation]({{ '/installation.html' | relative_url }}) | Building from source, installing a release binary |
| [Consuming an existing library]({{ '/consuming.html' | relative_url }}) | Pulling down a shared library with `init --from-dir`/`--from-repo`, and pooling more sources with `fetch`/`update` |
| [Commands]({{ '/commands.html' | relative_url }}) | `init` / `fetch` / `update` / `add` / `get` / `list` / `search` / `log` / `diff` / `tag` / `skills`, flags, `--json` shapes |
| [Architecture]({{ '/architecture.html' | relative_url }}) | Crate layout, the git-backed store design, what was deliberately left out |
| [FAQ]({{ '/faq.html' | relative_url }}) | Common questions |
| [Templating guide](https://docs.rs/lengua-core/latest/lengua_core/template/) | Full minijinja syntax overview (rustdoc) — interpolation, filters, conditionals, loops, escaping |
