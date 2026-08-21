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
}

pub type Result<T> = std::result::Result<T, Error>;
