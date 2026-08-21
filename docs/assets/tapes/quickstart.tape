# Records the docs/index.md quick-start flow: init a store, add a template
# with frontmatter (piped via a heredoc), then render it with variables.
#
# Run from the repo root, with a `lengua` build on $PATH, e.g.:
#   cargo build --release -p lengua-cli
#   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/quickstart.tape
#
# Regenerate this whenever `init`/`add`/`get`'s output format changes, so the
# GIF in the docs never drifts from what the tool actually prints.

Output docs/assets/img/quickstart.gif

Require lengua

Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 850
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms

Hide
Type `cd "$(sh docs/assets/tapes/quickstart-setup.sh)"; clear`
Enter
Sleep 500ms
Show

Type `lengua init --store ./templates-repo`
Enter
Sleep 1.5s

Type `lengua add letters/thank-you.md --store ./templates-repo --title "Thank You" --field tone=formal <<'EOF'`
Enter
Type `Dear {{ name }},`
Enter
Enter
Type `Thank you for {{ reason }}.`
Enter
Type `EOF`
Enter
Sleep 2s

Type `lengua get letters/thank-you.md --store ./templates-repo --var name=Ada --var reason="your thoughtful review"`
Enter
Sleep 3s
