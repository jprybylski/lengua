use anyhow::Result;
use lengua_core::Library;
use serde::Serialize;

use crate::output::{print_json, print_shadow_warnings};

#[derive(Serialize)]
struct ListEntry {
    name: String,
    title: Option<String>,
    source: String,
}

pub fn run(store_path: &std::path::Path, source: Option<String>, json: bool) -> Result<()> {
    let library = Library::open(store_path)?;
    if source.is_none() {
        print_shadow_warnings(library.shadow_warnings());
    }
    let entries = library.list(source.as_deref())?;

    let rows: Vec<ListEntry> = entries
        .into_iter()
        .map(|(e, source)| ListEntry {
            name: e.name,
            title: e.meta.title,
            source,
        })
        .collect();

    if json {
        print_json(&rows);
    } else if rows.is_empty() {
        println!("(no templates)");
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
