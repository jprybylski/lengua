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
