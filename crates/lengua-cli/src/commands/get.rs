use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use lengua_core::{Library, template};
use serde::Serialize;

use crate::cli::parse_kv_pairs;
use crate::output::{print_json, print_shadow_warnings};

#[derive(Serialize)]
struct GetOutput<'a> {
    name: &'a str,
    source: &'a str,
    rendered: String,
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    store_path: &std::path::Path,
    name: &str,
    vars: &[String],
    raw: bool,
    rev: Option<String>,
    source: Option<String>,
    json: bool,
) -> Result<()> {
    let library = Library::open(store_path)?;
    if source.is_none() {
        print_shadow_warnings(library.shadow_warnings());
    }
    let (entry, resolved_source) = library.get(source.as_deref(), name, rev.as_deref())?;

    let rendered = if raw {
        entry.body.clone()
    } else {
        let var_pairs = parse_kv_pairs(vars).map_err(|e| anyhow!(e))?;
        let ctx: BTreeMap<String, String> = var_pairs.into_iter().collect();
        template::render(&entry.body, &ctx)?
    };

    if json {
        print_json(&GetOutput {
            name,
            source: &resolved_source,
            rendered,
        });
    } else {
        println!("{rendered}");
    }
    Ok(())
}
