use anyhow::Result;
use lengua_core::{DiffTag, Store, diff_text};
use serde::Serialize;

use crate::output::print_json;

#[derive(Serialize)]
struct DiffRow {
    tag: &'static str,
    line: String,
}

pub fn run(
    store_path: &std::path::Path,
    name: &str,
    from: &str,
    to: &str,
    json: bool,
) -> Result<()> {
    let store = Store::open(store_path)?;
    let old = store.read_at_revision(name, from)?;
    let new = store.read_at_revision(name, to)?;

    let rows: Vec<DiffRow> = diff_text(&old, &new)
        .into_iter()
        .map(|l| DiffRow {
            tag: match l.tag {
                DiffTag::Equal => "equal",
                DiffTag::Insert => "insert",
                DiffTag::Delete => "delete",
            },
            line: l.line,
        })
        .collect();

    if json {
        print_json(&rows);
    } else {
        for row in &rows {
            let prefix = match row.tag {
                "insert" => "+ ",
                "delete" => "- ",
                _ => "  ",
            };
            println!("{prefix}{}", row.line);
        }
    }
    Ok(())
}
