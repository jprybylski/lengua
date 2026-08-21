use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

use crate::output::print_json;

#[derive(Serialize)]
struct SkillsOutput {
    directory: String,
    created: Vec<String>,
}

pub fn run(directory: &Path, force: bool, json: bool) -> Result<()> {
    let created: Vec<PathBuf> = lengua_core::export_skills(directory, force)?;
    let directory = directory.to_string_lossy().to_string();
    let created: Vec<String> = created
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    if json {
        print_json(&SkillsOutput { directory, created });
    } else {
        println!("wrote {} skill file(s) to {}", created.len(), directory);
        for path in &created {
            println!("  {path}");
        }
    }
    Ok(())
}
