use anyhow::Result;
use lengua_core::Store;
use serde::Serialize;

use crate::output::print_json;

#[derive(Serialize)]
struct LogRow {
    commit: String,
    message: String,
}

pub fn run(store_path: &std::path::Path, name: &str, json: bool) -> Result<()> {
    let store = Store::open(store_path)?;
    let rows: Vec<LogRow> = store
        .log(name)?
        .into_iter()
        .map(|e| LogRow {
            commit: e.commit,
            message: e.message,
        })
        .collect();

    if json {
        print_json(&rows);
    } else if rows.is_empty() {
        println!("(no history)");
    } else {
        for row in &rows {
            println!(
                "{}  {}",
                &row.commit[..row.commit.len().min(12)],
                row.message
            );
        }
    }
    Ok(())
}
