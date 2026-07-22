#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PYTHONPATH="$SCRIPT_DIR/../mavlink_bridge${PYTHONPATH:+:$PYTHONPATH}"
exec "$SCRIPT_DIR/../.venv/bin/python" "$SCRIPT_DIR/run_acceptance.py" "$@"

