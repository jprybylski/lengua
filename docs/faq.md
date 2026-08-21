---
layout: default
title: FAQ
nav_order: 6
---

# FAQ
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Is lengua a project scaffolder / generator?

No. `lengua init` creates a new template-*library* git repo (a place to store and version
templates), not an application skeleton. If you want project scaffolding, tools like
`cargo generate` or `cookiecutter` solve a different problem.

## Does lengua require a network connection or an LLM?

No. lengua has no LLM integration and does no network I/O at all — it's a local git-backed
store plus a Jinja-style renderer. `--json` output is designed so that *you* can wire it up to
an LLM/agent if you want to, but lengua itself never calls out to one.

## What templating syntax does `get` support?

[minijinja](https://github.com/mitsuhiko/minijinja)'s Jinja2-compatible syntax: `{{ variable }}`
interpolation, filters (`{{ name | upper }}`), defaults (`{{ name | default("there") }}`),
conditionals, and loops. See minijinja's documentation for the full syntax.

## Can I use lengua from R?

Yes — see [lenguar](https://github.com/jprybylski/lenguar), a companion R package with FFI
(extendr) bindings to `lengua-core`, so it doesn't shell out to the `lengua` binary.

## Why doesn't `search` support a real query language?

It's a deliberate v1 simplification — see
[Architecture: What was deliberately left out]({{ '/architecture.html#what-was-deliberately-left-out' | relative_url }}).

## Where is a template's history actually stored?

In the store's own `.git` directory — `log`/`diff` walk real git commits made by `add`, they
aren't a separate change-tracking mechanism. You can `cd` into a store and run ordinary `git
log -- templates/<name>` and see the same history.

## How is a tag different from a git tag?

`git tag` creates a ref under `refs/tags/<name>` — one namespace shared by the whole repo, so
you can't have two different templates each with their own `"final"` tag pointing at different
commits. lengua's tags live under `refs/lengua/tags/<template>/<tag>` instead: a separate
namespace, scoped per template, that `lengua tag`/`get --rev`/`diff` read and write directly via
`gix` (never `git tag` itself). They show up in `git for-each-ref refs/lengua/` if you want to
inspect them with plain git, but ordinary `git tag -l` won't list them.

## What happens if I run lengua inside my own project's git repo?

`--store` defaults to the current directory, and lengua opens whatever `.git` it finds there —
so running a command with no `--store` from inside a directory that's *already* your own
project's git repo, but isn't a lengua store, is a real risk: without a check, lengua would
happily commit `templates/...` blobs straight into your project's history.

lengua guards against this: every command except `init` refuses to operate on a directory that
doesn't already have a `templates/` folder, with an error telling you to pass `--store` or run
`init` first. The guard is a heuristic (a directory that happens to have an unrelated top-level
`templates/` folder would slip past it), so the safest layout is still to keep your template
store in its own dedicated directory — a sibling of your project, not nested inside it — and
always pass `--store` explicitly if it *is* nested, rather than relying on the default `.`.
