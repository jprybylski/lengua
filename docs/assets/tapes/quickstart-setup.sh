#!/bin/sh
# Creates a scratch directory for the quickstart tape and prints its path,
# so the tape can `cd "$(sh quickstart-setup.sh)"` into a clean starting point.
set -e
dir="$(mktemp -d)"
echo "$dir"
