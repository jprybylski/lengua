use crate::meta::TemplateMeta;

/// A simple AND-of-equality filter over frontmatter fields, e.g.
/// `Query::new().with("tense", "present").with("formality", "high")`.
/// Deliberately not a query language: the metadata is small and in-memory,
/// so a predicate list is all that's needed.
#[derive(Debug, Clone, Default)]
pub struct Query {
    filters: Vec<(String, String)>,
}

impl Query {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.filters.push((key.into(), value.into()));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    pub fn matches(&self, meta: &TemplateMeta) -> bool {
        self.filters
            .iter()
            .all(|(key, value)| meta.field_strings(key).iter().any(|v| v == value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn meta_with(fields: &[(&str, serde_yaml::Value)]) -> TemplateMeta {
        TemplateMeta {
            title: None,
            fields: fields
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn matches_scalar_and_sequence_fields() {
        let meta = meta_with(&[
            ("tense", serde_yaml::Value::String("present".into())),
            (
                "jurisdiction",
                serde_yaml::Value::Sequence(vec![
                    serde_yaml::Value::String("CA".into()),
                    serde_yaml::Value::String("NY".into()),
                ]),
            ),
        ]);

        assert!(Query::new().with("tense", "present").matches(&meta));
        assert!(Query::new().with("jurisdiction", "NY").matches(&meta));
        assert!(!Query::new().with("tense", "past").matches(&meta));
        assert!(
            !Query::new()
                .with("tense", "present")
                .with("jurisdiction", "TX")
                .matches(&meta)
        );
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(Query::new().matches(&TemplateMeta::default()));
    }
}
