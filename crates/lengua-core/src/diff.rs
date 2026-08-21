use similar::{ChangeTag, TextDiff};

/// A single line of a unified-style diff between two revisions of a template.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffLine {
    pub tag: DiffTag,
    pub line: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffTag {
    Equal,
    Insert,
    Delete,
}

pub fn diff_text(old: &str, new: &str) -> Vec<DiffLine> {
    TextDiff::from_lines(old, new)
        .iter_all_changes()
        .map(|change| {
            let tag = match change.tag() {
                ChangeTag::Equal => DiffTag::Equal,
                ChangeTag::Insert => DiffTag::Insert,
                ChangeTag::Delete => DiffTag::Delete,
            };
            DiffLine {
                tag,
                line: change.to_string_lossy().trim_end_matches('\n').to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_added_and_removed_lines() {
        let lines = diff_text("a\nb\nc\n", "a\nx\nc\n");
        let tags: Vec<DiffTag> = lines.iter().map(|l| l.tag).collect();
        assert!(tags.contains(&DiffTag::Insert));
        assert!(tags.contains(&DiffTag::Delete));
    }
}
