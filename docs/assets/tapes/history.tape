# Records committing two versions of the same template, then `log` and
# `diff` showing the resulting git history.
#
# Run from the repo root, with a `lengua` build on $PATH, e.g.:
#   cargo build --release -p lengua-cli
#   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/history.tape
#
# Regenerate this whenever `log`/`diff`'s output format changes.

Output docs/assets/img/history.gif

Require lengua

Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 850
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms

Hide
Type `export STORE="$(sh docs/assets/tapes/history-setup.sh)"; clear`
Enter
Sleep 500ms
Show

Type `echo "Dear {{ name }}, thank you for your time." | lengua add letter.md --store "$STORE" --message "v1"`
Enter
Sleep 1.5s

Type `echo "Dear {{ name }}, thank you for {{ reason }}." | lengua add letter.md --store "$STORE" --message "v2"`
Enter
Sleep 1.5s

Type `lengua log letter.md --store "$STORE"`
Enter
Sleep 2.5s

Type `lengua diff letter.md --store "$STORE"`
Enter
Sleep 3s
