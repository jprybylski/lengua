use lengua_core::{Query, Store, TemplateMeta};

fn meta(title: &str) -> TemplateMeta {
    TemplateMeta {
        title: Some(title.to_string()),
        fields: Default::default(),
    }
}

#[test]
fn init_add_get_list_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path()).unwrap();

    store
        .add(
            "greetings/hello.md",
            &meta("Hello"),
            "Dear {{ name }},\n",
            "add hello",
        )
        .unwrap();
    store
        .add("bye.md", &meta("Bye"), "Farewell, {{ name }}.\n", "add bye")
        .unwrap();

    let hello = store.get("greetings/hello.md").unwrap();
    assert_eq!(hello.meta.title.as_deref(), Some("Hello"));
    assert_eq!(hello.body.trim_end(), "Dear {{ name }},");

    let all = store.list().unwrap();
    assert_eq!(all.len(), 2);
    let names: Vec<_> = all.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"greetings/hello.md"));
    assert!(names.contains(&"bye.md"));
}

#[test]
fn search_filters_by_frontmatter_field() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path()).unwrap();

    let mut formal = meta("Formal Letter");
    formal.fields.insert(
        "tone".to_string(),
        serde_yaml::Value::String("formal".into()),
    );
    let mut casual = meta("Casual Note");
    casual.fields.insert(
        "tone".to_string(),
        serde_yaml::Value::String("casual".into()),
    );

    store
        .add("formal.md", &formal, "Dear Sir,\n", "add formal")
        .unwrap();
    store
        .add("casual.md", &casual, "Hey!\n", "add casual")
        .unwrap();

    let all = store.list().unwrap();
    let query = Query::new().with("tone", "formal");
    let matched: Vec<_> = all.iter().filter(|e| query.matches(&e.meta)).collect();

    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].name, "formal.md");
}

#[test]
fn add_creates_real_git_commits_and_history() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path()).unwrap();

    store
        .add("letter.md", &meta("v1"), "First version.\n", "v1")
        .unwrap();
    store
        .add("letter.md", &meta("v2"), "Second version.\n", "v2")
        .unwrap();

    let log = store.log("letter.md").unwrap();
    assert_eq!(
        log.len(),
        2,
        "expected two commits touching letter.md: {log:?}"
    );
    assert_eq!(log[0].message, "v2");
    assert_eq!(log[1].message, "v1");

    let repo = gix::open(dir.path()).unwrap();
    assert!(repo.head_id().is_ok());
}

#[test]
fn read_at_revision_returns_historical_content() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path()).unwrap();

    store
        .add("letter.md", &meta("v1"), "First version.\n", "v1")
        .unwrap();
    store
        .add("letter.md", &meta("v2"), "Second version.\n", "v2")
        .unwrap();

    let old = store.read_at_revision("letter.md", "HEAD~1").unwrap();
    let new = store.read_at_revision("letter.md", "HEAD").unwrap();

    assert!(old.contains("First version."));
    assert!(new.contains("Second version."));

    let diff = lengua_core::diff_text(&old, &new);
    assert!(diff.iter().any(|l| l.tag == lengua_core::DiffTag::Delete));
    assert!(diff.iter().any(|l| l.tag == lengua_core::DiffTag::Insert));
}

#[test]
fn open_rejects_missing_repo() {
    let dir = tempfile::tempdir().unwrap();
    assert!(Store::open(dir.path()).is_err());
}

#[test]
fn init_rejects_existing_repo() {
    let dir = tempfile::tempdir().unwrap();
    Store::init(dir.path()).unwrap();
    assert!(Store::init(dir.path()).is_err());
}
