#!/usr/bin/env bash

set -eu

exe="$1"

cp "$exe" "$HOME/.local/bin/$(basename "$exe")"

if ! $SETUP; then
    "$(dirname "$0")/check.sh"
fi
