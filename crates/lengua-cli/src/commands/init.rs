use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use lengua_core::Library;
use serde::Serialize;

use crate::from_repo;
use crate::output::{heading, print_json};

#[derive(Serialize)]
struct InitOutput<'a> {
    status: &'static str,
    path: &'a str,
    source: String,
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    store_path: &Path,
    from_dir: Option<PathBuf>,
    from_repo: Option<String>,
    git_ref: Option<String>,
    subdir: Option<String>,
    name: Option<String>,
    force: bool,
    json: bool,
) -> Result<()> {
    let (status, source) = if from_dir.is_some() || from_repo.is_some() {
        let (from_repo_url, parsed_ref, parsed_subdir) = match &from_repo {
            Some(spec) => {
                let parsed = from_repo::parse(spec).map_err(|e| anyhow!(e))?;
                (Some(parsed.url), parsed.git_ref, parsed.subdir)
            }
            None => (None, None, None),
        };
        let git_ref = git_ref.or(parsed_ref);
        let subdir = subdir.or(parsed_subdir);
        let library = Library::init_from(
            store_path,
            name.as_deref(),
            from_dir.as_deref(),
            from_repo_url.as_deref(),
            git_ref.as_deref(),
            subdir.as_deref(),
            force,
        )?;
        ("adopted", library.manifest_order().remove(0))
    } else {
        let library = Library::init(store_path, name.as_deref())?;
        ("initialized", library.manifest_order().remove(0))
    };

    let path = store_path.to_string_lossy();
    if json {
        print_json(&InitOutput {
            status,
            path: &path,
            source,
        });
    } else if status == "adopted" {
        anstream::println!(
            "{}",
            heading(&format!(
                "Adopted lengua library at {path} (source '{source}')"
            ))
        );
    } else {
        anstream::println!(
            "{}",
            heading(&format!(
                "Initialized empty lengua library at {path} (source '{source}')"
            ))
        );
    }
    Ok(())
}
