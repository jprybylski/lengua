---
layout: default
title: End-user guide
nav_order: 4
---

# End-user guide: Consuming and layering template stores
{: .no_toc }

This guide covers advanced patterns for **end users** who consume templates published by their team, organization, or third parties without maintaining or modifying the upstream stores.

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Consumer mental model

In an organization, most people using `lengua` are consumers. A consumer might need:
- Standard corporate templates (e.g. legal notices, email footers, brand guidelines)
- Department-specific templates (e.g. sales pitches, engineering RFCs, support responses)
- Personal or project-specific template overrides

`lengua` models this with **libraries and sources**:
- A **library** is a `.lengua/` directory at your workspace root or a shared path.
- A **source** is an independent, git-backed template repository nested inside `.lengua/<source_name>/`.
- Reads (`get`, `list`, `search`) merge all sources seamlessly with **last-fetched-wins** precedence.

```
My Workspace Library (.lengua/)
├── corp/          <- Corporate base templates (lowest precedence)
├── engineering/   <- Department templates (medium precedence)
└── project/       <- Project-specific overrides (highest precedence)
```

---

## Setting up a pure consumer library

A pure consumer library contains only upstream sources and has no writable `local` source.

### 1. Adopt your first upstream source

Initialize your library by pointing directly to your team's upstream repository:

```bash
# From a remote git repository (GitHub / GitLab / Enterprise)
lengua init --store ./my-templates --from-repo acme-org/team-templates

# Or from a local filesystem path / network share
lengua init --store ./my-templates --from-dir /shared/templates/corp-standards
```

Notice that `init --from-repo`/`--from-dir` creates **only** the adopted source (e.g. `team-templates` or `corp-standards`). No empty `local` source is created.

### 2. Query and render templates immediately

```bash
# List all templates in the library
lengua --store ./my-templates list

# Search by frontmatter field
lengua --store ./my-templates search --field category=onboarding

# Render a template with variable substitution
lengua --store ./my-templates get letters/welcome.md --var name="Ada" --var role="Engineer"
```

---

## Multi-source layering and precedence

In complex workflows, you often need to combine templates from multiple origins. You add additional sources with `lengua fetch`.

### Layering example: 3-tier setup

Suppose your organization provides:
1. `acme/corp-standards`: Base corporate templates (`welcome.md`, `footer.md`, `legal/nda.md`).
2. `acme/sales-templates`: Department templates (`welcome.md`, `pitch.md`).
3. `acme/project-alpha`: Local project templates (`footer.md`, `summary.md`).

You assemble this stack by fetching each source in order from base to specific:

```bash
# Initialize with corporate standards
lengua init --store ./my-lib --from-repo acme/corp-standards --name corp

# Layer sales templates on top
lengua fetch --store ./my-lib --from-repo acme/sales-templates --name sales

# Layer project-specific templates on top
lengua fetch --store ./my-lib --from-repo acme/project-alpha --name project
```

### Precedence resolution: Last-fetched wins

When multiple sources define a template with the exact same name:
- The **most recently fetched source wins** the unscoped resolution.
- `lengua` prints a **shadow warning** to stderr whenever a name collision occurs.

| Template Name | Defined In Sources | Merge Winner (Unscoped) |
|---|---|---|
| `welcome.md` | `corp`, `sales` | `sales` (shadows `corp`) |
| `footer.md` | `corp`, `project` | `project` (shadows `corp`) |
| `legal/nda.md` | `corp` | `corp` |
| `pitch.md` | `sales` | `sales` |
| `summary.md` | `project` | `project` |

### Shadow warnings

Shadowing is never silent. When you fetch a source that collides with an existing one, `lengua` notifies you:

```
$ lengua fetch --store ./my-lib --from-repo acme/sales-templates --name sales
warning: 'welcome.md' is now shadowed by 'sales' (also defined in 'corp')
Fetched source 'sales'
```

When you render or list templates, shadow warnings are printed to stderr so you always know which layer provided the template.

