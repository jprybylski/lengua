use std::path::Path;

use lengua_core::{Error, Library, Query, Store, TemplateMeta, UpdateStatus};

fn meta(title: &str) -> TemplateMeta {
    TemplateMeta {
        title: Some(title.to_string()),
        fields: Default::default(),
    }
}

fn default_branch_name(repo_path: &Path) -> String {
    gix::open(repo_path)
        .unwrap()
        .head_ref()
        .unwrap()
        .unwrap()
        .name()
        .shorten()
        .to_string()
}

#[test]
fn init_creates_dot_lengua_with_default_local_source() {
    let dir = tempfile::tempdir().unwrap();
    Library::init(dir.path(), None).unwrap();

    assert!(dir.path().join(".lengua/sources.toml").is_file());
    assert!(dir.path().join(".lengua/local/.git").is_dir());
    assert!(dir.path().join(".lengua/local/templates").is_dir());
    assert!(!dir.path().join(".git").exists());
    assert!(!dir.path().join("templates").exists());
}

#[test]
fn init_with_custom_name_via_name_flag() {
    let dir = tempfile::tempdir().unwrap();
    Library::init(dir.path(), Some("mine")).unwrap();

    assert!(dir.path().join(".lengua/mine/.git").is_dir());
    assert!(!dir.path().join(".lengua/local").exists());
}

#[test]
fn init_from_dir_pure_consumer_mode_creates_no_local_source() {
    let source_dir = tempfile::tempdir().unwrap();
    let source = Store::init(source_dir.path()).unwrap();
    source.add("a.md", &meta("A"), "hi\n", "add").unwrap();

    let dir = tempfile::tempdir().unwrap();
    let library = Library::init_from(
        dir.path(),
        None,
        Some(source_dir.path()),
        None,
        None,
        None,
        false,
    )
    .unwrap();

    let names: Vec<_> = library
        .list(None)
        .unwrap()
        .into_iter()
        .map(|(_, source)| source)
        .collect();
    assert_eq!(names.len(), 1);
    assert!(!dir.path().join(".lengua/local").exists());
}

#[test]
fn init_rejects_existing_dot_lengua() {
    let dir = tempfile::tempdir().unwrap();
    Library::init(dir.path(), None).unwrap();
    assert!(matches!(
        Library::init(dir.path(), None),
        Err(Error::AlreadyInitialized(_))
    ));
}

#[test]
fn open_rejects_missing_dot_lengua() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        Library::open(dir.path()),
        Err(Error::NotALibrary(_))
    ));
}

#[test]
fn open_rejects_old_flat_layout() {
    // The pre-#3 layout: `.git` + `templates/` directly at the root, no `.lengua/` at all.
    let dir = tempfile::tempdir().unwrap();
    Store::init(dir.path()).unwrap();
    assert!(matches!(
        Library::open(dir.path()),
        Err(Error::NotALibrary(_))
    ));
}

#[test]
fn fetch_requires_existing_dot_lengua() {
    let source_dir = tempfile::tempdir().unwrap();
    Store::init(source_dir.path()).unwrap();

    let dir = tempfile::tempdir().unwrap();
    // No init() first -- Library::open would fail; fetch is only reachable on an open Library,
    // so this proves the CLI-level guidance path (commands/fetch.rs) has something to catch.
    assert!(matches!(
        Library::open(dir.path()),
        Err(Error::NotALibrary(_))
    ));
}

#[test]
fn fetch_appends_second_source_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let mut library = Library::init(dir.path(), None).unwrap();

    let source_dir = tempfile::tempdir().unwrap();
    let source = Store::init(source_dir.path()).unwrap();
    source.add("a.md", &meta("A"), "hi\n", "add").unwrap();

    let outcome = library
        .fetch(
            Some("extra"),
            Some(source_dir.path()),
            None,
            None,
            None,
            false,
        )
        .unwrap();
    assert_eq!(outcome.source, "extra");
    assert!(outcome.warnings.is_empty());

    let library = Library::open(dir.path()).unwrap();
    let order: Vec<_> = library.manifest_order();
    assert_eq!(order, vec!["local".to_string(), "extra".to_string()]);
}

