//! Parses `init --from-repo`'s value: either a full git URL, or the
//! `[host/]owner/repo[/subdir][@ref]` shorthand (host defaults to
//! `github.com`; an explicit host is how GitHub Enterprise is supported).

pub struct FromRepo {
    pub url: String,
    pub subdir: Option<String>,
    pub git_ref: Option<String>,
}

pub fn parse(spec: &str) -> Result<FromRepo, String> {
    if looks_like_url(spec) {
        return Ok(FromRepo {
            url: spec.to_string(),
            subdir: None,
            git_ref: None,
        });
    }

    let (path_part, git_ref) = match spec.rsplit_once('@') {
        Some((p, r)) => (p, Some(r.to_string())),
        None => (spec, None),
    };
    let mut segments: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Err(format!(
            "expected `[host/]owner/repo[/subdir][@ref]` or a URL, got `{spec}`"
        ));
    }

    let host = if segments.len() > 2 && segments[0].contains('.') {
        segments.remove(0).to_string()
    } else {
        "github.com".to_string()
    };
    if segments.len() < 2 {
        return Err(format!(
            "expected `[host/]owner/repo[/subdir][@ref]` or a URL, got `{spec}`"
        ));
    }
    let owner = segments[0];
    let repo = segments[1];
    let subdir = if segments.len() > 2 {
        Some(segments[2..].join("/"))
    } else {
        None
    };

    Ok(FromRepo {
        url: format!("https://{host}/{owner}/{repo}.git"),
        subdir,
        git_ref,
    })
}

fn looks_like_url(spec: &str) -> bool {
    spec.starts_with("http://")
        || spec.starts_with("https://")
        || spec.starts_with("ssh://")
        || spec.starts_with("file://")
        || spec.starts_with("git@")
        || spec.contains("://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_owner_repo_shorthand_with_default_host() {
        let parsed = parse("acme/templates").unwrap();
        assert_eq!(parsed.url, "https://github.com/acme/templates.git");
        assert_eq!(parsed.subdir, None);
        assert_eq!(parsed.git_ref, None);
    }

    #[test]
    fn parses_explicit_host_for_enterprise() {
        let parsed = parse("git.acme.internal/team/templates").unwrap();
        assert_eq!(parsed.url, "https://git.acme.internal/team/templates.git");
    }

    #[test]
    fn parses_subdir_and_ref() {
        let parsed = parse("acme/templates/letters@v2").unwrap();
        assert_eq!(parsed.url, "https://github.com/acme/templates.git");
        assert_eq!(parsed.subdir.as_deref(), Some("letters"));
        assert_eq!(parsed.git_ref.as_deref(), Some("v2"));
    }

    #[test]
    fn passes_through_full_urls_unchanged() {
        let parsed = parse("https://example.com/acme/templates.git").unwrap();
        assert_eq!(parsed.url, "https://example.com/acme/templates.git");
        let parsed = parse("git@github.com:acme/templates.git").unwrap();
        assert_eq!(parsed.url, "git@github.com:acme/templates.git");
    }

    #[test]
    fn rejects_a_bare_owner_with_no_repo() {
        assert!(parse("acme").is_err());
    }
}
