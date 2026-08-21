# Records the `search` command from docs/commands.md: adding templates
# with different frontmatter, then filtering by field.
#
# Run from the repo root, with a `lengua` build on $PATH, e.g.:
#   cargo build --release -p lengua-cli
#   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/search.tape
#
# Regenerate this whenever `search`'s output format changes.

Output docs/assets/img/search.gif

Require lengua

Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 850
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms

Hide
Type `export STORE="$(sh docs/assets/tapes/search-setup.sh)"; clear`
Enter
Sleep 500ms
Show

Type `echo "Dear Sir," | lengua add formal.md --store "$STORE" --field tone=formal`
Enter
Sleep 1s

Type `echo "Hey!" | lengua add casual.md --store "$STORE" --field tone=casual`
Enter
Sleep 1s

Type `lengua search --store "$STORE" --field tone=formal`
Enter
Sleep 3s
