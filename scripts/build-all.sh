#!/usr/bin/env bash
# One version, one source snapshot, every platform. No platform subset switches.
set -euo pipefail
if [[ -n "${ENZIME_BUILD_ENV:-}" ]]; then
  [[ -f "$ENZIME_BUILD_ENV" ]] || { echo "Build environment not found: $ENZIME_BUILD_ENV" >&2; exit 1; }
  source "$ENZIME_BUILD_ENV"
elif [[ -f "${XDG_CONFIG_HOME:-$HOME/.config}/enzime/build-env.sh" ]]; then
  source "${XDG_CONFIG_HOME:-$HOME/.config}/enzime/build-env.sh"
fi
cd "$(dirname "$0")/.."
exec python3 scripts/build_set.py "$@"
