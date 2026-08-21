# Records `get --raw`, showing the unrendered body (with `{{ variables }}`
# left as literal Jinja syntax) alongside a normal rendered `get`, from
# docs/commands.md's `get` section.
#
# Run from the repo root, with a `lengua` build on $PATH, e.g.:
#   cargo build --release -p lengua-cli
#   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/raw.tape
#
# Regenerate this whenever `get`'s output format changes.

Output docs/assets/img/raw.gif

Require lengua

Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 850
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms

Hide
Type `export STORE="$(sh docs/assets/tapes/raw-setup.sh)"; clear`
Enter
Sleep 500ms
Show

Type `lengua get letters/thank-you.md --store "$STORE" --var name=Ada --var reason="the review"`
Enter
Sleep 2s

Type `lengua get letters/thank-you.md --store "$STORE" --raw`
Enter
Sleep 3s
