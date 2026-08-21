use anyhow::Result;
use lengua_core::Store;
use serde::Serialize;

use crate::output::{dim, print_json, tag_name};

#[derive(Serialize)]
struct TagRow {
    tag: String,
    commit: String,
}

pub fn add(
    store_path: &std::path::Path,
    template: &str,
    tag: &str,
    rev: Option<String>,
    force: bool,
    json: bool,
) -> Result<()> {
    let store = Store::open(store_path)?;
    let entry = store.tag_create(template, tag, rev.as_deref(), force)?;
    let row = TagRow {
        tag: entry.tag,
        commit: entry.commit,
    };
    if json {
        print_json(&row);
    } else {
        anstream::println!(
            "Tagged {template} @ {} as {}",
            dim(&row.commit[..row.commit.len().min(12)]),
            tag_name(&row.tag)
        );
    }
    Ok(())
}

pub fn list(store_path: &std::path::Path, template: &str, json: bool) -> Result<()> {
    let store = Store::open(store_path)?;
    let rows: Vec<TagRow> = store
        .tag_list(template)?
        .into_iter()
        .map(|e| TagRow {
            tag: e.tag,
            commit: e.commit,
        })
        .collect();

    if json {
        print_json(&rows);
    } else if rows.is_empty() {
        println!("(no tags)");
    } else {
        for row in &rows {
            anstream::println!(
                "{}  {}",
                tag_name(&row.tag),
                dim(&row.commit[..row.commit.len().min(12)])
            );
        }
    }
    Ok(())
}

pub fn rm(store_path: &std::path::Path, template: &str, tag: &str, json: bool) -> Result<()> {
    let store = Store::open(store_path)?;
    store.tag_remove(template, tag)?;
    if json {
        print_json(&serde_json::json!({ "status": "removed", "template": template, "tag": tag }));
    } else {
        println!("Removed tag {tag} from {template}");
    }
    Ok(())
}
