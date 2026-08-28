use std::collections::BTreeMap;

use lengua_core::{Library, Query, Store, TemplateMeta, UpdateStatus};
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
fn pure_consumer_single_upstream_lifecycle() {
    // 1. Storekeeper initializes and populates the upstream organization store
    let upstream_dir = tempfile::tempdir().unwrap();
    let upstream_store = Store::init(upstream_dir.path()).unwrap();
    upstream_store
        .add(
            "guidelines/code-of-conduct.md",
            &meta("Code of Conduct", &[("dept", "hr"), ("status", "active")]),
            "All members of {{ org }} agree to treat everyone with respect.\n",
            "initial commit",
        )
        .unwrap();
    upstream_store
        .tag_create("guidelines/code-of-conduct.md", "v1.0", None, false)
        .unwrap();

    // 2. End-user sets up a pure consumer library (no local source created)
    let consumer_dir = tempfile::tempdir().unwrap();
    let library = Library::init_from(
        consumer_dir.path(),
        Some("org-core"),
        Some(upstream_dir.path()),
        None,
        None,
        None,
        false,
    )
    .unwrap();

    // Assert only org-core exists (no empty "local" source was created)
    assert_eq!(library.manifest_order(), vec!["org-core".to_string()]);
    assert!(!consumer_dir.path().join(".lengua/local").exists());

    // 3. Consumer queries templates
    let list = library.list(None).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].0.name, "guidelines/code-of-conduct.md");
    assert_eq!(list[0].1, "org-core");

    let (entry, source) = library
        .get(None, "guidelines/code-of-conduct.md", None)
        .unwrap();
    assert_eq!(source, "org-core");
    assert!(entry.body.contains("All members of {{ org }}"));

    // Verify search works cleanly in pure-consumer mode
    let q = Query::new().with("dept", "hr");
    let search_results = library.search(None, &q).unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(
        search_results[0].0.meta.title.as_deref(),
        Some("Code of Conduct")
    );

    // 4. Upstream updates the template and adds a new tag
    upstream_store
        .add(
            "guidelines/code-of-conduct.md",
            &meta("Code of Conduct", &[("dept", "hr"), ("status", "active")]),
            "All members of {{ org }} agree to treat everyone with dignity and respect.\n",
            "add dignity",
        )
        .unwrap();
    upstream_store
        .tag_create("guidelines/code-of-conduct.md", "v2.0", None, false)
        .unwrap();

    // 5. Consumer updates the library
    let update_results = library.update(None);
    assert_eq!(update_results.len(), 1);
    assert_eq!(update_results[0].0, "org-core");
    match &update_results[0].1 {
        Ok(UpdateStatus::FastForwarded { .. }) => {}
        other => panic!("expected FastForwarded, got {other:?}"),
    }

    // 6. Consumer sees updated content in working tree and can access both tags
    let library = Library::open(consumer_dir.path()).unwrap();
    let (v2_entry, _) = library
        .get(None, "guidelines/code-of-conduct.md", None)
        .unwrap();
    assert!(v2_entry.body.contains("dignity and respect"));

    let (v1_entry, _) = library
        .get(None, "guidelines/code-of-conduct.md", Some("v1.0"))
        .unwrap();
    assert!(
        v1_entry
            .body
            .contains("All members of {{ org }} agree to treat everyone with respect")
    );
    assert!(!v1_entry.body.contains("dignity"));

    let (tagged_v2_entry, _) = library
        .get(None, "guidelines/code-of-conduct.md", Some("v2.0"))
        .unwrap();
    assert!(tagged_v2_entry.body.contains("dignity and respect"));
}

