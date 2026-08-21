use gray_matter::Matter;
use gray_matter::engine::YAML;

use crate::error::{Error, Result};
use crate::meta::TemplateMeta;

pub struct Parsed {
    pub meta: TemplateMeta,
    pub body: String,
}

/// Parses YAML frontmatter (`---\n...\n---`) off the top of `input`, returning
/// the deserialized metadata and the remaining body. Files with no frontmatter
/// parse fine and yield default (empty) metadata.
pub fn parse(input: &str) -> Result<Parsed> {
    let matter = Matter::<YAML>::new();
    let parsed = matter
        .parse::<TemplateMeta>(input)
        .map_err(|e| Error::Frontmatter(e.to_string()))?;
    Ok(Parsed {
        meta: parsed.data.unwrap_or_default(),
        body: parsed.content,
    })
}

/// Serializes `meta` back into a YAML frontmatter block prepended to `body`.
pub fn write(meta: &TemplateMeta, body: &str) -> Result<String> {
    let yaml = serde_yaml::to_string(meta).map_err(|e| Error::FrontmatterWrite(e.to_string()))?;
    Ok(format!("---\n{yaml}---\n\n{body}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_metadata_and_body() {
        let meta = TemplateMeta {
            title: Some("Thank You".into()),
            fields: [(
                "tags".to_string(),
                serde_yaml::Value::Sequence(vec![
                    serde_yaml::Value::String("formal".into()),
                    serde_yaml::Value::String("english".into()),
                ]),
            )]
            .into_iter()
            .collect(),
        };
        let body = "Dear {{ name }}, thank you.\n";

        let text = write(&meta, body).unwrap();
        let parsed = parse(&text).unwrap();

        assert_eq!(parsed.meta, meta);
        assert_eq!(parsed.body.trim_end(), body.trim_end());
    }

    #[test]
    fn parses_body_with_no_frontmatter() {
        let parsed = parse("just a plain body\n").unwrap();
        assert_eq!(parsed.meta, TemplateMeta::default());
        assert_eq!(parsed.body.trim_end(), "just a plain body");
    }
}
