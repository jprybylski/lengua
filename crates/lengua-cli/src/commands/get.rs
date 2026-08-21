use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use lengua_core::{Store, template};
use serde::Serialize;

use crate::cli::parse_kv_pairs;
use crate::output::print_json;

#[derive(Serialize)]
struct GetOutput<'a> {
    name: &'a str,
    rendered: String,
}

pub fn run(
    store_path: &std::path::Path,
    name: &str,
    vars: &[String],
    raw: bool,
    rev: Option<String>,
    json: bool,
) -> Result<()> {
    let store = Store::open(store_path)?;
    let entry = match &rev {
        Some(rev) => store.get_at_revision(name, rev)?,
        None => store.get(name)?,
    };

    let rendered = if raw {
        entry.body.clone()
    } else {
        let var_pairs = parse_kv_pairs(vars).map_err(|e| anyhow!(e))?;
        let ctx: BTreeMap<String, String> = var_pairs.into_iter().collect();
        template::render(&entry.body, &ctx)?
    };

    if json {
        print_json(&GetOutput { name, rendered });
    } else {
        println!("{rendered}");
    }
    Ok(())
}