---

## Bypassing the merge: Reading shadowed templates

If a template is shadowed, you can still access the underlying base version by passing `--source <NAME>`:

```bash
# Unscoped read resolves to the winning 'sales' version:
lengua --store ./my-lib get welcome.md --var name=Ada

# Explicitly read the corporate base version:
lengua --store ./my-lib get welcome.md --source corp --var name=Ada
```

The `--source` flag works across all read commands:
- `lengua get <name> --source <source>`
- `lengua list --source <source>`
- `lengua search --field <key=val> --source <source>`

---

## Reproducibility: Pinning tags and revisions

When generating reports, contracts, or automated documentation, you may need to ensure templates do not change unexpectedly.

### Pinning at checkout time (`--ref`)

When adopting a source, check out a specific release branch or git tag:

```bash
# Pin to a major release branch or tag
lengua init --store ./my-lib --from-repo acme/corp-templates@v2.0 --name corp
```

### Time-travel rendering (`--rev`)

You can render a template as it existed at any historical revision or template-scoped tag without altering your working tree:

```bash
# Render using a template tag
lengua --store ./my-lib get legal/nda.md --rev v1.0 --var recipient="Acme Partner"

# Render using git revision syntax (HEAD~1, commit SHA)
lengua --store ./my-lib get legal/nda.md --rev HEAD~2 --var recipient="Acme Partner"
```

---

## Keeping upstream sources up to date

Upstream storekeepers will regularly publish new templates, bug fixes, and release tags. You refresh your local sources with `lengua update`.

### Refreshing all sources

```bash
lengua --store ./my-lib update
```

```
corp: fast-forwarded (a1b2c3d4..f6e5d4c3)
sales: up-to-date
project: fast-forwarded (9f8e7d6c..1a2b3c4d)
```

`lengua update` executes a **fast-forward only** update (equivalent to `git pull --ff-only`). It never discards history or silently resolves merge conflicts.

### Inspecting what changed

Before or after updating, inspect upstream history and line diffs:

```bash
# View recent commits for a template in a specific source
lengua --store ./my-lib log legal/nda.md --source corp

# Diff between two revisions or tags
lengua --store ./my-lib diff legal/nda.md v1.0 v2.0 --source corp
```

---

## Consuming monorepo slices (`--subdir`)

If your organization maintains several stores inside a single monorepo (e.g. `shared-repo/templates/marketing/` and `shared-repo/templates/legal/`), you can import just the slice you need:

```bash
lengua init --store ./my-lib --from-repo acme-org/monorepo/templates/marketing --name mktg
```

> [!NOTE]
> `--subdir` imports the current snapshot of that folder without carrying full git history. To update a `--subdir` source later, re-run `fetch --force`.

---

## CLI Scripting & AI Agent Workflows

### Scripting with `--json` and `jq`

Every `lengua` command supports `--json`. Stderr is reserved for warnings and diagnostics, ensuring stdout is always valid, pipeable JSON.

```bash
# Extract all template names in a source
lengua --store ./my-lib list --source corp --json | jq -r '.[].name'

# Render and capture output in a shell script
RENDERED=$(lengua --store ./my-lib get welcome.md --var name="Ada" --json | jq -r '.rendered')
```

### Distributing AI Agent skills (`lengua skills`)

If you use AI coding assistants (such as Claude Code, Cursor, or Gemini CLI), export lengua's bundled agent skills directly into your agent directory:

```bash
# Export skills for Claude Code / coding agents
lengua skills .claude/skills
```

This generates `SKILL.md` files that instruct AI agents how to query, render, and author templates in your repositories.

---

## See also

- [Storekeeper guide]({{ '/storekeeper-guide.html' | relative_url }}) — for managing and publishing organizational stores
- [Consuming an existing library]({{ '/consuming.html' | relative_url }}) — quick reference for `init` and `fetch`
- [Commands reference]({{ '/commands.html' | relative_url }}) — complete CLI flag specifications
