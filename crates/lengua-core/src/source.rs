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
use crate::store::Store;
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
) -> Result<gix::Repository> {
    match source {
        Source::Dir(path) => copy_local_dir(dest, path),
        Source::Url(url) => match git_ref {
            None => match local_path_of(url)? {
                Some(path) => copy_local_dir(dest, &path),
                None => clone_via_transport(dest, url, None),
            },
            Some(git_ref) => clone_via_transport(dest, url, Some(git_ref)),
        },
    }
}

/// Parses `url` and returns the local filesystem path it names, if any
/// (a bare path or a `file://` URL) — `None` for a genuine remote.
fn local_path_of(url: &str) -> Result<Option<PathBuf>> {
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
