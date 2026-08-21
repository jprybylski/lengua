---
name: lengua-template-authoring
description: Use this skill when a user wants to build, organize, or grow a lengua template library — requests like "help me set up a template library for X", "what fields should this template have", "turn this into a reusable template", or "how do I find/render a template later". Covers designing frontmatter fields, writing sources rich enough to generate good templates from, questions to ask before generating one, and how to actually render a template once it exists (via the `lengua` CLI, or `lenguar`'s equivalent R functions).
---

# Authoring a lengua template library

lengua stores each template as one file with YAML frontmatter plus a
[minijinja](https://github.com/mitsuhiko/minijinja) body, versioned by git. This skill is about
the authoring workflow around that — designing a library's frontmatter fields, writing enough
source material to draft a good template from, and rendering it back out later — not the CLI's
mechanics, which are covered in `docs/commands.md` in the `lengua` repo.

## 1. Organize with frontmatter fields, not directory-encoded taxonomy

`title` is the only frontmatter field lengua itself knows about. Every other field is an
arbitrary `key: value` (or `key: [a, b]` list) the library's author defines — `search --field
key=value` is what makes them useful, not any fixed schema. Two habits keep a library
searchable instead of just browsable:

- Don't duplicate a template's directory/path in its own frontmatter (a `letters/thank-you.md`
  template doesn't need a `type: letter` field — that's already in the name). Reserve fields
  for things that cut *across* the directory structure: `tone`, `audience`, `jurisdiction`,
  `language`, whatever actually varies within a category.
- Prefer a small, consistent field vocabulary reused across templates over a different set of
  ad hoc fields per template — `search --field tone=formal` is only useful if more than one
  template actually sets `tone`.

`lengua tag` is a *different* feature — named pointers at a specific git revision of one
template (`refs/lengua/tags/<template>/<tag>`), not a frontmatter field. Don't reach for a
`tags:` frontmatter field to mean "milestones of this template's history" — that's what
`lengua tag add` is for.

## 2. What makes a good writing source to supply

Before generating a template from a writing sample, richer source material produces a template
that generalizes better than a single example does:

- More than one real instance of the text, if available — generating from a single email
  makes it easy to over-fit incidental details (a specific name, date, or number) into the
  template instead of the actual variable.
- The *range* of variation across instances: what changes every time (name, date, amount) vs.
  what's structurally fixed (the greeting, the closing, the legal boilerplate).
  Freely-varying pieces become `{{ variables }}`; structurally-fixed pieces stay literal text.
- Any existing frontmatter-worthy metadata already implicit in the source (a formal vs. casual
  register, an intended audience, a jurisdiction) — these become candidate frontmatter fields
  rather than getting baked into the template body as static text.

## 3. Questions to clarify with the user before generating a template

- Which parts of the source text are meant to vary per-use (become `{{ variables }}`), and
  which are fixed boilerplate that should render identically every time?
- What should an unset variable do — render as empty, or should the template supply a
  `| default(...)`?
- What frontmatter fields does this template need to be findable by later, given the library's
  existing field vocabulary (reuse existing field names where the meaning matches, rather than
  inventing a near-duplicate)?
- Does any variable need light logic — a conditional greeting, a pluralized count, a loop over
  a list — or is it pure interpolation? (See the templating guide referenced below for what
  minijinja actually supports.)

## 4. Rendering a template once it exists

CLI (`lengua` repo):

```bash
lengua get letters/thank-you.md --var name=Ada --var reason="the review"
lengua search --field tone=formal
```

R (`lenguar` package, same underlying library via FFI — no shelling out to the `lengua`
binary):

```r
lq_get("letters/thank-you.md", vars = c(name = "Ada", reason = "the review"))
lq_search(fields = c(tone = "formal"))
```

Both read from the same on-disk `.lengua/` library — pick whichever fits the calling
environment.

## 5. Template syntax itself

This skill is about library/template *authoring*, not template syntax. As of this writing, no
dedicated "minijinja" coding-agent skill is known to exist publicly — for the full templating
syntax (conditionals, loops, filters, escaping), see lengua-core's own rustdoc templating guide
(`crates/lengua-core/src/template.rs`'s module docs, published at
https://docs.rs/lengua-core/latest/lengua_core/template/) or minijinja's own documentation
directly.
