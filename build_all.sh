#!/usr/bin/env bash
# One version, one source snapshot, every platform this host builds. No subset switches.
# Windows MSI/MSIX for the same version: build_all_windows.ps1 on a Windows boot.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
python3 scripts/build_set.py "$@"

# BUMP-1: after a successful full build, advance the source version (MINOR+1, BUILD from
# epoch minutes), update version.txt/version.json, commit only those two files, push, and
# print the completed artifact version and the next source version distinctly.
completed="$(cat version.txt)"
next_version="$(python3 - "$completed" <<'PY'
import json
import sys
import time
from datetime import datetime

completed = sys.argv[1].strip()
major_s, minor_s, _build = completed.split(".")
major = int(major_s)
minor = int(minor_s) + 1
build = int(time.time() // 60) % 100000
version = "%d.%d.%05d" % (major, minor, build)

with open("version.json", "r", encoding="utf-8") as f:
    data = json.load(f)
data["version"] = version
data["semver"] = "%d.%d.%d" % (major, minor, build)
data["versionBase"] = "%d.%d" % (major, minor)
data["buildNumber"] = "%05d" % build
data["versionCode"] = major * 100000 + minor
data["buildDate"] = datetime.now().astimezone().isoformat(timespec="seconds")
data["builtAt"] = None
with open("version.txt", "w", encoding="utf-8") as f:
    f.write(version + "\n")
with open("version.json", "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
print(version)
PY
)"
git commit --only version.txt version.json -m "Advance source version to ${next_version} after successful build [skip ci]"
git push
printf 'Completed artifact version: %s\n' "$completed"
printf 'Next source version: %s\n' "$next_version"