#[test]
fn fetch_auto_names_from_from_dir_basename() {
    let dir = tempfile::tempdir().unwrap();
    let mut library = Library::init(dir.path(), None).unwrap();

    let parent = tempfile::tempdir().unwrap();
    let source_dir = parent.path().join("acme-templates");
    let source = Store::init(&source_dir).unwrap();
    source.add("a.md", &meta("A"), "hi\n", "add").unwrap();

    let outcome = library
        .fetch(None, Some(&source_dir), None, None, None, false)
        .unwrap();
    assert_eq!(outcome.source, "acme-templates");
}

#[test]
fn fetch_auto_names_from_from_repo_local_path() {
    let dir = tempfile::tempdir().unwrap();
    let mut library = Library::init(dir.path(), None).unwrap();

    let parent = tempfile::tempdir().unwrap();
    let source_dir = parent.path().join("team-templates");
    let source = Store::init(&source_dir).unwrap();
    source.add("a.md", &meta("A"), "hi\n", "add").unwrap();
    let url = format!("file://{}", source_dir.display());

    let outcome = library
        .fetch(None, None, Some(&url), None, None, false)
        .unwrap();
    assert_eq!(outcome.source, "team-templates");
}

#[test]
fn fetch_name_collision_errors_asking_for_explicit_name() {
    let dir = tempfile::tempdir().unwrap();
    let mut library = Library::init(dir.path(), None).unwrap();

    let parent1 = tempfile::tempdir().unwrap();
    let source1 = parent1.path().join("shared");
    let store1 = Store::init(&source1).unwrap();
    store1.add("a.md", &meta("A"), "1\n", "add").unwrap();
    library
        .fetch(None, Some(&source1), None, None, None, false)
        .unwrap();

    let parent2 = tempfile::tempdir().unwrap();
    let source2 = parent2.path().join("shared");
    let store2 = Store::init(&source2).unwrap();
    store2.add("b.md", &meta("B"), "2\n", "add").unwrap();
    let err = library
        .fetch(None, Some(&source2), None, None, None, false)
        .unwrap_err();
    assert!(matches!(err, Error::DuplicateSourceName { .. }));
}

#[test]
fn add_defaults_to_sole_source() {
    let dir = tempfile::tempdir().unwrap();
    let library = Library::init(dir.path(), None).unwrap();
    library
        .add(None, "a.md", &meta("A"), "hi\n", "add")
        .unwrap();
    let (entry, source) = library.get(None, "a.md", None).unwrap();
    assert_eq!(source, "local");
    assert_eq!(entry.meta.title.as_deref(), Some("A"));
}

#[test]
fn add_requires_source_when_multiple_sources_exist() {
    let dir = tempfile::tempdir().unwrap();
    let mut library = Library::init(dir.path(), None).unwrap();
    let source_dir = tempfile::tempdir().unwrap();
    Store::init(source_dir.path()).unwrap();
    library
        .fetch(
            Some("extra"),
            Some(source_dir.path()),
            None,
            None,
            None,
            false,
        )
        .unwrap();

    let err = library
        .add(None, "a.md", &meta("A"), "hi\n", "add")
        .unwrap_err();
    assert!(matches!(err, Error::AmbiguousSource { .. }));

    library
        .add(Some("local"), "a.md", &meta("A"), "hi\n", "add")
        .unwrap();
    let (entry, source) = library.get(Some("local"), "a.md", None).unwrap();
    assert_eq!(source, "local");
    assert_eq!(entry.meta.title.as_deref(), Some("A"));
}

#[test]
fn get_list_search_merge_with_last_fetched_wins() {
    let dir = tempfile::tempdir().unwrap();
    let mut library = Library::init(dir.path(), None).unwrap();
    library
        .add(None, "shared.md", &meta("Local"), "local body\n", "add")
        .unwrap();

    let source_dir = tempfile::tempdir().unwrap();
    let source = Store::init(source_dir.path()).unwrap();
    source
        .add("shared.md", &meta("Fetched"), "fetched body\n", "add")
        .unwrap();
    library
        .fetch(
            Some("extra"),
            Some(source_dir.path()),
            None,
            None,
            None,
            false,
        )
        .unwrap();

    let (entry, source_name) = library.get(None, "shared.md", None).unwrap();
    assert_eq!(source_name, "extra");
    assert_eq!(entry.meta.title.as_deref(), Some("Fetched"));

    let results = library.search(None, &Query::new()).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, "extra");
}

