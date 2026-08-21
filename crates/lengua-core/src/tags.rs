//! Named pointers to a specific revision of a single template.
//!
//! These are *not* git tags (`refs/tags/*`, which are repo-wide). Each tag
//! lives at `refs/lengua/tags/<template>/<tag>` so it's scoped to one
//! template, letting the same tag name (e.g. `"final"`) exist independently
//! on several templates.

use crate::error::{Error, Result};

/// Ref namespace lengua's own tags live under, distinct from `refs/tags/*`.
pub const TAG_REF_PREFIX: &str = "refs/lengua/tags";

/// A tag pointing at the commit that introduced a template's tagged content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagEntry {
    pub tag: String,
    pub commit: String,
}

pub(crate) fn tag_ref_name(template: &str, tag: &str) -> String {
    format!("{TAG_REF_PREFIX}/{template}/{tag}")
}

/// Rejects tag names that would be ambiguous with the revspecs
/// `Store::read_at_revision` already understands (`HEAD`, a commit sha).
pub(crate) fn validate_tag_name(tag: &str) -> Result<()> {
    if tag.is_empty() {
        return Err(Error::InvalidTagName {
            tag: tag.to_string(),
            reason: "must not be empty".to_string(),
        });
    }
    if tag.eq_ignore_ascii_case("HEAD") {
        return Err(Error::InvalidTagName {
            tag: tag.to_string(),
            reason: "'HEAD' is reserved".to_string(),
        });
    }
    if tag.len() >= 4 && tag.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::InvalidTagName {
            tag: tag.to_string(),
            reason: "looks like a commit id, which would be ambiguous as a revision".to_string(),
        });
    }
    Ok(())
}
