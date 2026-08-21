---
layout: default
title: FAQ
nav_order: 5
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
