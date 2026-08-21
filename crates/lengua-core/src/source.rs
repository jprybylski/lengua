//! Adopting an existing store (local directory or remote git URL) as the
//! starting point for a new one.
//!
//! A local directory source is copied directly on the filesystem — never
//! through `gix`'s network transport, and never by shelling out to a `git`
//! binary. This isn't just an optimization: `gix`'s local-file transport
//! (used for `file://` URLs and bare local paths alike) spawns a
//! `git-upload-pack` child process to simulate the smart protocol even for
//! a source that's already sitting on the same filesystem, and that child
//! process is unsafe to rely on from any embedder that sets `SIGPIPE` to
//! `SIG_IGN` at startup — R does, which is how this was found (see
//! `lenguar`'s `?lq_init` docs for the full story). Since a local directory
//! needs no network protocol to reach `dest` in the first place, this
//! sidesteps the problem entirely rather than working around it. A genuine
//! remote (`https://`, `ssh://`, ...) still goes through `gix`'s transport,
//! which does not spawn a subprocess and isn't affected.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::store::{Store, TEMPLATES_DIR};
use crate::tags::TAG_REF_PREFIX;

/// Where `init_from_dir`/`init_from_repo` should adopt a store from.
pub(crate) enum Source<'a> {
    /// Always the safe, subprocess-free filesystem-copy path — this is
    /// what `Store::init_from_dir` constructs, since its source is already
    /// known to be a local directory.
    Dir(&'a Path),
    /// A URL or `init_from_repo`'s free-form string, which might still
    /// turn out to name a local path once parsed (see [`clone_or_copy`]).
    Url(&'a str),
}

/// Which mechanism [`clone_or_copy`] actually used to adopt a source, so callers can record
/// it (see `crate::manifest::SourceKind`) and later pick the matching `update_*` strategy.
/// This is a property of *how the content ended up on disk*, not of which CLI flag was
/// passed — `--from-repo` naming a local/`file://` path with no explicit ref takes the same
/// `Copied` path as `--from-dir` (see the `Source::Url` arm of `clone_or_copy` below).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdoptionMechanism {
    /// A raw filesystem copy (`copy_local_dir`) — updating this source later must avoid
    /// `gix`'s local-file transport for the same SIGPIPE-safety reason described below.
    Copied,
    /// A real `gix` transport clone (`clone_via_transport`) — updating this source later can
    /// safely reuse that same transport.
    Cloned,
}

/// The result of a successful `update_cloned`/`update_copied` fast-forward.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// The source's local content already matched its origin; nothing changed.
    UpToDate,
    /// The source's local branch was fast-forwarded from `from` to `to` (both commit ids).
    FastForwarded { from: String, to: String },
}

/// Adopts `source` into `dest`, choosing the safe filesystem-copy path
/// automatically whenever the source is local and no specific `git_ref`
/// was requested (copying already leaves `HEAD` wherever the source's
/// `HEAD` points, so there's nothing to select). Falls back to `gix`'s
/// transport-based clone for genuine remotes, and for a local source when
/// `git_ref` forces an actual checkout (a rarer combination, left on the
/// transport path rather than teaching the copy path to check out an
/// arbitrary ref after the fact).
pub(crate) fn clone_or_copy(
    dest: &Path,
    source: Source,
    git_ref: Option<&str>,
) -> Result<(gix::Repository, AdoptionMechanism)> {
    match source {
        Source::Dir(path) => {
            copy_local_dir(dest, path).map(|repo| (repo, AdoptionMechanism::Copied))
        }
        Source::Url(url) => match git_ref {
            None => match local_path_of(url)? {
                Some(path) => {
                    copy_local_dir(dest, &path).map(|repo| (repo, AdoptionMechanism::Copied))
                }
                None => clone_via_transport(dest, url, None)
                    .map(|repo| (repo, AdoptionMechanism::Cloned)),
            },
            Some(git_ref) => clone_via_transport(dest, url, Some(git_ref))
                .map(|repo| (repo, AdoptionMechanism::Cloned)),
        },
    }
}

