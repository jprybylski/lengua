# Records the tag workflow from docs/consuming.md and docs/commands.md's
# `tag` section: committing two revisions of a template, tagging the
# current one, retroactively tagging the prior one via --rev, listing
# tags, and diffing between two tag names.
#
# Run from the repo root, with a `lengua` build on $PATH, e.g.:
#   cargo build --release -p lengua-cli
#   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/tag.tape
#
# Regenerate this whenever `tag`/`diff`'s output format changes.

Output docs/assets/img/tag.gif

Require lengua

Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 850
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms

Hide
Type `export STORE="$(sh docs/assets/tapes/tag-setup.sh)"; clear`
Enter
Sleep 500ms
Show

Type `echo "We had so much fun." | lengua add letter.md --store "$STORE" --message "past tense"`
Enter
Sleep 1.5s

Type `echo "We will have so much fun." | lengua add letter.md --store "$STORE" --message "future tense"`
Enter
Sleep 1.5s

Type `lengua tag add letter.md tense-future --store "$STORE"`
Enter
Sleep 1.5s

Type `lengua tag add letter.md tense-past --store "$STORE" --rev HEAD~1`
Enter
Sleep 1.5s

Type `lengua tag list letter.md --store "$STORE"`
Enter
Sleep 2s

Type `lengua diff letter.md tense-past tense-future --store "$STORE"`
Enter
Sleep 3s
