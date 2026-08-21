use anyhow::{Result, anyhow};
use lengua_core::{Query, Store};
use serde::Serialize;

use crate::cli::parse_kv_pairs;
use crate::output::print_json;

#[derive(Serialize)]
struct SearchEntry {
    name: String,
    title: Option<String>,
}

pub fn run(store_path: &std::path::Path, fields: &[String], json: bool) -> Result<()> {
    let store = Store::open(store_path)?;
    let field_pairs = parse_kv_pairs(fields).map_err(|e| anyhow!(e))?;

    let mut query = Query::new();
    for (k, v) in field_pairs {
        query = query.with(k, v);
    }

    let rows: Vec<SearchEntry> = store
        .list()?
        .into_iter()
        .filter(|e| query.matches(&e.meta))
        .map(|e| SearchEntry {
            name: e.name,
            title: e.meta.title,
        })
        .collect();

    if json {
        print_json(&rows);
    } else if rows.is_empty() {
        println!("(no matches)");
    } else {
        for row in &rows {
            match &row.title {
                Some(title) => println!("{}\t{}", row.name, title),
                None => println!("{}", row.name),
            }
        }
    }
    Ok(())
}
