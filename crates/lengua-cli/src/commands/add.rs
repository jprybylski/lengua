use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use lengua_core::{Store, frontmatter};
use serde::Serialize;
use serde_yaml::Value;

use crate::cli::parse_kv_pairs;
use crate::output::print_json;

#[derive(Serialize)]
struct AddOutput<'a> {
    status: &'static str,
    name: &'a str,
    commit: String,
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    store_path: &std::path::Path,
    name: &str,
    file: Option<PathBuf>,
    title: Option<String>,
    fields: &[String],
    message: &str,
    json: bool,
) -> Result<()> {
    let store = Store::open(store_path)?;

    let input = match file {
        Some(path) => {
            std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?
        }
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .context("reading template body from stdin")?;
            buf
        }
    };

    // The input may already be a complete template (its own frontmatter +
    // body) if it was authored by hand or exported via `lengua get --raw`.
    // Parse it first so `--title`/`--field` merge into (and can override)
    // any frontmatter it already carries, instead of nesting a second block.
    let parsed = frontmatter::parse(&input)?;
    let mut meta = parsed.meta;
    if title.is_some() {
        meta.title = title;
    }
    let field_pairs = parse_kv_pairs(fields).map_err(|e| anyhow!(e))?;
    for (k, v) in field_pairs {
        meta.fields.insert(k, Value::String(v));
    }

    let commit = store.add(name, &meta, &parsed.body, message)?;

    if json {
        print_json(&AddOutput {
            status: "added",
            name,
            commit,
        });
    } else {
        println!("Added {name} ({commit})");
    }
    Ok(())
}
