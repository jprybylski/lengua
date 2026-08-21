use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Frontmatter metadata attached to a template: a well-known `title` plus an
/// open-ended bag of fields (tense, tone, jurisdiction, ...) so the library
/// doesn't need to know the metadata schema in advance. Unrelated to
/// [`crate::tags`] — that's revision pointers (`lengua tag`), not frontmatter.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TemplateMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(flatten)]
    pub fields: BTreeMap<String, serde_yaml::Value>,
}

impl TemplateMeta {
    /// Returns the string representation(s) of `key`, for matching against a
    /// query filter. Scalars yield a single string; sequences are flattened.
    pub fn field_strings(&self, key: &str) -> Vec<String> {
        if key == "title" {
            return self.title.iter().cloned().collect();
        }
        self.fields
            .get(key)
            .map(value_to_strings)
            .unwrap_or_default()
    }
}

fn value_to_strings(value: &serde_yaml::Value) -> Vec<String> {
    match value {
        serde_yaml::Value::String(s) => vec![s.clone()],
        serde_yaml::Value::Bool(b) => vec![b.to_string()],
        serde_yaml::Value::Number(n) => vec![n.to_string()],
        serde_yaml::Value::Sequence(seq) => seq.iter().flat_map(value_to_strings).collect(),
        _ => Vec::new(),
    }
}