#[test]
fn name_collision_emits_shadow_warning_on_fetch() {
    let dir = tempfile::tempdir().unwrap();
    let mut library = Library::init(dir.path(), None).unwrap();
    library
        .add(None, "shared.md", &meta("Local"), "local body\n", "add")
        .unwrap();

    let source_dir = tempfile::tempdir().unwrap();
    let source = Store::init(source_dir.path()).unwrap();
    source
        .add("shared.md", &meta("Fetched"), "fetched body\n", "add")
        .unwrap();
    let outcome = library
        .fetch(
            Some("extra"),
            Some(source_dir.path()),
            None,
            None,
            None,
            false,
        )
        .unwrap();

    assert_eq!(outcome.warnings.len(), 1);
    assert_eq!(outcome.warnings[0].name, "shared.md");
    assert_eq!(outcome.warnings[0].winner, "extra");
    assert_eq!(outcome.warnings[0].loser, "local");

    assert_eq!(library.shadow_warnings().len(), 1);
}

#[test]
fn list_and_search_report_source_per_entry() {
    let dir = tempfile::tempdir().unwrap();
    let mut library = Library::init(dir.path(), None).unwrap();
    library
        .add(None, "a.md", &meta("A"), "hi\n", "add")
        .unwrap();

    let source_dir = tempfile::tempdir().unwrap();
    let source = Store::init(source_dir.path()).unwrap();
    source.add("b.md", &meta("B"), "hi\n", "add").unwrap();
    library
        .fetch(
            Some("extra"),
            Some(source_dir.path()),
            None,
            None,
            None,
            false,
        )
        .unwrap();

    let mut all = library.list(None).unwrap();
    all.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    assert_eq!(all[0].1, "local");
    assert_eq!(all[1].1, "extra");
}

#[test]
fn get_with_explicit_source_bypasses_merge() {
    let dir = tempfile::tempdir().unwrap();
    let mut library = Library::init(dir.path(), None).unwrap();
    library
        .add(None, "shared.md", &meta("Local"), "local body\n", "add")
        .unwrap();

    let source_dir = tempfile::tempdir().unwrap();
    let source = Store::init(source_dir.path()).unwrap();
    source
        .add("shared.md", &meta("Fetched"), "fetched body\n", "add")
        .unwrap();
    library
        .fetch(
            Some("extra"),
            Some(source_dir.path()),
            None,
            None,
            None,
            false,
        )
        .unwrap();

    // Unscoped resolves to the shadowing winner ("extra"); explicit --source reaches the
    // shadowed "local" copy directly.
    let (entry, source_name) = library.get(Some("local"), "shared.md", None).unwrap();
    assert_eq!(source_name, "local");
    assert_eq!(entry.meta.title.as_deref(), Some("Local"));
}

#[test]
fn update_fast_forwards_a_cloned_source() {
    let origin_dir = tempfile::tempdir().unwrap();
    let origin = Store::init(origin_dir.path()).unwrap();
    origin.add("a.md", &meta("A"), "v1\n", "v1").unwrap();
    let branch = default_branch_name(origin_dir.path());
    let url = format!("file://{}", origin_dir.path().display());

    let dir = tempfile::tempdir().unwrap();
    let library = Library::init_from(
        dir.path(),
        Some("remote"),
        None,
        Some(&url),
        Some(&branch),
        None,
        false,
    )
    .unwrap();

    origin.add("b.md", &meta("B"), "v2\n", "v2").unwrap();

    let results = library.update(None);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "remote");
    assert!(matches!(
        results[0].1,
        Ok(UpdateStatus::FastForwarded { .. })
    ));

    let library = Library::open(dir.path()).unwrap();
    let (entry, source) = library.get(Some("remote"), "b.md", None).unwrap();
    assert_eq!(source, "remote");
    assert!(entry.body.contains("v2"));
}

