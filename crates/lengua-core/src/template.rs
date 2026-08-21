use serde::Serialize;

use crate::error::{Error, Result};

/// Renders `body` as a minijinja template against `context`.
pub fn render(body: &str, context: &impl Serialize) -> Result<String> {
    let env = minijinja::Environment::new();
    env.render_str(body, context)
        .map_err(|e| Error::Render(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn substitutes_variables() {
        let mut ctx = BTreeMap::new();
        ctx.insert("name", "Ada");
        let out = render("Dear {{ name }},", &ctx).unwrap();
        assert_eq!(out, "Dear Ada,");
    }

    #[test]
    fn renders_without_variables_when_unused() {
        let ctx: BTreeMap<&str, &str> = BTreeMap::new();
        let out = render("static text", &ctx).unwrap();
        assert_eq!(out, "static text");
    }
}
