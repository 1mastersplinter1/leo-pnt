#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$SCRIPT_DIR/ardupilot"
PYTHON="$SCRIPT_DIR/../.venv/bin/python"
PIN="3fc7011a7d3dc047cbb17d8bd98ee94577d144c6"

if [[ ! -d "$SOURCE_DIR/.git" ]]; then
    git clone --filter=blob:none --no-checkout https://github.com/ArduPilot/ardupilot.git "$SOURCE_DIR"
fi
git -C "$SOURCE_DIR" fetch --depth=1 origin "$PIN"
git -C "$SOURCE_DIR" checkout --detach "$PIN"
git -C "$SOURCE_DIR" submodule update --init --recursive --depth=1
(
    cd "$SOURCE_DIR"
    "$PYTHON" ./waf configure --board sitl
    "$PYTHON" ./waf rover
)
ARTIFACT="$SOURCE_DIR/build/sitl/bin/ardurover"
test -x "$ARTIFACT"
sha256sum "$ARTIFACT" | tee "$SCRIPT_DIR/ardurover.sha256"