#[test]
fn update_fast_forwards_a_copied_source() {
    let origin_dir = tempfile::tempdir().unwrap();
    let origin = Store::init(origin_dir.path()).unwrap();
    origin.add("a.md", &meta("A"), "v1\n", "v1").unwrap();

    let dir = tempfile::tempdir().unwrap();
    let library = Library::init_from(
        dir.path(),
        Some("copy"),
        Some(origin_dir.path()),
        None,
        None,
        None,
        false,
    )
    .unwrap();

    origin.add("b.md", &meta("B"), "v2\n", "v2").unwrap();

    let results = library.update(None);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "copy");
    assert!(matches!(
        results[0].1,
        Ok(UpdateStatus::FastForwarded { .. })
    ));

    let library = Library::open(dir.path()).unwrap();
    let (entry, source) = library.get(Some("copy"), "b.md", None).unwrap();
    assert_eq!(source, "copy");
    assert!(entry.body.contains("v2"));
}

#[test]
fn update_rejects_local_source() {
    let dir = tempfile::tempdir().unwrap();
    let library = Library::init(dir.path(), None).unwrap();
    let results = library.update(None);
    assert_eq!(results.len(), 1);
    assert!(matches!(
        &results[0].1,
        Err(Error::SourceNotUpdatable { .. })
    ));
}

#[test]
fn update_rejects_subdir_imported_source() {
    let origin_dir = tempfile::tempdir().unwrap();
    let origin = Store::init(origin_dir.path()).unwrap();
    origin.add("a.md", &meta("A"), "v1\n", "v1").unwrap();

    let dir = tempfile::tempdir().unwrap();
    let library = Library::init_from(
        dir.path(),
        Some("sub"),
        Some(origin_dir.path()),
        None,
        None,
        Some("."),
        false,
    )
    .unwrap();

    let results = library.update(None);
    assert_eq!(results.len(), 1);
    assert!(matches!(
        &results[0].1,
        Err(Error::SourceNotUpdatable { .. })
    ));
}

#[test]
fn update_fails_loudly_on_divergence() {
    let origin_dir = tempfile::tempdir().unwrap();
    let origin = Store::init(origin_dir.path()).unwrap();
    origin.add("a.md", &meta("A"), "v1\n", "v1").unwrap();

    let dir = tempfile::tempdir().unwrap();
    let library = Library::init_from(
        dir.path(),
        Some("copy"),
        Some(origin_dir.path()),
        None,
        None,
        None,
        false,
    )
    .unwrap();

    // Origin and the local copy diverge onto different commits from the same base.
    origin.add("b.md", &meta("B"), "v2\n", "v2").unwrap();
    let local_copy = Store::open(dir.path().join(".lengua").join("copy")).unwrap();
    local_copy.add("c.md", &meta("C"), "v3\n", "v3").unwrap();

    let before_head = gix::open(dir.path().join(".lengua").join("copy"))
        .unwrap()
        .head_id()
        .unwrap()
        .detach();

    let results = library.update(None);
    assert_eq!(results.len(), 1);
    match &results[0].1 {
        Err(Error::NotFastForward { name }) => assert_eq!(name, "copy"),
        other => panic!("expected NotFastForward, got {other:?}"),
    }

    let after_head = gix::open(dir.path().join(".lengua").join("copy"))
        .unwrap()
        .head_id()
        .unwrap()
        .detach();
    assert_eq!(before_head, after_head);
    assert!(local_copy.get("c.md").is_ok());
}

#[test]
fn update_all_reports_every_source_even_after_one_failure() {
    let dir = tempfile::tempdir().unwrap();
    let mut library = Library::init(dir.path(), None).unwrap();

    let origin_dir = tempfile::tempdir().unwrap();
    let origin = Store::init(origin_dir.path()).unwrap();
    origin.add("a.md", &meta("A"), "v1\n", "v1").unwrap();
    library
        .fetch(
            Some("copy"),
            Some(origin_dir.path()),
            None,
            None,
            None,
            false,
        )
        .unwrap();

    let results = library.update(None);
    let names: Vec<_> = results.iter().map(|(name, _)| name.clone()).collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"local".to_string()));
    assert!(names.contains(&"copy".to_string()));

    let local_result = results.iter().find(|(name, _)| name == "local").unwrap();
    assert!(matches!(
        local_result.1,
        Err(Error::SourceNotUpdatable { .. })
    ));
    let copy_result = results.iter().find(|(name, _)| name == "copy").unwrap();
    assert!(matches!(copy_result.1, Ok(UpdateStatus::UpToDate)));
}
