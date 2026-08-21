#!/bin/sh
# Creates a scratch store, already initialized, for the tag tape.
set -e
dir="$(mktemp -d)"
lengua init --store "$dir" >/dev/null
echo "$dir"
