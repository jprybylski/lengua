use std::collections::BTreeMap;

use lengua_core::{DiffTag, Store, TemplateMeta, diff_text, template};
use serde_yaml::Value;

fn meta(title: &str, fields: &[(&str, &str)]) -> TemplateMeta {
    let mut field_map = BTreeMap::new();
    for (k, v) in fields {
        field_map.insert(k.to_string(), Value::String(v.to_string()));
    }
    TemplateMeta {
        title: Some(title.to_string()),
        fields: field_map,
    }
}

#[test]
fn multi_store_lifecycle_and_organization() {
    let base_dir = tempfile::tempdir().unwrap();

    // Storekeeper manages 3 distinct domain stores
    let legal_path = base_dir.path().join("acme-legal");
    let eng_path = base_dir.path().join("acme-eng");
    let mktg_path = base_dir.path().join("acme-mktg");

    let legal_store = Store::init(&legal_path).unwrap();
    let eng_store = Store::init(&eng_path).unwrap();
    let mktg_store = Store::init(&mktg_path).unwrap();

    legal_store
        .add(
            "contracts/nda.md",
            &meta("Standard NDA", &[("category", "legal"), ("version", "1.0")]),
            "Mutual NDA between Acme and {{ party }}.\n",
            "add nda",
        )
        .unwrap();

    eng_store
        .add(
            "rfc/template.md",
            &meta(
                "RFC Template",
                &[("category", "engineering"), ("version", "2.1")],
            ),
            "# RFC: {{ title }}\n\nAuthor: {{ author }}\n",
            "add rfc",
        )
        .unwrap();

    mktg_store
        .add(
            "campaigns/launch.md",
            &meta(
                "Product Launch Email",
                &[("category", "marketing"), ("version", "1.0")],
            ),
            "Exciting news about {{ product }}!\n",
            "add launch email",
        )
        .unwrap();

    // Verify isolation and content
    assert_eq!(legal_store.list().unwrap().len(), 1);
    assert_eq!(eng_store.list().unwrap().len(), 1);
    assert_eq!(mktg_store.list().unwrap().len(), 1);

    assert_eq!(legal_store.list().unwrap()[0].name, "contracts/nda.md");
    assert_eq!(eng_store.list().unwrap()[0].name, "rfc/template.md");
    assert_eq!(mktg_store.list().unwrap()[0].name, "campaigns/launch.md");
}

#[test]
fn monorepo_multi_store_structure() {
    let monorepo_dir = tempfile::tempdir().unwrap();

    // Storekeeper organizes multiple stores under monorepo directories
    let store_a = Store::init(monorepo_dir.path().join("packages/store-a")).unwrap();
    let store_b = Store::init(monorepo_dir.path().join("packages/store-b")).unwrap();

    store_a
        .add("template_a.md", &meta("A", &[]), "Body A\n", "init A")
        .unwrap();
    store_b
        .add("template_b.md", &meta("B", &[]), "Body B\n", "init B")
        .unwrap();

    assert_eq!(store_a.list().unwrap()[0].name, "template_a.md");
    assert_eq!(store_b.list().unwrap()[0].name, "template_b.md");
}

#[test]
fn semantic_tagging_release_lifecycle() {
    let store_dir = tempfile::tempdir().unwrap();
    let store = Store::init(store_dir.path()).unwrap();

    // 1. Initial Release v1.0.0
    let commit_v1 = store
        .add(
            "reports/quarterly.md",
            &meta("Quarterly Report", &[("status", "stable")]),
            "Q1 Summary for {{ company }}.\n",
            "v1.0.0 release",
        )
        .unwrap();
    store
        .tag_create("reports/quarterly.md", "v1.0.0", None, false)
        .unwrap();
    store
        .tag_create("reports/quarterly.md", "latest", None, false)
        .unwrap();

    // 2. Second Release v2.0.0
    let commit_v2 = store
        .add(
            "reports/quarterly.md",
            &meta("Quarterly Report", &[("status", "stable")]),
            "Q2 Comprehensive Summary for {{ company }} with charts.\n",
            "v2.0.0 release",
        )
        .unwrap();
    store
        .tag_create("reports/quarterly.md", "v2.0.0", None, false)
        .unwrap();

    // Update 'latest' tag to point to v2.0.0 using force = true
    store
        .tag_create("reports/quarterly.md", "latest", None, true)
        .unwrap();

    // 3. Retroactively tag an older revision (e.g. v1.0.1 hotfix / alias)
    store
        .tag_create("reports/quarterly.md", "v1.0-legacy", Some("HEAD~1"), false)
        .unwrap();

    // 4. Verify tag listing
    let tags = store.tag_list("reports/quarterly.md").unwrap();
    let tag_names: Vec<_> = tags.iter().map(|t| t.tag.as_str()).collect();
    assert_eq!(tag_names, vec!["latest", "v1.0-legacy", "v1.0.0", "v2.0.0"]);

    // Verify tag commits
    let latest_tag = tags.iter().find(|t| t.tag == "latest").unwrap();
    assert_eq!(latest_tag.commit, commit_v2);

    let v1_legacy_tag = tags.iter().find(|t| t.tag == "v1.0-legacy").unwrap();
    assert_eq!(v1_legacy_tag.commit, commit_v1);

    // 5. Read at tagged revisions
    let body_latest = store
        .read_at_revision("reports/quarterly.md", "latest")
        .unwrap();
    assert!(body_latest.contains("Q2 Comprehensive Summary"));

    let body_v1 = store
        .read_at_revision("reports/quarterly.md", "v1.0.0")
        .unwrap();
    assert!(body_v1.contains("Q1 Summary for {{ company }}"));

    // 6. Delete a tag
    store
        .tag_remove("reports/quarterly.md", "v1.0-legacy")
        .unwrap();
    let updated_tags = store.tag_list("reports/quarterly.md").unwrap();
    assert!(!updated_tags.iter().any(|t| t.tag == "v1.0-legacy"));
}

