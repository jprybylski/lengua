# Records `--json` output piped through `jq`, to show the documented JSON
# shapes (docs/commands.md) are meant to be scripted against, not just
# printed for humans.
#
# Run from the repo root, with a `lengua` build on $PATH, e.g.:
#   cargo build --release -p lengua-cli
#   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/json.tape
#
# Regenerate this whenever `list`/`get`'s --json shape changes.

Output docs/assets/img/json.gif

Require lengua
Require jq

Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 850
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms

Hide
Type `export STORE="$(sh docs/assets/tapes/json-setup.sh)"; clear`
Enter
Sleep 500ms
Show

Type `lengua list --store "$STORE" --json | jq .`
Enter
Sleep 2.5s

Type `lengua get letters/thank-you.md --store "$STORE" --json --var name=Ada | jq .`
Enter
Sleep 3s
