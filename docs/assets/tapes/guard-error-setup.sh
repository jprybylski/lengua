#!/bin/sh
# Creates a plain git repo (no templates/ directory) for the guard-error
# tape, to demonstrate lengua's error when --store isn't a lengua store.
set -e
dir="$(mktemp -d)"
git init --quiet "$dir"
echo "$dir"