/// Parses `url` and returns the local filesystem path it names, if any
/// (a bare path or a `file://` URL) — `None` for a genuine remote.
pub(crate) fn local_path_of(url: &str) -> Result<Option<PathBuf>> {
    let parsed: gix::Url = url
        .try_into()
        .map_err(|e: gix::url::parse::Error| Error::Git(e.to_string()))?;
    Ok((parsed.scheme == gix::url::Scheme::File).then(|| gix_path::from_bstring(parsed.path)))
}

/// Copies `source_dir` (an existing lengua store) directly onto the
/// filesystem at `dest`: a real, independent copy of its `.git` history
/// and working tree, not a hardlink or a reference to the original — so
/// `dest` is safe to modify without any risk to `source_dir`. `dest` must
/// not already exist or must be empty.
fn copy_local_dir(dest: &Path, source_dir: &Path) -> Result<gix::Repository> {
    if dest.exists()
        && std::fs::read_dir(dest)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false)
    {
        return Err(Error::AlreadyInitialized(dest.to_path_buf()));
    }
    // Validate the source is an actual lengua store before copying anything.
    Store::open(source_dir)?;

    copy_dir_recursive(source_dir, dest)?;
    gix::open(dest).map_err(|e| Error::Git(e.to_string()))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|source| Error::Io {
        path: dst.to_path_buf(),
        source,
    })?;
    let entries = std::fs::read_dir(src).map_err(|source| Error::Io {
        path: src.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| Error::Io {
            path: src.to_path_buf(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| Error::Io {
            path: entry.path(),
            source,
        })?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else if file_type.is_symlink() {
            copy_symlink(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path).map_err(|source| Error::Io {
                path: dest_path,
                source,
            })?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn copy_symlink(src: &Path, dst: &Path) -> Result<()> {
    let target = std::fs::read_link(src).map_err(|source| Error::Io {
        path: src.to_path_buf(),
        source,
    })?;
    std::os::unix::fs::symlink(target, dst).map_err(|source| Error::Io {
        path: dst.to_path_buf(),
        source,
    })
}

#[cfg(not(unix))]
fn copy_symlink(src: &Path, dst: &Path) -> Result<()> {
    // Best effort on platforms without a simple symlink() call (e.g.
    // Windows, where creating one can require elevated privileges):
    // copy the target file's contents instead of failing outright. A
    // lengua store created by `Store::init`/`add` never contains symlinks,
    // so this only matters for a hand-modified source directory.
    std::fs::copy(src, dst).map_err(|source| Error::Io {
        path: dst.to_path_buf(),
        source,
    })?;
    Ok(())
}

/// Clones `source` (a remote URL, or a local path/`file://` URL when a
/// specific `git_ref` needs checking out) via `gix`'s network transport,
/// checking out `git_ref` if given (else the source's default branch/
/// `HEAD`). `dest` must not already exist or must be empty.
///
/// `git_ref` must be a branch or tag name, not a commit id — gix's clone
/// checkout only supports naming a ref to fetch/check out, matching the
/// same restriction as `git clone --branch`.
fn clone_via_transport(dest: &Path, url: &str, git_ref: Option<&str>) -> Result<gix::Repository> {
    if dest.exists()
        && std::fs::read_dir(dest)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false)
    {
        return Err(Error::AlreadyInitialized(dest.to_path_buf()));
    }

    // A plain clone only brings across branches and `refs/tags/*`; lengua's
    // own tags live under a different namespace, so they need an explicit
    // refspec or they'd silently be left behind.
    let tag_refspec = format!("{TAG_REF_PREFIX}/*:{TAG_REF_PREFIX}/*");
    let mut prepare = gix::prepare_clone(url, dest)
        .map_err(|e| Error::Git(e.to_string()))?
        .configure_remote(move |remote| {
            remote
                .with_refspecs([tag_refspec.as_str()], gix::remote::Direction::Fetch)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        });
    if let Some(git_ref) = git_ref {
        prepare = prepare
            .with_ref_name(Some(git_ref))
            .map_err(|e| Error::Git(e.to_string()))?;
    }
    let (mut checkout, _outcome) = prepare
        .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .map_err(|e| Error::Git(e.to_string()))?;
    let (repo, _outcome) = checkout
        .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .map_err(|e| Error::Git(e.to_string()))?;
    Ok(repo)
}

/// Refreshes a `Cloned` source: a real `gix` transport fetch (safe — no subprocess is
/// spawned, unlike the local-file transport `copy_local_dir` deliberately avoids) against
/// its `origin` remote, then a local fast-forward of whatever branch is currently checked
/// out. `git_ref` is the branch the source was originally adopted with (`None` means the
/// remote's default branch was used, which is still whatever local branch is checked out
/// now) — either way, the branch actually being tracked is read back from the local repo's
/// own `HEAD`, so this doesn't depend on `git_ref` matching anything on the remote today.
pub(crate) fn update_cloned(
    dest_root: &Path,
    source_name: &str,
    _git_ref: Option<&str>,
) -> Result<UpdateStatus> {
    let repo = gix::open(dest_root).map_err(|e| Error::Git(e.to_string()))?;
    let branch = current_branch_name(&repo)?;
    let before = repo.head_id().ok().map(|id| id.detach());

    let tag_refspec = format!("{TAG_REF_PREFIX}/*:{TAG_REF_PREFIX}/*");
    let remote = repo
        .find_fetch_remote(None)
        .map_err(|e| Error::Git(e.to_string()))?
        .with_refspecs([tag_refspec.as_str()], gix::remote::Direction::Fetch)
        .map_err(|e| Error::Git(e.to_string()))?;
    let connection = remote
        .connect(gix::remote::Direction::Fetch)
        .map_err(|e| Error::Git(e.to_string()))?;
    let prepare = connection
        .prepare_fetch(gix::progress::Discard, Default::default())
        .map_err(|e| Error::Git(e.to_string()))?;

    let remote_branch_ref: gix::bstr::BString = format!("refs/heads/{branch}").into();
    let target = prepare
        .ref_map()
        .mappings
        .iter()
        .find(|m| m.remote.as_name() == Some(remote_branch_ref.as_ref()))
        .and_then(|m| m.remote.as_id())
        .map(|id| id.to_owned())
        .ok_or_else(|| Error::Git(format!("origin has no '{branch}' branch to fetch")))?;

    prepare
        .receive(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .map_err(|e| Error::Git(e.to_string()))?;

    fast_forward(&repo, source_name, &branch, before, target, dest_root)
}

/// Refreshes a `Copied` source: merges any git objects the recorded `origin` local
/// directory has that this copy doesn't (content-hashed filenames make this purely
/// additive/idempotent), reads `origin`'s current `HEAD` via `gix`'s repository API (a
/// local metadata read — never the transport/subprocess path `copy_local_dir` avoids), and
/// fast-forwards the local branch to it.
pub(crate) fn update_copied(
    dest_root: &Path,
    source_name: &str,
    origin: &Path,
) -> Result<UpdateStatus> {
    let origin_repo = gix::open(origin).map_err(|e| Error::Git(e.to_string()))?;
    let target = origin_repo
        .head_id()
        .map_err(|e| Error::Git(e.to_string()))?
        .detach();

    merge_objects(&origin.join(".git"), &dest_root.join(".git"))?;

    let repo = gix::open(dest_root).map_err(|e| Error::Git(e.to_string()))?;
    let branch = current_branch_name(&repo)?;
    let before = repo.head_id().ok().map(|id| id.detach());

    fast_forward(&repo, source_name, &branch, before, target, dest_root)
}

/// The short name of the branch `HEAD` currently points at (e.g. `"main"`), which is what
/// `update_cloned`/`update_copied` fast-forward — `Store::add` always commits onto whatever
/// branch `HEAD` references, so this is always the branch that matters, regardless of what
/// was originally requested via `--ref` at adoption time.
fn current_branch_name(repo: &gix::Repository) -> Result<String> {
    let head_ref = repo
        .head_ref()
        .map_err(|e| Error::Git(e.to_string()))?
        .ok_or_else(|| {
            Error::Git("HEAD is detached or unborn, nothing to update against".into())
        })?;
    Ok(head_ref.name().shorten().to_string())
}

/// Attempts to fast-forward `branch` from `before` to `target`, updating the ref and
/// re-materializing `templates/` on disk if it moves. `source_name` is used only to build a
/// clear [`Error::NotFastForward`] if `branch` has diverged.
fn fast_forward(
    repo: &gix::Repository,
    source_name: &str,
    branch: &str,
    before: Option<gix::ObjectId>,
    target: gix::ObjectId,
    dest_root: &Path,
) -> Result<UpdateStatus> {
    if before == Some(target) {
        return Ok(UpdateStatus::UpToDate);
    }
    let is_fast_forward = match before {
        None => true,
        Some(before) => repo
            .find_object(target)
            .map_err(|e| Error::Git(e.to_string()))?
            .try_into_commit()
            .map_err(|e| Error::Git(e.to_string()))?
            .ancestors()
            .all()
            .map_err(|e| Error::Git(e.to_string()))?
            .any(|info| info.map(|info| info.id == before).unwrap_or(false)),
    };
    if !is_fast_forward {
        return Err(Error::NotFastForward {
            name: source_name.to_string(),
        });
    }

    repo.reference(
        format!("refs/heads/{branch}"),
        target,
        gix::refs::transaction::PreviousValue::MustExist,
        "lengua update: fast-forward",
    )
    .map_err(|e| Error::Git(e.to_string()))?;
    checkout_tree(repo, target, dest_root)?;

    Ok(UpdateStatus::FastForwarded {
        from: before.map(|id| id.to_string()).unwrap_or_default(),
        to: target.to_string(),
    })
}

/// Re-materializes `templates/` under `dest_root` to match `commit_id`'s tree — `Store`'s
/// reads are working-tree-based, not git-object-based, so a ref update alone wouldn't be
/// visible to `get`/`list`/`search` until the working tree is rewritten to match.
fn checkout_tree(repo: &gix::Repository, commit_id: gix::ObjectId, dest_root: &Path) -> Result<()> {
    let commit = repo
        .find_object(commit_id)
        .map_err(|e| Error::Git(e.to_string()))?
        .try_into_commit()
        .map_err(|e| Error::Git(e.to_string()))?;
    let tree = commit.tree().map_err(|e| Error::Git(e.to_string()))?;
    let entries = tree
        .traverse()
        .breadthfirst
        .files()
        .map_err(|e| Error::Git(e.to_string()))?;

    let templates_dir = dest_root.join(TEMPLATES_DIR);
    if templates_dir.exists() {
        std::fs::remove_dir_all(&templates_dir).map_err(|source| Error::Io {
            path: templates_dir.clone(),
            source,
        })?;
    }

    // `breadthfirst.files()` still yields tree (directory) entries alongside blobs despite
    // its name — skip them, `create_dir_all` below already makes any directory we need.
    for entry in entries.into_iter().filter(|entry| !entry.mode.is_tree()) {
        let rel = gix_path::from_bstring(entry.filepath);
        let path = dest_root.join(&rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| Error::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let data = repo
            .find_object(entry.oid)
            .map_err(|e| Error::Git(e.to_string()))?
            .data
            .clone();
        std::fs::write(&path, data).map_err(|source| Error::Io { path, source })?;
    }
    Ok(())
}

/// Copies any git objects under `origin_git_dir/objects/**` that `dest_git_dir/objects/**`
/// doesn't already have. Object filenames are content hashes, so this is purely additive —
/// it can only add missing objects, never touch or corrupt an existing one.
fn merge_objects(origin_git_dir: &Path, dest_git_dir: &Path) -> Result<()> {
    let origin_objects = origin_git_dir.join("objects");
    let dest_objects = dest_git_dir.join("objects");
    merge_dir_recursive(&origin_objects, &dest_objects)
}

fn merge_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    let entries = match std::fs::read_dir(src) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    std::fs::create_dir_all(dst).map_err(|source| Error::Io {
        path: dst.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| Error::Io {
            path: src.to_path_buf(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| Error::Io {
            path: entry.path(),
            source,
        })?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            merge_dir_recursive(&entry.path(), &dest_path)?;
        } else if !dest_path.exists() {
            std::fs::copy(entry.path(), &dest_path).map_err(|source| Error::Io {
                path: dest_path,
                source,
            })?;
        }
    }
    Ok(())
}
