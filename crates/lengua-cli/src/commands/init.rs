use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use lengua_core::Store;
use serde::Serialize;

use crate::from_repo;
use crate::output::{heading, print_json};

#[derive(Serialize)]
struct InitOutput<'a> {
    status: &'static str,
    path: &'a str,
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    store_path: &Path,
    from_dir: Option<PathBuf>,
    from_repo: Option<String>,
    git_ref: Option<String>,
    subdir: Option<String>,
    force: bool,
    json: bool,
) -> Result<()> {
    let (status, path) = if let Some(from_dir) = from_dir {
        Store::init_from_dir(store_path, &from_dir, subdir.as_deref(), force)?;
        ("adopted", store_path.to_string_lossy())
    } else if let Some(from_repo) = from_repo {
        let parsed = from_repo::parse(&from_repo).map_err(|e| anyhow!(e))?;
        let git_ref = git_ref.or(parsed.git_ref);
        let subdir = subdir.or(parsed.subdir);
        Store::init_from_repo(
            store_path,
            &parsed.url,
            git_ref.as_deref(),
            subdir.as_deref(),
            force,
        )?;
        ("adopted", store_path.to_string_lossy())
    } else {
        Store::init(store_path)?;
        ("initialized", store_path.to_string_lossy())
    };

    if json {
        print_json(&InitOutput {
            status,
            path: &path,
        });
    } else if status == "adopted" {
        anstream::println!("{}", heading(&format!("Adopted lengua store at {path}")));
    } else {
        anstream::println!(
            "{}",
            heading(&format!("Initialized empty lengua store at {path}"))
        );
    }
    Ok(())
}
