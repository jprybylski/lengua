//! `Library`: a `.lengua/` directory holding one or more named [`Store`]s, orchestrated via
//! `sources.toml` (see [`crate::manifest`]). This is the type `lengua init`/`fetch`/`update`
//! and every other CLI command actually operates on — `Store` itself stays the unchanged,
//! single-source primitive, nested one level deeper at `.lengua/<name>/`.
//!
//! Reads (`get`/`list`/`search`) with no explicit source merge across every source, last
//! fetched (i.e. last in `sources.toml`) wins on a name collision, and a collision always
//! produces a [`ShadowWarning`] — never a silent resolution. Writes (`add`) and the
//! inherently single-source `log`/`diff`/`tag_*` need an unambiguous target: an explicit
//! source name, or the sole source if there's only one, else [`crate::Error::AmbiguousSource`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::error::{Error, Result};
use crate::manifest::{self, SourceEntry, SourceKind, SourcesManifest};
use crate::meta::TemplateMeta;
use crate::query::Query;
use crate::source::{self, AdoptionMechanism, Source, UpdateStatus};
use crate::store::{LogEntry, Store, TemplateEntry};
use crate::tags::TagEntry;

pub(crate) const LENGUA_DIR: &str = ".lengua";
const DEFAULT_LOCAL_NAME: &str = "local";

/// A name collision between two sources, discovered either when the shadowing source is
/// `fetch`ed or lazily when a merged `get`/`list`/`search` first resolves it. Never silent:
/// `winner` is whichever source is higher-precedence (later in `sources.toml`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowWarning {
    pub name: String,
    pub winner: String,
    pub loser: String,
}

/// The result of a successful [`Library::fetch`].
#[derive(Debug, Clone)]
pub struct FetchOutcome {
    pub source: String,
    pub warnings: Vec<ShadowWarning>,
}

struct Merged {
    winners: BTreeMap<String, usize>,
    warnings: Vec<ShadowWarning>,
}

pub struct Library {
    root: PathBuf,
    lengua_dir: PathBuf,
    manifest: SourcesManifest,
    stores: Vec<(SourceEntry, Store)>,
    merged: OnceLock<Merged>,
}

impl Library {
    /// Creates `.lengua/` at `root` with a single empty, writable source (default name
    /// `"local"`, overridden by `name`). Errors if `.lengua/` already exists.
    pub fn init(root: impl AsRef<Path>, name: Option<&str>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let lengua_dir = root.join(LENGUA_DIR);
        if lengua_dir.exists() {
            return Err(Error::AlreadyInitialized(root));
        }
        std::fs::create_dir_all(&lengua_dir).map_err(|source| Error::Io {
            path: lengua_dir.clone(),
            source,
        })?;

        let name = name.unwrap_or(DEFAULT_LOCAL_NAME).to_string();
        Store::init(lengua_dir.join(&name))?;

        let mut sources = SourcesManifest::new();
        sources.push(SourceEntry {
            name,
            kind: SourceKind::Local,
        })?;
        sources.save(&manifest::manifest_path(&lengua_dir))?;

        Self::open(&root)
    }

    /// Creates `.lengua/` at `root`, adopting exactly one source (via `--from-dir`/
    /// `--from-repo`-shaped arguments) as its first and only entry — no empty `local` source
    /// is force-created (pure-consumer mode). Errors if `.lengua/` already exists.
    #[allow(clippy::too_many_arguments)]
    pub fn init_from(
        root: impl AsRef<Path>,
        name: Option<&str>,
        from_dir: Option<&Path>,
        from_repo_url: Option<&str>,
        git_ref: Option<&str>,
        subdir: Option<&str>,
        force: bool,
    ) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let lengua_dir = root.join(LENGUA_DIR);
        if lengua_dir.exists() {
            return Err(Error::AlreadyInitialized(root));
        }
        std::fs::create_dir_all(&lengua_dir).map_err(|source| Error::Io {
            path: lengua_dir.clone(),
            source,
        })?;

        let sources = SourcesManifest::new();
        let (entry, _store) = adopt_source(
            &lengua_dir,
            name,
            from_dir,
            from_repo_url,
            git_ref,
            subdir,
            force,
            &sources,
        )?;
        let mut sources = sources;
        sources.push(entry)?;
        sources.save(&manifest::manifest_path(&lengua_dir))?;

