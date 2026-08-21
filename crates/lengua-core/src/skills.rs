//! Bundled coding-agent skill files this crate exports (`lengua skills`, `lenguar`'s
//! `lq_export_skills()`). Hand-authored `SKILL.md` content under `../skills/` — this module is
//! just a thin, format-contract-tested exporter, not a generator; see the tests below for what
//! the checked-in format contract actually verifies.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

const SKILLS: &[(&str, &str)] = &[(
    "lengua-template-authoring",
    include_str!("../skills/lengua-template-authoring/SKILL.md"),
)];

/// Copies every bundled `SKILL.md` into `directory/<skill-name>/SKILL.md`, refusing to
/// overwrite an existing file unless `force` is set. Returns the paths written.
pub fn export_skills(directory: &Path, force: bool) -> Result<Vec<PathBuf>> {
    let targets: Vec<(PathBuf, &str)> = SKILLS
        .iter()
        .copied()
        .map(|(name, content)| (directory.join(name).join("SKILL.md"), content))
        .collect();

    if !force {
        let conflicts: Vec<PathBuf> = targets
            .iter()
            .map(|(path, _)| path)
            .filter(|p| p.exists())
            .cloned()
            .collect();
        if !conflicts.is_empty() {
            return Err(Error::SkillAlreadyExists { paths: conflicts });
        }
    }

    for (target, content) in &targets {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        fs::write(target, *content).map_err(|e| Error::Io {
            path: target.clone(),
            source: e,
        })?;
    }

    Ok(targets.into_iter().map(|(path, _)| path).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontmatter;

    #[test]
    fn every_skill_name_has_a_checked_in_directory() {
        for (name, _) in SKILLS.iter().copied() {
            let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("skills").join(name);
            assert!(dir.join("SKILL.md").is_file(), "missing {}", dir.display());
        }
    }

    #[test]
    fn frontmatter_name_matches_the_containing_directory() {
        for (name, content) in SKILLS.iter().copied() {
            let parsed = frontmatter::parse(content).unwrap();
            let got = parsed.meta.fields.get("name").and_then(|v| v.as_str());
            assert_eq!(got, Some(name));
        }
    }

    #[test]
    fn frontmatter_description_is_a_non_empty_string() {
        for (_, content) in SKILLS.iter().copied() {
            let parsed = frontmatter::parse(content).unwrap();
            let description = parsed
                .meta
                .fields
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            assert!(!description.trim().is_empty());
        }
    }

    #[test]
    fn body_is_non_empty() {
        for (_, content) in SKILLS.iter().copied() {
            let parsed = frontmatter::parse(content).unwrap();
            assert!(!parsed.body.trim().is_empty());
        }
    }

    #[test]
    fn export_writes_every_skill_and_refuses_to_overwrite_without_force() {
        let dir = tempfile::tempdir().unwrap();
        let written = export_skills(dir.path(), false).unwrap();
        assert_eq!(written.len(), SKILLS.len());
        for path in &written {
            assert!(path.is_file());
        }

        let err = export_skills(dir.path(), false).unwrap_err();
        assert!(matches!(err, Error::SkillAlreadyExists { .. }));

        // force overwrites cleanly
        export_skills(dir.path(), true).unwrap();
    }
}
