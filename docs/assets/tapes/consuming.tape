# Records the docs/consuming.md walkthrough: adopting an existing store
# with `init --from-dir` instead of starting from an empty one, then
# confirming the adopted store's templates and history came along.
#
# Run from the repo root, with a `lengua` build on $PATH, e.g.:
#   cargo build --release -p lengua-cli
#   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/consuming.tape
#
# Regenerate this whenever `init`/`list`/`log`'s output format changes.

Output docs/assets/img/consuming.gif

Require lengua

Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 850
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms

Hide
Type `export SHARED="$(sh docs/assets/tapes/consuming-setup.sh)"; cd "$(mktemp -d)"; clear`
Enter
Sleep 500ms
Show

Type `lengua init --store ./my-copy --from-dir "$SHARED"`
Enter
Sleep 2s

Type `lengua list --store ./my-copy`
Enter
Sleep 2s

Type `lengua log letters/thank-you.md --store ./my-copy`
Enter
Sleep 3s
