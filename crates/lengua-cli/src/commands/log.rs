use anyhow::Result;
use lengua_core::Library;
use serde::Serialize;

use crate::output::{dim, print_json};

#[derive(Serialize)]
struct LogRow {
    commit: String,
    message: String,
}

pub fn run(
    store_path: &std::path::Path,
    name: &str,
    source: Option<String>,
    json: bool,
) -> Result<()> {
    let library = Library::open(store_path)?;
    let rows: Vec<LogRow> = library
        .log(source.as_deref(), name)?
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
            anstream::println!(
                "{}  {}",
                dim(&row.commit[..row.commit.len().min(12)]),
                row.message
            );
        }
    }
    Ok(())
}