        Self::open(&root)
    }

    /// Opens an existing `.lengua/` at `root`, loading its manifest and every source it
    /// names. Errors with [`Error::NotALibrary`] if `.lengua/sources.toml` is missing or
    /// unparseable — this, not any `templates/`-adjacent heuristic, is now the entire
    /// "is this a lengua library" check.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let lengua_dir = root.join(LENGUA_DIR);
        let manifest_path = manifest::manifest_path(&lengua_dir);
        if !manifest_path.is_file() {
            return Err(Error::NotALibrary(root));
        }
        let manifest = SourcesManifest::load(&manifest_path)?;
        let mut stores = Vec::with_capacity(manifest.sources.len());
        for entry in &manifest.sources {
            let store = Store::open(lengua_dir.join(&entry.name))?;
            stores.push((entry.clone(), store));
        }
        Ok(Self {
            root,
            lengua_dir,
            manifest,
            stores,
            merged: OnceLock::new(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Every source's name, in manifest/precedence order (lowest to highest precedence).
    pub fn manifest_order(&self) -> Vec<String> {
        self.manifest
            .sources
            .iter()
            .map(|entry| entry.name.clone())
            .collect()
    }

    /// Adds a new source to an already-`init`ed library — same adoption-flag language as
    /// [`Library::init_from`], but appends rather than replacing. The new entry becomes the
    /// highest-precedence source (see the module docs on merge precedence).
    #[allow(clippy::too_many_arguments)]
    pub fn fetch(
        &mut self,
        name: Option<&str>,
        from_dir: Option<&Path>,
        from_repo_url: Option<&str>,
        git_ref: Option<&str>,
        subdir: Option<&str>,
        force: bool,
    ) -> Result<FetchOutcome> {
        let (entry, store) = adopt_source(
            &self.lengua_dir,
            name,
            from_dir,
            from_repo_url,
            git_ref,
            subdir,
            force,
            &self.manifest,
        )?;

        let new_names: std::collections::BTreeSet<String> =
            store.list()?.into_iter().map(|entry| entry.name).collect();
        let mut warnings = Vec::new();
        for (existing_entry, existing_store) in &self.stores {
            for existing_name in existing_store.list()?.into_iter().map(|e| e.name) {
                if new_names.contains(&existing_name) {
                    warnings.push(ShadowWarning {
                        name: existing_name,
                        winner: entry.name.clone(),
                        loser: existing_entry.name.clone(),
                    });
                }
            }
        }

        self.manifest.push(entry.clone())?;
        self.manifest
            .save(&manifest::manifest_path(&self.lengua_dir))?;
        self.stores.push((entry.clone(), store));
        self.merged = OnceLock::new();

        Ok(FetchOutcome {
            source: entry.name,
            warnings,
        })
    }

    /// Refreshes one source (`Some(name)`) or every source (`None`) by fetching from its
    /// recorded origin and fast-forwarding — never stops at the first failure, so an
    /// all-sources update always reports on every source. A `local` or `--subdir`-imported
    /// source reports [`Error::SourceNotUpdatable`] (informational, not a hard failure); a
    /// genuinely diverged source reports [`Error::NotFastForward`].
    pub fn update(&self, source: Option<&str>) -> Vec<(String, Result<UpdateStatus>)> {
        let targets: Vec<&SourceEntry> = match source {
            Some(name) => match self.stores.iter().find(|(e, _)| e.name == name) {
                Some((entry, _)) => vec![entry],
                None => {
                    return vec![(
                        name.to_string(),
                        Err(Error::UnknownSource {
                            name: name.to_string(),
                        }),
                    )];
                }
            },
            None => self.stores.iter().map(|(entry, _)| entry).collect(),
        };

        targets
            .into_iter()
            .map(|entry| {
                let dest = self.lengua_dir.join(&entry.name);
                let result = match &entry.kind {
                    SourceKind::Local => Err(Error::SourceNotUpdatable {
                        name: entry.name.clone(),
                        reason: "a local source has no origin to update from".to_string(),
                    }),
                    SourceKind::Copied {
                        subdir: Some(_), ..
                    }
                    | SourceKind::Cloned {
                        subdir: Some(_), ..
                    } => Err(Error::SourceNotUpdatable {
                        name: entry.name.clone(),
                        reason: "imported via --subdir; history wasn't preserved so there's \
                                 nothing to fast-forward against"
                            .to_string(),
                    }),
                    SourceKind::Copied {
                        origin,
                        subdir: None,
                    } => source::update_copied(&dest, &entry.name, origin),
                    SourceKind::Cloned {
                        git_ref,
                        subdir: None,
                        ..
                    } => source::update_cloned(&dest, &entry.name, git_ref.as_deref()),
                };
                (entry.name.clone(), result)
            })
            .collect()
    }

    fn find_store(&self, name: &str) -> Result<&Store> {
        self.stores
            .iter()
            .find(|(entry, _)| entry.name == name)
            .map(|(_, store)| store)
            .ok_or_else(|| Error::UnknownSource {
                name: name.to_string(),
            })
    }

    /// Resolves the single source a write (`add`) or inherently single-source read
    /// (`log`/`diff`/`tag_*`) should target: the named source if given, else the sole
    /// source, else [`Error::AmbiguousSource`] naming every candidate.
    pub fn resolve_source(&self, source: Option<&str>) -> Result<&Store> {
        match source {
            Some(name) => self.find_store(name),
            None => match self.stores.len() {
                1 => Ok(&self.stores[0].1),
                _ => Err(Error::AmbiguousSource {
                    candidates: self
                        .stores
                        .iter()
                        .map(|(entry, _)| entry.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                }),
            },
        }
    }

    pub fn add(
        &self,
        source: Option<&str>,
        name: &str,
        meta: &TemplateMeta,
        body: &str,
        message: &str,
    ) -> Result<String> {
        self.resolve_source(source)?.add(name, meta, body, message)
    }

    pub fn log(&self, source: Option<&str>, name: &str) -> Result<Vec<LogEntry>> {
        self.resolve_source(source)?.log(name)
    }

    pub fn tag_create(
        &self,
        source: Option<&str>,
        name: &str,
        tag: &str,
        rev: Option<&str>,
        force: bool,
    ) -> Result<TagEntry> {
        self.resolve_source(source)?
            .tag_create(name, tag, rev, force)
    }

    pub fn tag_list(&self, source: Option<&str>, name: &str) -> Result<Vec<TagEntry>> {
        self.resolve_source(source)?.tag_list(name)
    }

    pub fn tag_remove(&self, source: Option<&str>, name: &str, tag: &str) -> Result<()> {
        self.resolve_source(source)?.tag_remove(name, tag)
    }

    /// Reads `name`, either from an explicit `source` (bypassing the merge entirely — this
    /// is how a shadowed copy is reached) or, unscoped, from whichever source currently wins
    /// that name in the merge. Returns the entry alongside the source it actually came from.
    pub fn get(
        &self,
        source: Option<&str>,
        name: &str,
        rev: Option<&str>,
    ) -> Result<(TemplateEntry, String)> {
        if let Some(source_name) = source {
            let store = self.find_store(source_name)?;
            let entry = match rev {
                Some(rev) => store.get_at_revision(name, rev)?,
                None => store.get(name)?,
            };
            return Ok((entry, source_name.to_string()));
        }

        let idx = *self
            .merge()
            .winners
            .get(name)
            .ok_or_else(|| Error::NotFound(name.to_string()))?;
        let (entry_meta, store) = &self.stores[idx];
        let entry = match rev {
            Some(rev) => store.get_at_revision(name, rev)?,
            None => store.get(name)?,
        };
        Ok((entry, entry_meta.name.clone()))
    }

    /// Lists every template in one `source`, or (unscoped) the merged, last-fetched-wins
    /// view across all sources — each entry tagged with the source it actually came from.
    pub fn list(&self, source: Option<&str>) -> Result<Vec<(TemplateEntry, String)>> {
        if let Some(source_name) = source {
            let store = self.find_store(source_name)?;
            return Ok(store
                .list()?
                .into_iter()
                .map(|entry| (entry, source_name.to_string()))
                .collect());
        }

        let merged = self.merge();
        let mut out = Vec::with_capacity(merged.winners.len());
        for (name, &idx) in &merged.winners {
            let (entry_meta, store) = &self.stores[idx];
            out.push((store.get(name)?, entry_meta.name.clone()));
        }
        Ok(out)
    }

    pub fn search(
        &self,
        source: Option<&str>,
        query: &Query,
    ) -> Result<Vec<(TemplateEntry, String)>> {
        Ok(self
            .list(source)?
            .into_iter()
            .filter(|(entry, _)| query.matches(&entry.meta))
            .collect())
    }

    /// Every currently-known name collision across all sources — forces the lazy merge
    /// computation if it hasn't run yet. Callers (CLI `list`/`search`/unscoped `get`) print
    /// these once, when non-empty, so a shadowed name is never silently resolved.
    pub fn shadow_warnings(&self) -> &[ShadowWarning] {
        &self.merge().warnings
    }

    fn merge(&self) -> &Merged {
        self.merged.get_or_init(|| {
            let mut winners: BTreeMap<String, usize> = BTreeMap::new();
            let mut warnings = Vec::new();
            for (idx, (entry, store)) in self.stores.iter().enumerate() {
                let names = store.list().unwrap_or_default();
                for template in names {
                    if let Some(&prev_idx) = winners.get(&template.name) {
                        warnings.push(ShadowWarning {
                            name: template.name.clone(),
                            winner: entry.name.clone(),
                            loser: self.stores[prev_idx].0.name.clone(),
                        });
                    }
                    winners.insert(template.name, idx);
                }
            }
            Merged { winners, warnings }
        })
    }
}

/// Shared by [`Library::init_from`]/[`Library::fetch`]: adopts one source under `lengua_dir`,
/// deriving/validating its name and recording the right [`SourceKind`] (by adoption
/// mechanism, not by which CLI flag was used — see [`AdoptionMechanism`]).
#[allow(clippy::too_many_arguments)]
fn adopt_source(
    lengua_dir: &Path,
    name: Option<&str>,
    from_dir: Option<&Path>,
    from_repo_url: Option<&str>,
    git_ref: Option<&str>,
    subdir: Option<&str>,
    force: bool,
    existing: &SourcesManifest,
) -> Result<(SourceEntry, Store)> {
    // A local `--from-dir` might itself be a Library (`.lengua/`-shaped) rather than a bare
    // Store (`.git`+`templates/`) — the common case of pulling from another local project.
    // Redirect to its sole source's own directory, which *is* bare-Store-shaped; naming and
    // display below still use the outer path the caller actually gave.
    let resolved_dir = from_dir.map(resolve_local_source_dir).transpose()?;

    let source = match (resolved_dir.as_deref(), from_repo_url) {
        (Some(dir), None) => Source::Dir(dir),
        (None, Some(url)) => Source::Url(url),
        _ => {
            return Err(Error::Git(
                "exactly one of --from-dir/--from-repo is required".to_string(),
            ));
        }
    };

    let auto_name = match (from_dir, from_repo_url) {
        (Some(dir), _) => derive_name_from_dir(dir),
        (_, Some(url)) => derive_name_from_url(url),
        _ => unreachable!("checked above"),
    };
    let name = match name {
        Some(explicit) => explicit.to_string(),
        None => {
            if existing.contains(&auto_name) {
                return Err(Error::DuplicateSourceName { name: auto_name });
            }
            auto_name
        }
    };
    if existing.contains(&name) {
        return Err(Error::DuplicateSourceName { name });
    }

    let dest = lengua_dir.join(&name);
    let (store, mechanism) =
        Store::init_from_with_mechanism(&dest, source, git_ref, subdir, force)?;

    let kind = match mechanism {
        AdoptionMechanism::Copied => {
            let origin = match (resolved_dir, from_repo_url) {
                (Some(dir), _) => dir,
                (None, Some(url)) => source::local_path_of(url)?.ok_or_else(|| {
                    Error::Git(format!("'{url}' was copied but isn't a local path"))
                })?,
                _ => unreachable!("checked above"),
            };
            let origin = origin.canonicalize().unwrap_or(origin);
            SourceKind::Copied {
                origin,
                subdir: subdir.map(str::to_string),
            }
        }
        AdoptionMechanism::Cloned => SourceKind::Cloned {
            url: from_repo_url
                .expect("Cloned mechanism only occurs for --from-repo")
                .to_string(),
            git_ref: git_ref.map(str::to_string),
            subdir: subdir.map(str::to_string),
        },
    };

    Ok((SourceEntry { name, kind }, store))
}

/// If `path` is a `Library` root (`.lengua/sources.toml` exists), redirects to its sole
/// source's directory — the actual bare-Store-shaped git repo `Store::init_from_with_mechanism`
/// needs. Errors if the library has more than one source (ambiguous — the caller should name
/// one directly, e.g. `<path>/.lengua/<name>`) or none. Any other `path` (a bare Store, or
/// anything else) is returned unchanged.
fn resolve_local_source_dir(path: &Path) -> Result<PathBuf> {
    let manifest_path = path.join(LENGUA_DIR).join(manifest::MANIFEST_FILE);
    if !manifest_path.is_file() {
        return Ok(path.to_path_buf());
    }
    let inner = SourcesManifest::load(&manifest_path)?;
    match inner.sources.as_slice() {
        [only] => Ok(path.join(LENGUA_DIR).join(&only.name)),
        [] => Err(Error::Git(format!(
            "'{}' is a lengua library with no sources",
            path.display()
        ))),
        many => Err(Error::Git(format!(
            "'{}' is a lengua library with multiple sources ({}) — point --from-dir at one \
             directly, e.g. '{}/.lengua/<name>'",
            path.display(),
            many.iter()
                .map(|e| e.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            path.display()
        ))),
    }
}

fn derive_name_from_dir(dir: &Path) -> String {
    dir.file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty())
        .unwrap_or("source")
        .to_string()
}

fn derive_name_from_url(url: &str) -> String {
    let trimmed = url.trim_end_matches('/').trim_end_matches(".git");
    trimmed
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("source")
        .to_string()
}
