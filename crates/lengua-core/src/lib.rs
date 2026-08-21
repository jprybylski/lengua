//! Core engine for `lengua`: a git-backed library of templated text.
//!
//! Reads (`get`/`list`/`search`) operate on the working tree; `add` writes
//! and commits via `gix`; `log`/`diff` inspect git history. No CLI or R
//! bindings live here.

pub mod diff;
pub mod error;
pub mod frontmatter;
pub mod meta;
pub mod query;
mod source;
pub mod store;
pub mod tags;
pub mod template;

pub use diff::{DiffLine, DiffTag, diff_text};
pub use error::{Error, Result};
pub use meta::TemplateMeta;
pub use query::Query;
pub use store::{LogEntry, Store, TemplateEntry};
pub use tags::TagEntry;
