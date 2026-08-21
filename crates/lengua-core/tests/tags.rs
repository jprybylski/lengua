use lengua_core::{Store, TemplateMeta};

fn meta(title: &str) -> TemplateMeta {
    TemplateMeta {
        title: Some(title.to_string()),
        fields: Default::default(),
    }
}

#[test]
fn tag_create_list_remove_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path()).unwrap();
    store
        .add("letter.md", &meta("v1"), "First version.\n", "v1")
        .unwrap();

    let entry = store.tag_create("letter.md", "final", None, false).unwrap();
    assert_eq!(entry.tag, "final");

    let tags = store.tag_list("letter.md").unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].tag, "final");
    assert_eq!(tags[0].commit, entry.commit);

    store.tag_remove("letter.md", "final").unwrap();
    assert!(store.tag_list("letter.md").unwrap().is_empty());
}

#[test]
fn tag_create_refuses_overwrite_without_force() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path()).unwrap();
    store
        .add("letter.md", &meta("v1"), "First version.\n", "v1")
        .unwrap();

    store.tag_create("letter.md", "final", None, false).unwrap();
    assert!(store.tag_create("letter.md", "final", None, false).is_err());
    assert!(store.tag_create("letter.md", "final", None, true).is_ok());
}

#[test]
fn tag_create_rejects_reserved_names() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path()).unwrap();
    store
        .add("letter.md", &meta("v1"), "First version.\n", "v1")
        .unwrap();

    assert!(store.tag_create("letter.md", "HEAD", None, false).is_err());
    assert!(store.tag_create("letter.md", "head", None, false).is_err());
    assert!(
        store
            .tag_create("letter.md", "deadbeef", None, false)
            .is_err()
    );
}

#[test]
fn retroactive_tag_and_tag_aware_revision_resolution() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path()).unwrap();

    store
        .add(
            "letter.md",
            &meta("v1"),
            "We had so much fun.\n",
            "past tense",
        )
        .unwrap();
    store
        .add(
            "letter.md",
            &meta("v2"),
            "We will have so much fun.\n",
            "future tense",
        )
        .unwrap();

    store
        .tag_create("letter.md", "tense-future", None, false)
        .unwrap();
    store
        .tag_create("letter.md", "tense-past", Some("HEAD~1"), false)
        .unwrap();

    let future = store.get_at_revision("letter.md", "tense-future").unwrap();
    let past = store.get_at_revision("letter.md", "tense-past").unwrap();
    assert!(future.body.contains("will have"));
    assert!(past.body.contains("We had"));

    let diff = lengua_core::diff_text(&past.body, &future.body);
    assert!(diff.iter().any(|l| l.tag == lengua_core::DiffTag::Insert));
}

#[test]
fn tag_rm_missing_tag_errors() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path()).unwrap();
    store
        .add("letter.md", &meta("v1"), "First version.\n", "v1")
        .unwrap();
    assert!(store.tag_remove("letter.md", "nope").is_err());
}

#[test]
fn open_rejects_git_repo_without_templates_dir() {
    let dir = tempfile::tempdir().unwrap();
    gix::init(dir.path()).unwrap();
    let Err(err) = Store::open(dir.path()) else {
        panic!("expected Store::open to fail without a templates/ dir");
    };
    assert!(matches!(err, lengua_core::Error::NotAStore(_)));
}
