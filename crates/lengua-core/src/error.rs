use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("git error: {0}")]
    Git(String),

    #[error("template render error: {0}")]
    Render(String),

    #[error("frontmatter parse error: {0}")]
    Frontmatter(String),

    #[error("frontmatter serialize error: {0}")]
    FrontmatterWrite(String),

    #[error("template not found: {0}")]
    NotFound(String),

    #[error("invalid revision '{rev}' for '{name}': {reason}")]
    InvalidRevision {
        name: String,
        rev: String,
        reason: String,
    },

    #[error("store already initialized at {0}")]
    AlreadyInitialized(PathBuf),

    #[error(
        "'{0}' doesn't look like a lengua store (no templates/ directory) — pass --store <path>, or run `lengua init` here first"
    )]
    NotAStore(PathBuf),

    #[error("invalid tag name '{tag}': {reason}")]
    InvalidTagName { tag: String, reason: String },

    #[error("tag '{tag}' already exists on '{name}' (use --force to overwrite)")]
    TagAlreadyExists { name: String, tag: String },

    #[error("no tag '{tag}' on '{name}'")]
    TagNotFound { name: String, tag: String },

    #[error(
        "'{0}' doesn't look like a lengua library (no .lengua/sources.toml) — pass --store <path>, or run `lengua init` here first"
    )]
    NotALibrary(PathBuf),

    #[error("couldn't read sources manifest at {path}: {reason}")]
    SourcesManifest { path: PathBuf, reason: String },

    #[error("no source named '{name}'")]
    UnknownSource { name: String },

    #[error("multiple sources exist ({candidates}) — pass --source <name> to pick one")]
    AmbiguousSource { candidates: String },

    #[error("a source named '{name}' already exists — pass --name to pick a different one")]
    DuplicateSourceName { name: String },

    #[error("source '{name}' can't be updated: {reason}")]
    SourceNotUpdatable { name: String, reason: String },

    #[error(
        "source '{name}' has diverged from its origin and can't be fast-forwarded — resolve manually or re-fetch it fresh"
    )]
    NotFastForward { name: String },

    #[error("skill file(s) already exist: {} (use --force to overwrite)", paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "))]
    SkillAlreadyExists { paths: Vec<PathBuf> },
}

pub type Result<T> = std::result::Result<T, Error>;
