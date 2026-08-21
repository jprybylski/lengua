#!/bin/sh
# Creates a small populated store to stand in for a "shared team library",
# for the consuming tape to adopt via `init --from-dir`.
set -e
dir="$(mktemp -d)"
lengua init --store "$dir" >/dev/null
echo "Dear {{ name }}, thank you for {{ reason }}." | \
  lengua add letters/thank-you.md --store "$dir" --title "Thank You" --field tone=formal >/dev/null
echo "Hi {{ name }}!" | \
  lengua add greetings/hello.md --store "$dir" --title "Hello" >/dev/null
echo "$dir"
