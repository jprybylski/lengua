# Records the `list` command from docs/commands.md: adding a couple of
# templates, then listing everything in the store.
#
# Run from the repo root, with a `lengua` build on $PATH, e.g.:
#   cargo build --release -p lengua-cli
#   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/list.tape
#
# Regenerate this whenever `list`'s output format changes.

Output docs/assets/img/list.gif

Require lengua

Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 850
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms

Hide
Type `export STORE="$(sh docs/assets/tapes/list-setup.sh)"; clear`
Enter
Sleep 500ms
Show

Type `echo "Dear {{ name }}," | lengua add letters/thank-you.md --store "$STORE" --title "Thank You"`
Enter
Sleep 1s

Type `echo "Hi {{ name }}!" | lengua add greetings/hello.md --store "$STORE" --title "Hello"`
Enter
Sleep 1s

Type `lengua list --store "$STORE"`
Enter
Sleep 3s
