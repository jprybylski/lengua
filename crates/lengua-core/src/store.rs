use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use gix::objs::tree::EntryKind;
use gix::refs::transaction::{Change, PreviousValue, RefEdit, RefLog};

use crate::error::{Error, Result};
use crate::frontmatter;
use crate::meta::TemplateMeta;
use crate::tags::{self, TagEntry};

pub(crate) const TEMPLATES_DIR: &str = "templates";

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
        if !root.join(TEMPLATES_DIR).is_dir() {
            return Err(Error::NotAStore(root));
        }
        Ok(Self { repo, root })
    }

    /// Initializes a new store at `dest` by adopting an existing one from a
    /// local directory `source_dir`. If `subdir` is `None`, `source_dir` is
    /// copied as-is: full history and every `refs/lengua/tags/*` ref is
    /// preserved. If `subdir` is given, only the templates *currently* in
    /// that subdirectory of `source_dir` are imported as fresh commits —
    /// the source's original history, authorship, and any tags under that
    /// subdirectory are intentionally not preserved (there's no `gix`
    /// equivalent of `git subtree split` to do that losslessly).
    pub fn init_from_dir(
        dest: impl AsRef<Path>,
        source_dir: impl AsRef<Path>,
        subdir: Option<&str>,
        force: bool,
    ) -> Result<Self> {
        Self::init_from(
            dest.as_ref(),
            crate::source::Source::Dir(source_dir.as_ref()),
            None,
            subdir,
            force,
        )
    }

    /// Initializes a new store at `dest` by cloning an existing one from a
    /// remote git `url` (anything `gix` accepts: `https://…`, `git@…:…`,
    /// `ssh://…`, `file://…`). `git_ref` selects a branch/tag to check out
    /// (not a commit id — see [`source::clone_or_copy`](crate::source));
    /// `None` uses the remote's default branch. See [`Store::init_from_dir`]
    /// for what `subdir` preserves and discards.
    pub fn init_from_repo(
        dest: impl AsRef<Path>,
        url: &str,
        git_ref: Option<&str>,
        subdir: Option<&str>,
        force: bool,
    ) -> Result<Self> {
        Self::init_from(
            dest.as_ref(),
            crate::source::Source::Url(url),
            git_ref,
            subdir,
            force,
        )
    }

    fn init_from(
        dest: &Path,
        source: crate::source::Source<'_>,
        git_ref: Option<&str>,
        subdir: Option<&str>,
        force: bool,
    ) -> Result<Self> {
        Self::init_from_with_mechanism(dest, source, git_ref, subdir, force).map(|(store, _)| store)
    }

    /// Like [`Store::init_from_dir`]/[`Store::init_from_repo`] (via their shared
    /// `init_from`), but also returns which mechanism actually adopted the content — needed
    /// by `Library::fetch` to record the right [`crate::source::AdoptionMechanism`] in
    /// `sources.toml` so a later `update` picks the matching, SIGPIPE-safe strategy.
    pub(crate) fn init_from_with_mechanism(
        dest: &Path,
        source: crate::source::Source<'_>,
        git_ref: Option<&str>,
        subdir: Option<&str>,
        force: bool,
    ) -> Result<(Self, crate::source::AdoptionMechanism)> {
        let Some(subdir) = subdir else {
            let (repo, mechanism) = crate::source::clone_or_copy(dest, source, git_ref)?;
            let root = dest.to_path_buf();
            if !root.join(TEMPLATES_DIR).is_dir() {
                return Err(Error::NotAStore(root));
            }
            return Ok((Self { repo, root }, mechanism));
        };

        eprintln!(
            "note: --subdir imports current content only; source history/tags for this subdirectory are not preserved"
        );
        let staging = tempfile::tempdir().map_err(|source| Error::Io {
            path: PathBuf::from("<--subdir staging dir>"),
            source,
        })?;
        crate::source::clone_or_copy(staging.path(), source, git_ref)?;
        let staged_store = Self::open(staging.path().join(subdir))?;
        let entries = staged_store.list()?;

        let dest_store = if dest.join(".git").exists() {
            if !force {
                return Err(Error::AlreadyInitialized(dest.to_path_buf()));
            }
            Self::open(dest)?
        } else {
            Self::init(dest)?
        };
        for entry in entries {
            dest_store.add(
                &entry.name,
                &entry.meta,
                &entry.body,
                "import from --subdir",
            )?;
        }
        // History/tags for a `--subdir` import aren't preserved, so there's no continuous
        // relationship with an origin to fast-forward from later — `Copied` here is
        // arbitrary; `Library::fetch` records `subdir` regardless of mechanism, and any
        // source with `subdir` set is never updatable (see `crate::library::Library::update`).
        Ok((dest_store, crate::source::AdoptionMechanism::Copied))
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
        // Don't rely on gix's automatic author/committer resolution: it errors out
        // entirely if the environment has no git identity configured at all (no
        // ~/.gitconfig, no GIT_*_NAME/EMAIL), which is the default state on a fresh
        // CI runner and for plenty of real users who've never run `git config
        // user.name`. lengua's commits aren't meant to represent human authorship
        // (they're just delta storage for a template's history), so fall back to a
        // fixed synthetic identity rather than failing `add` outright.
        let sig = repo
            .committer()
            .and_then(|r| r.ok())
            .map(Into::into)
            .unwrap_or_else(|| gix::actor::Signature {
                name: "lengua".into(),
                email: "lengua@localhost".into(),
                time: gix::date::Time::now_local_or_utc(),
            });
        let mut author_buf = gix::date::parse::TimeBuf::default();
        let mut committer_buf = gix::date::parse::TimeBuf::default();
        let commit_id = repo
            .commit_as(
                sig.to_ref(&mut committer_buf),
                sig.to_ref(&mut author_buf),
                "HEAD",
                message,
                tree_id.detach(),
                parents,
            )
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

    /// Reads `name`'s full frontmatter+body text as it existed at `rev`.
    /// `rev` is first tried as a tag name scoped to `name` (see
    /// [`Store::tag_create`]); if no such tag exists, it's resolved as any
    /// revspec gix's `rev_parse_single` accepts (`HEAD`, `HEAD~1`, a commit
    /// id).
    pub fn read_at_revision(&self, name: &str, rev: &str) -> Result<String> {
        let rel = self.git_path(name);
        let repo = &self.repo;
        let id = match self.tag_resolve(name, rev)? {
            Some(id) => id,
            None => repo
                .rev_parse_single(rev)
                .map_err(|e| Error::InvalidRevision {
                    name: name.to_string(),
                    rev: rev.to_string(),
                    reason: e.to_string(),
                })?
                .detach(),
        };
        let commit = repo
            .find_object(id)
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

    /// Reads and parses `name` as it existed at `rev` (see
    /// [`Store::read_at_revision`] for how `rev` is resolved).
    pub fn get_at_revision(&self, name: &str, rev: &str) -> Result<TemplateEntry> {
        let text = self.read_at_revision(name, rev)?;
        let parsed = frontmatter::parse(&text)?;
        Ok(TemplateEntry {
            name: name.to_string(),
            meta: parsed.meta,
            body: parsed.body,
        })
    }

    /// Points a lengua tag (`refs/lengua/tags/<name>/<tag>`, not a git tag)
    /// at the commit for `rev` (default `HEAD`) of template `name`. Refuses
    /// to overwrite an existing tag unless `force` is set.
    pub fn tag_create(
        &self,
        name: &str,
        tag: &str,
        rev: Option<&str>,
        force: bool,
    ) -> Result<TagEntry> {
        tags::validate_tag_name(tag)?;
        let repo = &self.repo;
        let commit_id = repo
            .rev_parse_single(rev.unwrap_or("HEAD"))
            .map_err(|e| Error::InvalidRevision {
                name: name.to_string(),
                rev: rev.unwrap_or("HEAD").to_string(),
                reason: e.to_string(),
            })?
            .detach();
        if !force && self.tag_resolve(name, tag)?.is_some() {
            return Err(Error::TagAlreadyExists {
                name: name.to_string(),
                tag: tag.to_string(),
            });
        }
        let constraint = if force {
            PreviousValue::Any
        } else {
            PreviousValue::MustNotExist
        };
        repo.reference(
            tags::tag_ref_name(name, tag),
            commit_id,
            constraint,
            "lengua tag",
        )
        .map_err(|e| Error::Git(e.to_string()))?;
        Ok(TagEntry {
            tag: tag.to_string(),
            commit: commit_id.to_string(),
        })
    }

    /// Lists every lengua tag on template `name`.
    pub fn tag_list(&self, name: &str) -> Result<Vec<TagEntry>> {
        let prefix = format!("{}/{name}/", tags::TAG_REF_PREFIX);
        let platform = self
            .repo
            .references()
            .map_err(|e| Error::Git(e.to_string()))?;
        let iter = platform
            .prefixed(prefix.as_bytes())
            .map_err(|e| Error::Git(e.to_string()))?;
        let mut entries = Vec::new();
        for r in iter {
            let r = r.map_err(|e| Error::Git(e.to_string()))?;
            let full_name = r.name().to_string();
            let tag = full_name
                .strip_prefix(&prefix)
                .unwrap_or(&full_name)
                .to_string();
            let commit = r.id().to_string();
            entries.push(TagEntry { tag, commit });
        }
        entries.sort_by(|a, b| a.tag.cmp(&b.tag));
        Ok(entries)
    }

    /// Removes a lengua tag from template `name`.
    pub fn tag_remove(&self, name: &str, tag: &str) -> Result<()> {
        if self.tag_resolve(name, tag)?.is_none() {
            return Err(Error::TagNotFound {
                name: name.to_string(),
                tag: tag.to_string(),
            });
        }
        let ref_name = tags::tag_ref_name(name, tag)
            .try_into()
            .map_err(|e: gix::validate::reference::name::Error| Error::Git(e.to_string()))?;
        self.repo
            .edit_reference(RefEdit {
                change: Change::Delete {
                    expected: PreviousValue::MustExist,
                    log: RefLog::AndReference,
                },
                name: ref_name,
                deref: false,
            })
            .map_err(|e| Error::Git(e.to_string()))?;
        Ok(())
    }

    /// Resolves a lengua tag on template `name` to the commit it points at,
    /// or `None` if no such tag exists (including when `tag` isn't even a
    /// syntactically valid ref-name component, e.g. `"HEAD~1"` — such
    /// strings can never have been created as a tag, so they simply aren't
    /// one, rather than being an error).
    pub fn tag_resolve(&self, name: &str, tag: &str) -> Result<Option<gix::ObjectId>> {
        let ref_name = tags::tag_ref_name(name, tag);
        if gix::validate::reference::name_partial(gix::bstr::BStr::new(ref_name.as_bytes()))
            .is_err()
        {
            return Ok(None);
        }
        let reference = self
            .repo
            .try_find_reference(ref_name.as_str())
            .map_err(|e| Error::Git(e.to_string()))?;
        Ok(reference.map(|r| r.id().detach()))
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
        } else if let Ok(rel) = path.strip_prefix(root) {
            // Always join with `/`, not `Path`'s platform separator: `name` is used both as
            // a filesystem-joinable id (`abs_path`, where `/` works fine on Windows too) and
            // as a git tree path (`git_path`), and as the key `Library`'s merge matches
            // across sources by exact string equality — a `\`-joined name on Windows would
            // silently fail that lookup even though the file exists.
            let name: String = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            if !name.is_empty() {
                names.insert(name);
            }
        }
    }
    Ok(())
}
