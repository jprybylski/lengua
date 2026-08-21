---
name: regenerate-tapes
description: Rebuild lengua and re-run VHS over docs/assets/tapes/*.tape to refresh the demo GIFs in docs/assets/img/ after a CLI output change.
---

# Regenerate demo tape GIFs

Every file in `docs/assets/tapes/*.tape` records a live `lengua` terminal session and renders
it to the matching GIF in `docs/assets/img/`. Each tape's header comment says "regenerate this
whenever the CLI's output format changes" — this skill does that regeneration.

Run this whenever a change touches anything a tape's terminal output would show:
`crates/lengua-cli/src/**` (subcommand output/formatting) or `crates/lengua-core/src/**`
(anything that changes rendered/diff/log content).

## Steps

1. **Confirm `vhs` is installed** (`vhs --version`). If missing, stop and tell the user to
   install it (e.g. `brew install vhs`) — don't try to install it yourself.

2. **Build a release binary** so the tapes see realistic output/timing:
   ```bash
   cargo build --release -p lengua-cli
   ```

3. **Run every tape from the repo root**, with `target/release` first on `PATH` (the tapes'
   setup scripts reference repo-relative paths like `docs/assets/tapes/*-setup.sh`, so the
   working directory matters):
   ```bash
   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/quickstart.tape
   PATH="$PWD/target/release:$PATH" vhs docs/assets/tapes/history.tape
   ```

4. **Report what changed**, not just "done":
   ```bash
   git status --short docs/assets/img/
   ```
   GIFs are binary, so there's no useful `git diff` — list which of the 2 changed and which
   didn't. An unchanged GIF for a tape whose underlying output *did* change is worth flagging
   back to the user, since it usually means that tape doesn't exercise the changed code path.

5. Leave the refreshed GIFs staged/modified in the working tree for the user (or the calling
   agent) to review and commit — don't commit automatically.
