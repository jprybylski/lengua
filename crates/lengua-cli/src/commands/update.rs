use anyhow::{Result, anyhow};
use lengua_core::{Error, Library, UpdateStatus};
use serde::Serialize;

use crate::output::print_json;

#[derive(Serialize)]
struct UpdateRow {
    source: String,
    status: String,
    detail: Option<String>,
}

pub fn run(store_path: &std::path::Path, source: Option<String>, json: bool) -> Result<()> {
    let library = Library::open(store_path)?;
    let results = library.update(source.as_deref());

    let mut hard_failure = false;
    let rows: Vec<UpdateRow> = results
        .into_iter()
        .map(|(name, result)| match result {
            Ok(UpdateStatus::UpToDate) => UpdateRow {
                source: name,
                status: "up-to-date".to_string(),
                detail: None,
            },
            Ok(UpdateStatus::FastForwarded { from, to }) => UpdateRow {
                source: name,
                status: "fast-forwarded".to_string(),
                detail: Some(format!(
                    "{}..{}",
                    &from[..from.len().min(12)],
                    &to[..to.len().min(12)]
                )),
            },
            Err(err @ Error::NotFastForward { .. }) => {
                hard_failure = true;
                UpdateRow {
                    source: name,
                    status: "error".to_string(),
                    detail: Some(err.to_string()),
                }
            }
            Err(err) => UpdateRow {
                source: name,
                status: "not-updatable".to_string(),
                detail: Some(err.to_string()),
            },
        })
        .collect();

    if json {
        print_json(&rows);
    } else if rows.is_empty() {
        println!("(no sources)");
    } else {
        for row in &rows {
            match &row.detail {
                Some(detail) => println!("{}: {} ({detail})", row.source, row.status),
                None => println!("{}: {}", row.source, row.status),
            }
        }
    }

    if hard_failure {
        return Err(anyhow!("one or more sources failed to update"));
    }
    Ok(())
}
