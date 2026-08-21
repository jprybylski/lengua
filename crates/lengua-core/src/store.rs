use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use gix::objs::tree::EntryKind;

use crate::error::{Error, Result};
use crate::frontmatter;
use crate::meta::TemplateMeta;

const TEMPLATES_DIR: &str = "templates";

/// A git-backed library of templates rooted at a directory. Current-state
/// reads (`get`/`list`/`search`) go straight to the working tree; `add`
/// writes the file and commits it via gix; `log`/`diff` inspect git history.
pub struct Store {
    repo: gix::Repository,
    root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TemplateEntry {
    pub name: String,
    pub meta: TemplateMeta,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub commit: String,
    pub message: String,
}

impl Store {
    pub fn init(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        if root.join(".git").exists() {
            return Err(Error::AlreadyInitialized(root));
        }
        let templates_dir = root.join(TEMPLATES_DIR);
        std::fs::create_dir_all(&templates_dir).map_err(|source| Error::Io {
            path: templates_dir,
            source,
        })?;
        let repo = gix::init(&root).map_err(|e| Error::Git(e.to_string()))?;
        Ok(Self { repo, root })
    }

    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let repo = gix::open(&root).map_err(|e| Error::Git(e.to_string()))?;
        Ok(Self { repo, root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn templates_dir(&self) -> PathBuf {
        self.root.join(TEMPLATES_DIR)
    }

    fn abs_path(&self, name: &str) -> PathBuf {
        self.templates_dir().join(name)
    }

    fn git_path(&self, name: &str) -> String {
        format!("{TEMPLATES_DIR}/{name}")
    }

    /// Writes `name`'s frontmatter + body to the working tree and commits it.
    pub fn add(
        &self,
        name: &str,
        meta: &TemplateMeta,
        body: &str,
        message: &str,
    ) -> Result<String> {
        let text = frontmatter::write(meta, body)?;
        let path = self.abs_path(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| Error::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        std::fs::write(&path, &text).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        self.commit_file(name, &text, message)
    }

    fn commit_file(&self, name: &str, text: &str, message: &str) -> Result<String> {
        let repo = &self.repo;
        let base_tree = repo
            .head_tree_id_or_empty()
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();
        let mut editor = repo
            .edit_tree(base_tree)
            .map_err(|e| Error::Git(e.to_string()))?;
        let blob_id = repo
            .write_blob(text.as_bytes())
            .map_err(|e| Error::Git(e.to_string()))?;
        editor
            .upsert(
                self.git_path(name).as_str(),
                EntryKind::Blob,
                blob_id.detach(),
            )
            .map_err(|e| Error::Git(e.to_string()))?;
        let tree_id = editor.write().map_err(|e| Error::Git(e.to_string()))?;
        let parents: Vec<gix::ObjectId> = match repo.head_id() {
            Ok(id) => vec![id.detach()],
            Err(_) => Vec::new(),
        };
        let commit_id = repo
            .commit("HEAD", message, tree_id.detach(), parents)
            .map_err(|e| Error::Git(e.to_string()))?;
        Ok(commit_id.detach().to_string())
    }

    /// Reads and parses a template straight from the working tree.
    pub fn get(&self, name: &str) -> Result<TemplateEntry> {
        let path = self.abs_path(name);
        let text = std::fs::read_to_string(&path).map_err(|_| Error::NotFound(name.to_string()))?;
        let parsed = frontmatter::parse(&text)?;
        Ok(TemplateEntry {
            name: name.to_string(),
            meta: parsed.meta,
            body: parsed.body,
        })
    }

    /// Lists every template currently in the working tree, sorted by name.
    pub fn list(&self) -> Result<Vec<TemplateEntry>> {
        let dir = self.templates_dir();
        let mut names = BTreeSet::new();
        collect_names(&dir, &dir, &mut names)?;
        names.into_iter().map(|n| self.get(&n)).collect()
    }

    /// Returns the commit history that touched `name`, newest first,
    /// collapsing consecutive commits where the file's content didn't
    /// change (a first-parent path simplification, not full git history
    /// simplification).
    pub fn log(&self, name: &str) -> Result<Vec<LogEntry>> {
        let rel = self.git_path(name);
        let repo = &self.repo;
        let head = match repo.head_id() {
            Ok(id) => id,
            Err(_) => return Ok(Vec::new()),
        };

        let mut entries = Vec::new();
        let mut last_blob: Option<gix::ObjectId> = None;
        let walk = head
            .ancestors()
            .all()
            .map_err(|e| Error::Git(e.to_string()))?;
        for info in walk {
            let info = info.map_err(|e| Error::Git(e.to_string()))?;
            let commit = info
                .id()
                .object()
                .map_err(|e| Error::Git(e.to_string()))?
                .into_commit();
            let tree = commit.tree().map_err(|e| Error::Git(e.to_string()))?;
            let blob_id = tree
                .lookup_entry_by_path(&rel)
                .map_err(|e| Error::Git(e.to_string()))?
                .map(|entry| entry.id().detach());

            if blob_id.is_none() {
                continue;
            }
            if blob_id != last_blob {
                let message = commit
                    .message()
                    .map(|m| m.title.to_string())
                    .unwrap_or_default();
                entries.push(LogEntry {
                    commit: info.id().to_string(),
                    message,
                });
                last_blob = blob_id;
            }
        }
        Ok(entries)
    }

    /// Reads `name`'s full frontmatter+body text as it existed at `rev`
    /// (any revspec gix's `rev_parse_single` accepts, e.g. `HEAD`, `HEAD~1`,
    /// or a commit id).
    pub fn read_at_revision(&self, name: &str, rev: &str) -> Result<String> {
        let rel = self.git_path(name);
        let repo = &self.repo;
        let id = repo
            .rev_parse_single(rev)
            .map_err(|e| Error::InvalidRevision {
                name: name.to_string(),
                rev: rev.to_string(),
                reason: e.to_string(),
            })?;
        let commit = id
            .object()
            .map_err(|e| Error::Git(e.to_string()))?
            .try_into_commit()
            .map_err(|e| Error::InvalidRevision {
                name: name.to_string(),
                rev: rev.to_string(),
                reason: e.to_string(),
            })?;
        let tree = commit.tree().map_err(|e| Error::Git(e.to_string()))?;
        let entry = tree
            .lookup_entry_by_path(&rel)
            .map_err(|e| Error::Git(e.to_string()))?
            .ok_or_else(|| Error::NotFound(format!("{name} at {rev}")))?;
        let data = entry
            .id()
            .object()
            .map_err(|e| Error::Git(e.to_string()))?
            .data
            .clone();
        String::from_utf8(data).map_err(|e| Error::Git(e.to_string()))
    }
}

fn collect_names(root: &Path, dir: &Path, names: &mut BTreeSet<String>) -> Result<()> {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(()),
    };
    for entry in read_dir {
        let entry = entry.map_err(|source| Error::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_names(root, &path, names)?;
        } else if let Ok(rel) = path.strip_prefix(root)
            && let Some(name) = rel.to_str()
        {
            names.insert(name.to_string());
        }
    }
    Ok(())
}
