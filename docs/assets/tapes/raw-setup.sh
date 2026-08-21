#!/bin/sh
# Creates a scratch store with one template, for the raw tape.
set -e
dir="$(mktemp -d)"
lengua init --store "$dir" >/dev/null
echo "Dear {{ name }}, thank you for {{ reason }}." | \
  lengua add letters/thank-you.md --store "$dir" --title "Thank You" >/dev/null
echo "$dir"