#[test]
fn three_tier_upstream_layering_precedence_and_shadowing() {
    // Layer 1: Corporate Standards (base)
    let corp_dir = tempfile::tempdir().unwrap();
    let corp_store = Store::init(corp_dir.path()).unwrap();
    corp_store
        .add(
            "welcome.md",
            &meta("Welcome to Company", &[("tier", "corp")]),
            "Welcome to Acme Corp, {{ name }}!\n",
            "corp welcome",
        )
        .unwrap();
    corp_store
        .add(
            "footer.md",
            &meta("Corporate Footer", &[("tier", "corp")]),
            "Copyright (C) Acme Corp.\n",
            "corp footer",
        )
        .unwrap();
    corp_store
        .add(
            "legal/nda.md",
            &meta("Standard NDA", &[("tier", "corp")]),
            "NDA terms and conditions for {{ recipient }}.\n",
            "corp nda",
        )
        .unwrap();

    // Layer 2: Department Templates (e.g. Sales overrides welcome.md, adds pitch.md)
    let dept_dir = tempfile::tempdir().unwrap();
    let dept_store = Store::init(dept_dir.path()).unwrap();
    dept_store
        .add(
            "welcome.md",
            &meta("Sales Onboarding", &[("tier", "sales")]),
            "Welcome to Acme Sales Team, {{ name }}! Let's close deals.\n",
            "sales welcome",
        )
        .unwrap();
    dept_store
        .add(
            "sales/pitch.md",
            &meta("Sales Pitch", &[("tier", "sales")]),
            "Hi {{ client }}, here is our proposal for {{ product }}.\n",
            "sales pitch",
        )
        .unwrap();

    // Layer 3: Project Local Templates (e.g. overrides footer.md, adds summary.md)
    let project_dir = tempfile::tempdir().unwrap();
    let project_store = Store::init(project_dir.path()).unwrap();
    project_store
        .add(
            "footer.md",
            &meta("Project Footer", &[("tier", "project")]),
            "Acme Project Alpha confidential footer.\n",
            "project footer",
        )
        .unwrap();
    project_store
        .add(
            "project/summary.md",
            &meta("Project Summary", &[("tier", "project")]),
            "Status update for Sprint {{ sprint }}.\n",
            "project summary",
        )
        .unwrap();

    // End-User initializes library with Corp Standards, then fetches Sales, then fetches Project Local
    let consumer_dir = tempfile::tempdir().unwrap();
    let mut library = Library::init_from(
        consumer_dir.path(),
        Some("corp"),
        Some(corp_dir.path()),
        None,
        None,
        None,
        false,
    )
    .unwrap();

    let fetch_dept = library
        .fetch(
            Some("sales"),
            Some(dept_dir.path()),
            None,
            None,
            None,
            false,
        )
        .unwrap();
    // Sales shadows welcome.md from corp
    assert_eq!(fetch_dept.warnings.len(), 1);
    assert_eq!(fetch_dept.warnings[0].name, "welcome.md");
    assert_eq!(fetch_dept.warnings[0].winner, "sales");
    assert_eq!(fetch_dept.warnings[0].loser, "corp");

    let fetch_proj = library
        .fetch(
            Some("project"),
            Some(project_dir.path()),
            None,
            None,
            None,
            false,
        )
        .unwrap();
    // Project shadows footer.md from corp
    assert_eq!(fetch_proj.warnings.len(), 1);
    assert_eq!(fetch_proj.warnings[0].name, "footer.md");
    assert_eq!(fetch_proj.warnings[0].winner, "project");
    assert_eq!(fetch_proj.warnings[0].loser, "corp");

    // Reopen and check manifest order (lowest to highest precedence)
    let library = Library::open(consumer_dir.path()).unwrap();
    assert_eq!(
        library.manifest_order(),
        vec![
            "corp".to_string(),
            "sales".to_string(),
            "project".to_string()
        ]
    );

    // Verify shadow warnings on the library
    let all_warnings = library.shadow_warnings();
    assert_eq!(all_warnings.len(), 2);

    // Verify Merged Unscoped Reads:
    // 1. welcome.md -> winner is 'sales'
    let (welcome_entry, welcome_src) = library.get(None, "welcome.md", None).unwrap();
    assert_eq!(welcome_src, "sales");
    assert!(welcome_entry.body.contains("Acme Sales Team"));

    // 2. footer.md -> winner is 'project'
    let (footer_entry, footer_src) = library.get(None, "footer.md", None).unwrap();
    assert_eq!(footer_src, "project");
    assert!(
        footer_entry
            .body
            .contains("Project Alpha confidential footer")
    );

    // 3. legal/nda.md -> winner is 'corp' (not shadowed)
    let (nda_entry, nda_src) = library.get(None, "legal/nda.md", None).unwrap();
    assert_eq!(nda_src, "corp");
    assert!(nda_entry.body.contains("NDA terms"));

    // 4. sales/pitch.md -> winner is 'sales'
    let (_pitch_entry, pitch_src) = library.get(None, "sales/pitch.md", None).unwrap();
    assert_eq!(pitch_src, "sales");

    // 5. project/summary.md -> winner is 'project'
    let (_summary_entry, summary_src) = library.get(None, "project/summary.md", None).unwrap();
    assert_eq!(summary_src, "project");

    // Verify Explicit Scoped Reads bypass the merge:
    // Reach the shadowed corp version of welcome.md
    let (corp_welcome, corp_src) = library.get(Some("corp"), "welcome.md", None).unwrap();
    assert_eq!(corp_src, "corp");
    assert!(corp_welcome.body.contains("Welcome to Acme Corp"));

    // Reach the shadowed corp version of footer.md
    let (corp_footer, corp_src) = library.get(Some("corp"), "footer.md", None).unwrap();
    assert_eq!(corp_src, "corp");
    assert!(corp_footer.body.contains("Copyright (C) Acme Corp"));

    // Verify Merged List contains exactly 5 winning templates
    let list = library.list(None).unwrap();
    assert_eq!(list.len(), 5);
    let names_and_sources: BTreeMap<String, String> = list
        .into_iter()
        .map(|(entry, src)| (entry.name, src))
        .collect();
    assert_eq!(names_and_sources.get("welcome.md").unwrap(), "sales");
    assert_eq!(names_and_sources.get("footer.md").unwrap(), "project");
    assert_eq!(names_and_sources.get("legal/nda.md").unwrap(), "corp");
    assert_eq!(names_and_sources.get("sales/pitch.md").unwrap(), "sales");
    assert_eq!(
        names_and_sources.get("project/summary.md").unwrap(),
        "project"
    );

    // Verify Search across layers
    let corp_only = library
        .search(None, &Query::new().with("tier", "corp"))
        .unwrap();
    // Only legal/nda.md should match because corp's welcome and footer are shadowed by sales and project!
    assert_eq!(corp_only.len(), 1);
    assert_eq!(corp_only[0].0.name, "legal/nda.md");

    let sales_matches = library
        .search(None, &Query::new().with("tier", "sales"))
        .unwrap();
    assert_eq!(sales_matches.len(), 2); // welcome.md and sales/pitch.md
}

