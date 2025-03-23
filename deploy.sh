#!/usr/bin/env bash

set -eu

exe="$1"
bundle="$HOME/.local/bin/pi"

if ! $SETUP; then
    yes_or_no "pi is statically linked into backup, deploy backup too?" && (cd ../backup && cargo deploy)
fi

printf "%s" "Embedding raspbian.iso..."
bundler "$exe" "res/raspbian.img" "$bundle" "PI_END"
echo "done"
