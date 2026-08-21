# lengua

A git-backed CLI for managing a library of templated text: plain text, Markdown,
LaTeX, or code snippets, tagged with YAML/TOML frontmatter and rendered with
[minijinja](https://github.com/mitsuhiko/minijinja) (Jinja2-style) variables.

`lengua` is a persistent, queryable library of atomic templates — not a
project scaffolder. Every template lives as a file with frontmatter under a
`.lengua/<source>/templates/` directory, versioned by an ordinary git repository the tool
manages for you — a library can pool more than one such source together.

## Status

Early, pre-release. The core engine and CLI (`lengua-core` / `lengua-cli`) are
implemented and tested; an R package (`lenguar`) with extendr/FFI bindings, a
documentation site, and CI/release automation are planned but not yet built.

## Install

Requires a Rust toolchain ([rustup](https://rustup.rs)).

```sh
cargo install --path crates/lengua-cli
```

## Usage

```sh
# Create a new template library
lengua init my-templates
cd my-templates

# Add a template (frontmatter fields via --field, body from a file or stdin)
lengua add letters/thank-you.md \
  --title "Thank You Letter" \
  --field tone=formal \
  --file thank-you.md.tmpl

# Render it with variables substituted
lengua get letters/thank-you.md --var name=Ada --var reason="the gift"

# List everything in the library
lengua list

# Filter by frontmatter field
lengua search --field tone=formal

# Inspect history / diff two revisions
lengua log letters/thank-you.md
lengua diff letters/thank-you.md HEAD~1 HEAD
```

Every subcommand accepts `--json` for structured, script- and agent-friendly
output.

## Workspace layout

- `crates/lengua-core` — the engine: templating, frontmatter, git-backed
  storage/query. No CLI or R dependencies.
- `crates/lengua-cli` — the `lengua` binary (`clap`-based), depends only on
  `lengua-core`.

## Development

```sh
just build   # cargo build --workspace
just test    # cargo test --workspace
just lint    # cargo clippy --workspace -- -D warnings
just fmt     # cargo fmt --all
```
