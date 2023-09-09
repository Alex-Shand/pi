#!/usr/bin/env bash

set -eu

exe="$1"
bundle="$HOME/.local/bin/pi"

printf "%s" "Embedding raspbian.iso..."
bundler "$exe" "res/raspbian.img" "$bundle" "PI_END"
echo "done"
