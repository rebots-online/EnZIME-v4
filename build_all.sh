#!/usr/bin/env bash
# One version, one source snapshot, every platform this host builds. No subset switches.
# Windows MSI/MSIX for the same version: build_all_windows.ps1 on a Windows boot.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
exec python3 scripts/build_set.py "$@"
