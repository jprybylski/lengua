use assert_cmd::Command;
use predicates::prelude::*;

fn lengua() -> Command {
    Command::cargo_bin("lengua").unwrap()
}

fn init_store(dir: &std::path::Path) {
    lengua()
        .arg("--store")
        .arg(dir)
        .arg("init")
        .assert()
        .success();
}

#[test]
fn init_creates_a_store() {
    let dir = tempfile::tempdir().unwrap();
    lengua()
        .arg("--store")
        .arg(dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));
    assert!(dir.path().join(".git").is_dir());
    assert!(dir.path().join("templates").is_dir());
}

#[test]
fn add_then_get_renders_variables() {
    let dir = tempfile::tempdir().unwrap();
    init_store(dir.path());

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args([
            "add",
            "hello.md",
            "--title",
            "Hello",
            "--field",
            "tone=casual",
        ])
        .write_stdin("Hi {{ name }}!\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added hello.md"));

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["get", "hello.md", "--var", "name=Ada"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi Ada!"));
}

#[test]
fn list_and_search_reflect_added_templates() {
    let dir = tempfile::tempdir().unwrap();
    init_store(dir.path());

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["add", "formal.md", "--field", "tone=formal"])
        .write_stdin("Dear Sir,\n")
        .assert()
        .success();
    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["add", "casual.md", "--field", "tone=casual"])
        .write_stdin("Hey!\n")
        .assert()
        .success();

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            let v: serde_json::Value = serde_json::from_str(s).unwrap();
            v.as_array().unwrap().len() == 2
        }));

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["search", "--field", "tone=formal"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("formal.md").and(predicate::str::contains("casual.md").not()),
        );
}

#[test]
fn log_and_diff_track_history() {
    let dir = tempfile::tempdir().unwrap();
    init_store(dir.path());

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["add", "letter.md", "--message", "v1"])
        .write_stdin("First version.\n")
        .assert()
        .success();
    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["add", "letter.md", "--message", "v2"])
        .write_stdin("Second version.\n")
        .assert()
        .success();

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["log", "letter.md", "--json"])
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            let v: serde_json::Value = serde_json::from_str(s).unwrap();
            v.as_array().unwrap().len() == 2
        }));

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["diff", "letter.md", "HEAD~1", "HEAD"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("- First version.")
                .and(predicate::str::contains("+ Second version.")),
        );
}

#[test]
fn adding_a_file_that_already_has_frontmatter_does_not_nest_it() {
    let dir = tempfile::tempdir().unwrap();
    init_store(dir.path());

    // A hand-authored template file that already carries its own
    // frontmatter, as fixtures/templates/**/*.md do.
    let source = dir.path().join("source.md");
    std::fs::write(
        &source,
        "---\ntitle: Hello Greeting\ntone: casual\n---\n\nHi {{ name }}, welcome!\n",
    )
    .unwrap();

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["add", "hello.md", "--file"])
        .arg(&source)
        .assert()
        .success();

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["get", "hello.md", "--raw", "--json"])
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            let v: serde_json::Value = serde_json::from_str(s).unwrap();
            let rendered = v["rendered"].as_str().unwrap();
            // The body should contain no leftover `---` frontmatter fence.
            !rendered.contains("---")
        }));

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""title": "Hello Greeting"#));
}

#[test]
fn get_missing_template_fails_with_nonzero_exit() {
    let dir = tempfile::tempdir().unwrap();
    init_store(dir.path());

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["get", "does-not-exist.md"])
        .assert()
        .failure();
}

#[test]
fn tag_add_list_rm_roundtrip_and_retroactive_tagging() {
    let dir = tempfile::tempdir().unwrap();
    init_store(dir.path());

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["add", "letter.md", "--message", "past tense"])
        .write_stdin("We had so much fun.\n")
        .assert()
        .success();
    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["add", "letter.md", "--message", "future tense"])
        .write_stdin("We will have so much fun.\n")
        .assert()
        .success();

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["tag", "add", "letter.md", "tense-future"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tense-future"));
    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["tag", "add", "letter.md", "tense-past", "--rev", "HEAD~1"])
        .assert()
        .success();

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["tag", "list", "letter.md"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("tense-future").and(predicate::str::contains("tense-past")),
        );

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["get", "letter.md", "--rev", "tense-past"])
        .assert()
        .success()
        .stdout(predicate::str::contains("We had so much fun."));
    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["get", "letter.md", "--rev", "tense-future"])
        .assert()
        .success()
        .stdout(predicate::str::contains("We will have so much fun."));

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["tag", "rm", "letter.md", "tense-past"])
        .assert()
        .success();
    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["tag", "list", "letter.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tense-past").not());
}

#[test]
fn init_from_dir_adopts_an_existing_store() {
    let source = tempfile::tempdir().unwrap();
    init_store(source.path());
    lengua()
        .arg("--store")
        .arg(source.path())
        .args(["add", "hello.md"])
        .write_stdin("Hi {{ name }}!\n")
        .assert()
        .success();

    let parent = tempfile::tempdir().unwrap();
    let dest = parent.path().join("adopted");
    lengua()
        .arg("--store")
        .arg(&dest)
        .args(["init", "--from-dir"])
        .arg(source.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Adopted"));

    lengua()
        .arg("--store")
        .arg(&dest)
        .args(["get", "hello.md", "--var", "name=Ada"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi Ada!"));
}

#[test]
fn running_outside_a_store_fails_with_a_clear_error() {
    let dir = tempfile::tempdir().unwrap();
    gix::init(dir.path()).unwrap();

    lengua()
        .arg("--store")
        .arg(dir.path())
        .args(["add", "hello.md"])
        .write_stdin("Hi!\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("doesn't look like a lengua store"));
}
