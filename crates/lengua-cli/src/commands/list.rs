use anyhow::Result;
use lengua_core::Store;
use serde::Serialize;

use crate::output::print_json;

#[derive(Serialize)]
struct ListEntry {
    name: String,
    title: Option<String>,
}

pub fn run(store_path: &std::path::Path, json: bool) -> Result<()> {
    let store = Store::open(store_path)?;
    let entries = store.list()?;

    let rows: Vec<ListEntry> = entries
        .into_iter()
        .map(|e| ListEntry {
            name: e.name,
            title: e.meta.title,
        })
        .collect();

    if json {
        print_json(&rows);
    } else if rows.is_empty() {
        println!("(no templates)");
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
