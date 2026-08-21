Template bodies are rendered by [`render`], which is a bare
`minijinja::Environment::new()` plus `render_str` — no custom filters, functions, or tests are
registered, and no autoescaping is configured (minijinja only auto-escapes for `.html`/`.xml`
template *names*, and `render_str` uses none). Every feature described below is minijinja's own
stock Jinja2-compatible syntax, not something lengua opts into or restricts — see
[minijinja's own documentation](https://docs.rs/minijinja/) for the authoritative, complete
reference.

# Interpolation

```text
Dear {{ name }},
```

Unset variables are left as their default `undefined` behavior (minijinja renders them as an
empty string) rather than erroring, unless the template supplies its own default (see Filters,
below).

# Filters

```text
{{ name | upper }}
{{ name | default("there") }}
{{ items | join(", ") }}
```

# Conditionals

```text
{% if tone == "formal" %}Dear{% else %}Hi{% endif %} {{ name }},
```

# Loops

```text
{% for item in items %}{{ item }}{% endfor %}
```

# Arithmetic and comparison

```text
{{ count + 1 }}
{{ price * qty }}
{{ n > 10 }}
```

# Escaping

There is no automatic HTML escaping — rendered output is plain text, not HTML, so `{{ x }}`
never gets HTML-entity-encoded. Two escaping needs are worth telling apart:

- To emit a *literal* `{{ }}` in rendered output (rather than having it interpreted as
  interpolation), use minijinja's raw-text block tag — see its docs for the exact syntax.
- To HTML-escape a value for some other reason (e.g. the rendered text itself will later be
  embedded in HTML), use the `| escape` filter explicitly; it's never applied automatically.

# What this means for `--field`/frontmatter authors

Frontmatter fields referenced in a template body (via [`crate::meta::TemplateMeta`]'s
`fields`) aren't passed to `render` automatically — the caller (`lengua get --var
key=value`, or the equivalent FFI call) decides what context a render sees. A field's *value*
being present in frontmatter doesn't by itself make it a template variable.