#[test]
fn ci_cd_automated_store_validation_pipeline() {
    let store_dir = tempfile::tempdir().unwrap();
    let store = Store::init(store_dir.path()).unwrap();

    // Add valid templates
    store
        .add(
            "emails/welcome.md",
            &meta(
                "Welcome Email",
                &[("schema_version", "1"), ("category", "lifecycle")],
            ),
            "Hello {{ name }}, welcome to {{ team }}!\n",
            "add welcome",
        )
        .unwrap();

    store
        .add(
            "emails/reset-password.md",
            &meta(
                "Password Reset",
                &[("schema_version", "1"), ("category", "auth")],
            ),
            "Click here to reset your password: {{ link }}\n",
            "add reset",
        )
        .unwrap();

    // Storekeeper CI validation simulation:
    // 1. Iterate over every template in store
    let templates = store.list().unwrap();
    assert_eq!(templates.len(), 2);

    let mock_context: BTreeMap<String, String> = [
        ("name".to_string(), "Ada".to_string()),
        ("team".to_string(), "Engineering".to_string()),
        ("link".to_string(), "https://example.com/reset".to_string()),
    ]
    .into_iter()
    .collect();

    for entry in templates {
        // Validation check 1: Title must be present and non-empty
        assert!(entry.meta.title.is_some());
        assert!(!entry.meta.title.as_ref().unwrap().trim().is_empty());

        // Validation check 2: Required schema fields must be present
        assert!(entry.meta.fields.contains_key("schema_version"));
        assert!(entry.meta.fields.contains_key("category"));

        // Validation check 3: Jinja syntax must be valid and renderable
        let render_result = template::render(&entry.body, &mock_context);
        assert!(
            render_result.is_ok(),
            "Template {} failed rendering: {:?}",
            entry.name,
            render_result.err()
        );
    }
}

#[test]
fn cross_tag_diff_and_changelog_generation() {
    let store_dir = tempfile::tempdir().unwrap();
    let store = Store::init(store_dir.path()).unwrap();

    // Version 1
    store
        .add(
            "docs/getting-started.md",
            &meta("Getting Started", &[]),
            "Step 1: Install prerequisites.\nStep 2: Run build.\n",
            "v1.0.0",
        )
        .unwrap();
    store
        .tag_create("docs/getting-started.md", "v1.0.0", None, false)
        .unwrap();

    // Version 2
    store
        .add(
            "docs/getting-started.md",
            &meta("Getting Started", &[]),
            "Step 1: Install prerequisites.\nStep 2: Configure environment.\nStep 3: Run build.\n",
            "v2.0.0",
        )
        .unwrap();
    store
        .tag_create("docs/getting-started.md", "v2.0.0", None, false)
        .unwrap();

    // Storekeeper generates diff between v1.0.0 and v2.0.0
    let old_text = store
        .read_at_revision("docs/getting-started.md", "v1.0.0")
        .unwrap();
    let new_text = store
        .read_at_revision("docs/getting-started.md", "v2.0.0")
        .unwrap();

    let diff = diff_text(&old_text, &new_text);
    let inserted: Vec<_> = diff
        .iter()
        .filter(|l| l.tag == DiffTag::Insert)
        .map(|l| l.line.as_str())
        .collect();
    let deleted: Vec<_> = diff
        .iter()
        .filter(|l| l.tag == DiffTag::Delete)
        .map(|l| l.line.as_str())
        .collect();

    assert!(inserted.contains(&"Step 2: Configure environment."));
    assert!(inserted.contains(&"Step 3: Run build."));
    assert!(deleted.contains(&"Step 2: Run build."));
}
