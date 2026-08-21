//! `.lengua/sources.toml`: the ordered list of a library's named sources.
//!
//! Deliberately not a lockfile — no commit id is ever recorded here. Each source's own
//! nested git repo (`.lengua/<name>/.git`) is the sole authority on what commit it's at;
//! this file only records *what sources exist*, in what order (fetch order doubles as
//! merge-precedence order — see [`crate::library::Library`]), and how each was adopted
//! (needed by `update` to pick a fetch strategy per [`crate::source::AdoptionMechanism`]).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub(crate) const MANIFEST_FILE: &str = "sources.toml";
const MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SourcesManifest {
    pub version: u32,
    #[serde(rename = "source", default)]
    pub sources: Vec<SourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SourceEntry {
    pub name: String,
    #[serde(flatten)]
    pub kind: SourceKind,
}

/// How a source's content was adopted, which determines how [`crate::source::update_cloned`]/
/// [`crate::source::update_copied`] fetch new content for it. Recorded by *mechanism*, not by
/// which CLI flag was used: `--from-repo` naming a local/`file://` path with no explicit
/// `--ref` takes the same safe filesystem-copy path as `--from-dir` (see
/// `crate::source::clone_or_copy`), so it must be tracked as `Copied` too, not `Cloned`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub(crate) enum SourceKind {
    /// Created by plain `init` with no adoption flags: an empty, writable repo with no
    /// origin at all. Never updatable (there's nothing to fetch from).
    Local,
    /// Adopted via a raw filesystem copy (`crate::source::copy_local_dir`).
    Copied {
        origin: PathBuf,
        #[serde(skip_serializing_if = "Option::is_none")]
        subdir: Option<String>,
    },
    /// Adopted via a real `gix` transport clone (`crate::source::clone_via_transport`).
    Cloned {
        url: String,
        #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
        git_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        subdir: Option<String>,
    },
}

impl SourcesManifest {
    pub fn new() -> Self {
        Self {
            version: MANIFEST_VERSION,
            sources: Vec::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&text).map_err(|e| Error::SourcesManifest {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let text = toml::to_string_pretty(self).map_err(|e| Error::SourcesManifest {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        std::fs::write(path, text).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Appends a new source, becoming the new highest-precedence entry. Errors if `name` is
    /// already taken.
    pub fn push(&mut self, entry: SourceEntry) -> Result<()> {
        if self.sources.iter().any(|s| s.name == entry.name) {
            return Err(Error::DuplicateSourceName { name: entry.name });
        }
        self.sources.push(entry);
        Ok(())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.sources.iter().any(|s| s.name == name)
    }
}

pub(crate) fn manifest_path(lengua_dir: &Path) -> PathBuf {
    lengua_dir.join(MANIFEST_FILE)
}
