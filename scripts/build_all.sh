#!/usr/bin/env bash
# One version, one source snapshot, every platform. No platform subset switches.
set -euo pipefail
cd "$(dirname "$0")/.."
exec python3 scripts/build_set.py "$@"
