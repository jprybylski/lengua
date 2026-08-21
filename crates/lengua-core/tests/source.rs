use lengua_core::{Store, TemplateMeta};

fn meta(title: &str) -> TemplateMeta {
    TemplateMeta {
        title: Some(title.to_string()),
        fields: Default::default(),
    }
}

#[test]
fn init_from_dir_clones_full_history_and_tags() {
    let source_dir = tempfile::tempdir().unwrap();
    let source = Store::init(source_dir.path()).unwrap();
    source
        .add("letter.md", &meta("v1"), "First version.\n", "v1")
        .unwrap();
    source
        .add("letter.md", &meta("v2"), "Second version.\n", "v2")
        .unwrap();
    source
        .tag_create("letter.md", "final", None, false)
        .unwrap();

    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("adopted");
    let adopted = Store::init_from_dir(&dest, source_dir.path(), None, false).unwrap();

    assert_eq!(adopted.log("letter.md").unwrap().len(), 2);
    assert_eq!(adopted.tag_list("letter.md").unwrap().len(), 1);
    let entry = adopted.get("letter.md").unwrap();
    assert!(entry.body.contains("Second version."));
}

#[cfg(unix)]
#[test]
fn init_from_dir_works_when_sigpipe_is_ignored() {
    // Reproduces the exact environment that broke this for lenguar: R sets
    // SIGPIPE to SIG_IGN at startup, which real `git` (spawned by gix's
    // *transport-based* clone) detects and refuses to run under
    // ("ignoring SIGPIPE signal"). `init_from_dir` no longer goes through
    // that transport at all -- it's a plain filesystem copy -- so it must
    // keep working here regardless of SIGPIPE's disposition.
    struct RestoreSigpipe(libc::sighandler_t);
    impl Drop for RestoreSigpipe {
        fn drop(&mut self) {
            unsafe {
                libc::signal(libc::SIGPIPE, self.0);
            }
        }
    }
    let _restore = RestoreSigpipe(unsafe { libc::signal(libc::SIGPIPE, libc::SIG_IGN) });

    let source_dir = tempfile::tempdir().unwrap();
    let source = Store::init(source_dir.path()).unwrap();
    source
        .add("letter.md", &meta("v1"), "First version.\n", "v1")
        .unwrap();

    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("adopted");
    let adopted = Store::init_from_dir(&dest, source_dir.path(), None, false).unwrap();
    assert!(
        adopted
            .get("letter.md")
            .unwrap()
            .body
            .contains("First version.")
    );
}

#[test]
fn init_from_dir_refuses_nonempty_existing_destination() {
    let source_dir = tempfile::tempdir().unwrap();
    Store::init(source_dir.path()).unwrap();

    let dest_dir = tempfile::tempdir().unwrap();
    Store::init(dest_dir.path()).unwrap();

    assert!(Store::init_from_dir(dest_dir.path(), source_dir.path(), None, false).is_err());
}

#[test]
fn init_from_dir_with_subdir_flattens_to_current_content_only() {
    // `subdir = Some(".")` exercises the flattening code path (staging ->
    // open subdir -> replay as fresh commits) against the store's own root,
    // which is enough to prove it drops history/tags without needing a
    // multi-store fixture repo.
    let source_dir = tempfile::tempdir().unwrap();
    let source = Store::init(source_dir.path()).unwrap();
    source
        .add("letter.md", &meta("v1"), "First version.\n", "v1")
        .unwrap();
    source
        .add("letter.md", &meta("v2"), "Second version.\n", "v2")
        .unwrap();
    source
        .tag_create("letter.md", "final", None, false)
        .unwrap();

    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("adopted");
    let adopted = Store::init_from_dir(&dest, source_dir.path(), Some("."), false).unwrap();

    let entry = adopted.get("letter.md").unwrap();
    assert!(entry.body.contains("Second version."));
    // Only the current content was imported as one fresh commit, not the
    // source's two-commit history, and no tags carried over.
    assert_eq!(adopted.log("letter.md").unwrap().len(), 1);
    assert!(adopted.tag_list("letter.md").unwrap().is_empty());
}
