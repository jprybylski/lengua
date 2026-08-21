#!/bin/sh
# Creates a scratch store, already initialized, for the history tape.
set -e
dir="$(mktemp -d)"
lengua init --store "$dir" >/dev/null
echo "$dir"
