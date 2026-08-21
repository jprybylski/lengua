#!/bin/sh
# Creates a scratch store, already initialized and populated, for the json
# tape (--json piped through jq).
set -e
dir="$(mktemp -d)"
lengua init --store "$dir" >/dev/null
echo "Dear {{ name }}," | \
  lengua add letters/thank-you.md --store "$dir" --title "Thank You" --field tone=formal >/dev/null
echo "$dir"
