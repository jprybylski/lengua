use anyhow::{Result, anyhow};
use lengua_core::{Library, Query};
use serde::Serialize;

use crate::cli::parse_kv_pairs;
use crate::output::{print_json, print_shadow_warnings};

#[derive(Serialize)]
struct SearchEntry {
    name: String,
    title: Option<String>,
    source: String,
}

pub fn run(
    store_path: &std::path::Path,
    fields: &[String],
    source: Option<String>,
    json: bool,
) -> Result<()> {
    let library = Library::open(store_path)?;
    if source.is_none() {
        print_shadow_warnings(library.shadow_warnings());
    }
    let field_pairs = parse_kv_pairs(fields).map_err(|e| anyhow!(e))?;

    let mut query = Query::new();
    for (k, v) in field_pairs {
        query = query.with(k, v);
    }

    let rows: Vec<SearchEntry> = library
        .search(source.as_deref(), &query)?
        .into_iter()
        .map(|(e, source)| SearchEntry {
            name: e.name,
            title: e.meta.title,
            source,
        })
        .collect();

    if json {
        print_json(&rows);
    } else if rows.is_empty() {
        println!("(no matches)");
    } else {
        for row in &rows {
            match &row.title {
                Some(title) => println!("{}\t{}\t[{}]", row.name, title, row.source),
                None => println!("{}\t[{}]", row.name, row.source),
            }
        }
    }
    Ok(())
}