#[test]
fn upstream_update_propagation_across_multiple_layers() {
    let corp_dir = tempfile::tempdir().unwrap();
    let corp_store = Store::init(corp_dir.path()).unwrap();
    corp_store
        .add("terms.md", &meta("Terms", &[]), "Terms v1\n", "v1")
        .unwrap();

    let dept_dir = tempfile::tempdir().unwrap();
    let dept_store = Store::init(dept_dir.path()).unwrap();
    dept_store
        .add("notice.md", &meta("Notice", &[]), "Notice v1\n", "v1")
        .unwrap();

    let consumer_dir = tempfile::tempdir().unwrap();
    let mut library = Library::init_from(
        consumer_dir.path(),
        Some("corp"),
        Some(corp_dir.path()),
        None,
        None,
        None,
        false,
    )
    .unwrap();
    library
        .fetch(Some("dept"), Some(dept_dir.path()), None, None, None, false)
        .unwrap();

    // Upstream 1 releases terms.md v2 + tags it
    corp_store
        .add("terms.md", &meta("Terms", &[]), "Terms v2\n", "v2")
        .unwrap();
    corp_store
        .tag_create("terms.md", "v2.0", None, false)
        .unwrap();

    // Upstream 2 adds a brand new template
    dept_store
        .add(
            "announcement.md",
            &meta("Announce", &[]),
            "Big news!\n",
            "new",
        )
        .unwrap();

    // Consumer runs update across all sources
    let update_results = library.update(None);
    assert_eq!(update_results.len(), 2);
    for (name, result) in update_results {
        match result {
            Ok(UpdateStatus::FastForwarded { .. }) => {}
            other => panic!("source {name} expected FastForwarded, got {other:?}"),
        }
    }

    // Reopen and check that all updates and new templates are visible
    let library = Library::open(consumer_dir.path()).unwrap();
    let list = library.list(None).unwrap();
    assert_eq!(list.len(), 3);

    let (terms, _) = library.get(None, "terms.md", None).unwrap();
    assert!(terms.body.contains("Terms v2"));

    let (terms_v2_tag, _) = library.get(None, "terms.md", Some("v2.0")).unwrap();
    assert!(terms_v2_tag.body.contains("Terms v2"));

    let (announcement, src) = library.get(None, "announcement.md", None).unwrap();
    assert_eq!(src, "dept");
    assert!(announcement.body.contains("Big news!"));
}

#[test]
fn monorepo_subdirectory_slice_and_reimport() {
    let monorepo = tempfile::tempdir().unwrap();
    let store_root = monorepo.path();

    // Create a monorepo with multiple store subdirectories
    let legal_dir = store_root.join("stores/legal");
    let eng_dir = store_root.join("stores/engineering");

    let legal_store = Store::init(&legal_dir).unwrap();
    legal_store
        .add("contract.md", &meta("Contract", &[]), "Contract v1\n", "c1")
        .unwrap();

    let eng_store = Store::init(&eng_dir).unwrap();
    eng_store
        .add(
            "pr-template.md",
            &meta("PR Template", &[]),
            "PR checklist\n",
            "p1",
        )
        .unwrap();

    // Consumer adopts only the legal subdirectory
    let consumer_dir = tempfile::tempdir().unwrap();
    let library = Library::init_from(
        consumer_dir.path(),
        Some("legal-slice"),
        Some(&legal_dir),
        None,
        None,
        Some("."),
        false,
    )
    .unwrap();

    let list = library.list(None).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].0.name, "contract.md");
}
