use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use lengua_core::Library;
use serde::Serialize;

use crate::from_repo;
use crate::output::{heading, print_json, warning};

#[derive(Serialize)]
struct FetchOutput<'a> {
    status: &'static str,
    source: &'a str,
    warnings: Vec<String>,
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
    let mut library =
        Library::open(store_path).map_err(|e| anyhow!("{e} — run `lengua init` here first"))?;

    let (from_repo_url, parsed_ref, parsed_subdir) = match &from_repo {
        Some(spec) => {
            let parsed = from_repo::parse(spec).map_err(|e| anyhow!(e))?;
            (Some(parsed.url), parsed.git_ref, parsed.subdir)
        }
        None => (None, None, None),
    };
    let git_ref = git_ref.or(parsed_ref);
    let subdir = subdir.or(parsed_subdir);

    let outcome = library.fetch(
        name.as_deref(),
        from_dir.as_deref(),
        from_repo_url.as_deref(),
        git_ref.as_deref(),
        subdir.as_deref(),
        force,
    )?;

    let warning_lines: Vec<String> = outcome
        .warnings
        .iter()
        .map(|w| {
            format!(
                "'{}' is now shadowed by '{}' (also defined in '{}')",
                w.name, w.winner, w.loser
            )
        })
        .collect();
    for line in &warning_lines {
        anstream::eprintln!("{} {line}", warning("warning:"));
    }

    if json {
        print_json(&FetchOutput {
            status: "fetched",
            source: &outcome.source,
            warnings: warning_lines,
        });
    } else {
        anstream::println!(
            "{}",
            heading(&format!("Fetched source '{}'", outcome.source))
        );
    }
    Ok(())
}
