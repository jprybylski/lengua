use anyhow::Result;
use lengua_core::{DiffTag, Library, diff_text};
use serde::Serialize;

use crate::output::{deleted, inserted, print_json};

#[derive(Serialize)]
struct DiffRow {
    tag: &'static str,
    line: String,
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    store_path: &std::path::Path,
    name: &str,
    from: &str,
    to: &str,
    source: Option<String>,
    json: bool,
) -> Result<()> {
    let library = Library::open(store_path)?;
    let store = library.resolve_source(source.as_deref())?;
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
            match row.tag {
                "insert" => anstream::println!("{}", inserted(&format!("+ {}", row.line))),
                "delete" => anstream::println!("{}", deleted(&format!("- {}", row.line))),
                _ => println!("  {}", row.line),
            }
        }
    }
    Ok(())
}
