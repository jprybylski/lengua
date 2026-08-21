# Records the NotAStore error path from docs/faq.md's "What happens if I
# run lengua inside my own project's git repo?" — running a command
# against a --store that's a git repo but not a lengua store.
#
# Run from the repo root, with a `lengua` build on $PATH, e.g.:
#   cargo build --release -p lengua-cli
#   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/guard-error.tape
#
# Regenerate this whenever the NotAStore error message changes.

Output docs/assets/img/guard-error.gif

Require lengua

Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 850
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms

Hide
Type `export PROJECT="$(sh docs/assets/tapes/guard-error-setup.sh)"; clear`
Enter
Sleep 500ms
Show

Type `# a plain git repo — not a lengua store`
Enter
Type `lengua add hello.md --store "$PROJECT" <<< "Hi!"`
Enter
Sleep 3s
