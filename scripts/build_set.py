#!/usr/bin/env python3
"""Build one immutable EnZIME source snapshot across the complete target matrix."""
import argparse
import concurrent.futures
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
import fcntl
import hashlib
import html
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import sys
import tarfile
import threading
import tomllib

SCHEMA = "mba.robin.enzime.build-set.v1"
TARGETS = {
    "preparation": ("source", "web"),
    "linux": ("linux-appimage", "linux-deb", "linux-rpm"),
    "windows-nsis": ("windows-nsis",),
    "android": ("android-apk", "android-aab", "android-symbols"),
    "windows-native": ("windows-msi", "windows-msix"),
    "sbom": ("sbom",),
}
STAMPED = ("version.txt", "version.json", "package.json", "Cargo.toml",
           "apps/reader/src-tauri/Cargo.toml", "apps/reader/src-tauri/tauri.conf.json",
           "apps/reader/src-tauri/gen/android/tauri.properties", "Cargo.lock")
PRINT_LOCK = threading.Lock()
PROCESSES = set()
PROCESS_LOCK = threading.Lock()
STOP = threading.Event()


def now():
    return datetime.now(timezone.utc).isoformat()


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def atomic_json(path, value):
    temporary = path.with_name(path.name + ".writing")
    temporary.write_text(json.dumps(value, indent=2) + "\n")
    temporary.replace(path)


def report(lane, line):
    with PRINT_LOCK:
        print(f"[{lane}] {line.rstrip()}", flush=True)


def run(command, cwd, log, lane, env=None, echo=True):
    if STOP.is_set():
        raise RuntimeError("build interrupted")
    with log.open("a") as output:
        # Command strings can contain credential-bearing arguments; do not echo them.
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT, text=True,
                                   errors="replace", start_new_session=True)
        with PROCESS_LOCK:
            PROCESSES.add(process)
        try:
            for line in process.stdout:
                output.write(line)
                output.flush()
                if echo:
                    report(lane, line)
            code = process.wait()
        finally:
            with PROCESS_LOCK:
                PROCESSES.discard(process)
        if code:
            raise subprocess.CalledProcessError(code, command)


def stop(signum, _frame):
    STOP.set()
    with PROCESS_LOCK:
        for process in tuple(PROCESSES):
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass


@dataclass
class BuildArtifact:
    id: str
    platform: str
    kind: str
    filename: str
    size_bytes: int
    sha256: str
    signing: str
    locations: list


@dataclass
class LaneResult:
    id: str
    status: str = "failed"
    exit_code: int = 1
    reason: str = "not attempted"
    log_path: str = ""
    started_at: str = field(default_factory=now)
    finished_at: str = field(default_factory=now)
    artifacts: list = field(default_factory=list)


@dataclass
class BuildSet:
    root: Path
    snapshot: Path
    workspace: Path
    logs: Path
    artifact_dir: Path
    source_commit: str
    version: str = ""
    version_code: int = 0
    source_archive_sha256: str | None = None
    source_archive_filename: str | None = None
    started_at: str = field(default_factory=now)
    artifacts: list = field(default_factory=list)


def excluded(name):
    parts = Path(name).parts
    return (not parts or parts[0] in {"dist", "target", "signing", "CREDENTIALS"}
            or name.startswith("apps/reader/dist/") or name == "apps/reader/dist"
            or any(part in {"signing", "CREDENTIALS", "target"} for part in parts)
            or any(part in {"node_modules", ".git"} for part in parts)
            or any(part.startswith(".env") and part != ".env.example" for part in parts)
            or name == "release.lock")


def snapshot_commit(build):
    archive = build.workspace / "committed-source.tar"
    with archive.open("wb") as stream:
        subprocess.run(["git", "archive", build.source_commit], cwd=build.root,
                       stdout=stream, check=True)
    build.snapshot.mkdir()
    with tarfile.open(archive) as source:
        members = [member for member in source if not excluded(member.name)]
        source.extractall(build.snapshot, members=members, filter="data")


def collect(build, path, metadata):
    if not path.is_file() or path.is_symlink() or not path.stat().st_size:
        raise ValueError(f"Missing regular artifact: {path.name}")
    filename = metadata["filename"]
    if Path(filename).name != filename or not filename.startswith(f"enzime-reader-v{build.version}-"):
        raise ValueError("Artifact filename does not identify this build")
    target = build.artifact_dir / filename
    checksum = digest(path)
    if target.exists() and digest(target) != checksum:
        raise ValueError(f"Historical artifact collision: {filename}")
    if not target.exists():
        pending = target.with_name(target.name + ".writing")
        shutil.copy2(path, pending)
        pending.replace(target)
    artifact = BuildArtifact(metadata["id"], metadata["platform"], metadata["kind"],
                             filename, target.stat().st_size, checksum, metadata["signing"],
                             [{"role": "canonical-workspace", "transport": "local-fs",
                               "host": socket.gethostname(), "path": str(target), "public": False}])
    return artifact


def prepare_snapshot(repo, source_ref):
    commit = subprocess.check_output(["git", "rev-parse", "--verify", source_ref + "^{commit}"],
                                     cwd=repo, text=True).strip()
    attempt = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ") + "-" + commit[:12]
    workspace = Path(os.environ.get("ENZIME_BUILD_WORKSPACE_ROOT", str(repo / "target/build-set"))).expanduser().resolve() / attempt
    workspace.mkdir(parents=True)
    logs = Path(os.environ.get("ENZIME_BUILD_LOG_ROOT", str(Path.home() / "Admin-Manual/LOGS/EnZIME-v4/build-sets"))).expanduser().resolve() / attempt
    logs.mkdir(parents=True)
    artifact_dir = Path(os.environ.get("ENZIME_BUILD_ARTIFACT_DIR", str(repo / "dist"))).expanduser().resolve()
    build = BuildSet(repo, workspace / "source", workspace, logs, artifact_dir, commit)
    build.artifact_dir.mkdir(parents=True, exist_ok=True)
    result = LaneResult("preparation", log_path=str(logs / "preparation.log"))
    try:
        dirty = subprocess.check_output(["git", "status", "--porcelain", "-uno"], cwd=repo, text=True)
        report("preparation", f"Source commit {commit}; {len(dirty.splitlines())} tracked dirty paths excluded")
        originals = {name: (repo / name).read_bytes() if (repo / name).exists() else None for name in STAMPED}
        lock_before = (repo / "release.lock").read_bytes() if (repo / "release.lock").exists() else None
        if lock_before is not None:
            (logs / "previous-release.lock").write_bytes(lock_before)
        snapshot_commit(build)
        (build.snapshot / "version.txt").write_bytes(originals["version.txt"])
        run(["bash", "scripts/update-version.sh"], build.snapshot, Path(result.log_path), "preparation")
        version = json.loads((build.snapshot / "version.json").read_text())
        build.version, build.version_code = version["version"], version["versionCode"]
        major, minor, number = [int(value) for value in build.version.split(".")]
        freeze = f"MAJOR={major}\nMINOR={minor}\nBUILD_NUM={number}\n"
        (build.snapshot / "release.lock").write_text(freeze)
        # Compare every destination before modifying any of them.
        for name in STAMPED:
            observed = (repo / name).read_bytes() if (repo / name).exists() else None
            if observed != originals[name]:
                raise RuntimeError(f"Concurrent version change: {name}")
        current_lock = (repo / "release.lock").read_bytes() if (repo / "release.lock").exists() else None
        if current_lock != lock_before:
            raise RuntimeError("Concurrent release.lock change")
        for name in STAMPED:
            if name != "Cargo.lock" and (build.snapshot / name).exists():
                shutil.copy2(build.snapshot / name, repo / name)
        (repo / "release.lock").write_text(freeze)
        run(["pnpm", "install", "--frozen-lockfile"], build.snapshot, Path(result.log_path), "preparation")
        run(["pnpm", "exec", "tsc", "--noEmit"], build.snapshot / "apps/reader", Path(result.log_path), "preparation")
        run(["pnpm", "exec", "vite", "build"], build.snapshot / "apps/reader", Path(result.log_path), "preparation")
        # Cargo resolves workspace package version changes once; lanes consume this lock.
        run(["cargo", "metadata", "--offline", "--format-version", "1"], build.snapshot,
            logs / "cargo-metadata.json", "preparation", echo=False)
        packages = tomllib.loads((build.snapshot / "Cargo.lock").read_text())["package"]
        if not any(p["name"] == "reader" and p["version"] == build.version for p in packages):
            raise RuntimeError("Cargo lock does not contain stamped reader version")
        original_packages = tomllib.loads(originals["Cargo.lock"].decode())["package"]
        pinned = lambda items: {(p["name"], p["version"], p.get("source"), p.get("checksum")) for p in items if p.get("source")}
        if pinned(packages) != pinned(original_packages):
            raise RuntimeError("Preparation changed locked third-party dependencies")
        if (repo / "Cargo.lock").read_bytes() != originals["Cargo.lock"]:
            raise RuntimeError("Concurrent Cargo.lock change")
        shutil.copy2(build.snapshot / "Cargo.lock", repo / "Cargo.lock")
        source = workspace / f"enzime-reader-v{build.version}-source.tar.gz"
        names = subprocess.check_output(["git", "ls-tree", "-rz", "--name-only", commit], cwd=repo).split(b"\0")
        names = {os.fsdecode(name) for name in names if name and not excluded(os.fsdecode(name))}
        names.update(STAMPED)
        names.add("release.lock")
        with tarfile.open(source, "w:gz") as archive:
            for name in sorted(names):
                path = build.snapshot / name
                if path.exists() or path.is_symlink():
                    archive.add(path, arcname=name, recursive=False)
        build.source_archive_sha256, build.source_archive_filename = digest(source), source.name
        web = workspace / f"enzime-reader-v{build.version}-web.tar.gz"
        frontend = build.snapshot / "apps/reader/dist"
        if not (frontend / "index.html").is_file() or not list((frontend / "assets").glob("*.js")):
            raise RuntimeError("Prepared frontend lacks HTML/JS assets")
        with tarfile.open(web, "w:gz") as archive:
            archive.add(frontend, arcname="web")
        collected = [collect(build, path, {"id": identity, "platform": platform,
                "kind": "tar.gz", "filename": path.name, "signing": "not-applicable"})
                for path, identity, platform in [(source, "source", "source"), (web, "web", "web")]]
        build.artifacts.extend(collected)
        result.artifacts = [item.id for item in collected]
        result.status, result.exit_code, result.reason = "success", 0, ""
    except Exception as error:
        result.reason = str(error)
        build.source_archive_sha256 = build.source_archive_filename = None
        report("preparation", result.reason)
        if not build.version:
            # A failure before stamping must not create a false new version.
            data = json.loads((repo / "version.json").read_text())
            build.version, build.version_code = data["version"], data["versionCode"]
    result.finished_at = now()
    return build, result


def run_lane(build, lane):
    result = LaneResult(lane, log_path=str(build.logs / (lane + ".log")))
    lane_root = build.workspace / "lanes" / lane
    output = build.workspace / "output" / lane
    target = Path(os.environ.get("ENZIME_BUILD_CACHE_ROOT", str(build.root / "target/build-set-cache"))).expanduser().resolve() / lane
    try:
        output.mkdir(parents=True)
        target.mkdir(parents=True, exist_ok=True)
        lane_root.parent.mkdir(parents=True, exist_ok=True)
        run(["cp", "-a", "--reflink=auto", str(build.snapshot), str(lane_root)],
            build.root, Path(result.log_path), lane)
        env = os.environ.copy()
        credentials = Path(env.get("ENZIME_SIGNING_CONFIG", str(build.root / "signing/build-credentials.json")))
        if credentials.is_file():
            values = json.loads(credentials.read_text())
            for key in ("ENZIME_KEYSTORE_PASSWORD", "ENZIME_KEY_PASSWORD", "ENZIME_KEYSTORE_PATH", "ENZIME_KEYSTORE_ALIAS"):
                if key in values:
                    if not isinstance(values[key], str):
                        raise ValueError("Signing configuration values must be strings")
                    env.setdefault(key, values[key])
        env["ENZIME_PREPARED_SOURCE_ARCHIVE"] = str(build.workspace / build.source_archive_filename)
        env.setdefault("ENZIME_KEYSTORE_PATH", str(build.root / "signing/production.keystore"))
        env.setdefault("CARGO_BUILD_JOBS", "2")
        run(["bash", str(lane_root / "scripts/build-lane.sh"), lane, str(lane_root),
             str(target), str(output)], lane_root, Path(result.log_path), lane, env)
        metadata = json.loads((output / "artifacts.json").read_text())
        if not isinstance(metadata, list) or {item["id"] for item in metadata} != set(TARGETS[lane]) or len(metadata) != len(TARGETS[lane]):
            raise ValueError("Lane output does not match its required artifact set")
        # Validate the whole lane before publishing any of its artifacts.
        for item in metadata:
            path = output / item["filename"]
            if Path(item["filename"]).name != item["filename"] or not path.is_file() or not path.stat().st_size:
                raise ValueError("Invalid or empty lane artifact")
        collected = [collect(build, output / item["filename"], item) for item in metadata]
        build.artifacts.extend(collected)
        result.artifacts = [item.id for item in collected]
        result.status, result.exit_code, result.reason = "success", 0, ""
    except subprocess.CalledProcessError as error:
        result.exit_code = error.returncode
        result.status = "unavailable" if error.returncode == 69 else "failed"
        result.reason = f"Lane exited {error.returncode}; see {result.log_path}"
    except Exception as error:
        result.reason = str(error)
    result.finished_at = now()
    report(lane, f"{result.status}: {result.reason or 'all required artifacts collected'}")
    return result


def run_lanes(build, jobs):
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = {pool.submit(run_lane, build, lane): lane for lane in TARGETS if lane != "preparation"}
        results = []
        for future in concurrent.futures.as_completed(futures):
            try:
                results.append(future.result())
            except Exception as error:
                lane = futures[future]
                results.append(LaneResult(lane, reason=str(error), log_path=str(build.logs / (lane + ".log"))))
        return results


def finalize(build, results):
    complete = {artifact.id for artifact in build.artifacts} == {item for values in TARGETS.values() for item in values} and all(row.status == "success" for row in results)
    manifest = {"schema": SCHEMA, "version": build.version, "version_code": build.version_code,
        "status": "complete" if complete else "incomplete", "source": {"commit": build.source_commit,
        "archive_sha256": build.source_archive_sha256, "archive_filename": build.source_archive_filename},
        "started_at": build.started_at, "finished_at": now(), "lanes": [asdict(row) for row in results],
        "artifacts": [asdict(row) for row in build.artifacts],
        "required_artifact_ids": [item for values in TARGETS.values() for item in values]}
    path = build.artifact_dir / f"enzime-reader-v{build.version}-build-manifest.json"
    # A preparation failure before stamping must not overwrite an earlier attempt.
    if path.exists():
        path = build.artifact_dir / f"enzime-reader-v{build.version}-attempt-{build.workspace.name}-build-manifest.json"
    atomic_json(path, manifest)
    verified = subprocess.run([sys.executable, str(build.root / "scripts/verify-build-set.py"), str(path)]).returncode == 0
    if not verified:
        complete = False
        manifest["status"] = "incomplete"
        manifest["verification_error"] = "Build manifest failed independent verification; inspect lane logs"
        atomic_json(path, manifest)
    atomic_json(build.artifact_dir / "current-build-manifest.json", manifest)
    checksum = build.artifact_dir / f"enzime-reader-v{build.version}-sha256sums.txt"
    if not checksum.exists():
        checksum.write_text("".join(f"{a.sha256}  {a.filename}\n" for a in build.artifacts))
    artifacts = {item.id: item for item in build.artifacts}
    outcomes = {row.id: row for row in results}
    rows = []
    for lane, ids in TARGETS.items():
        for identity in ids:
            artifact = artifacts.get(identity)
            detail = (f'<a href="{html.escape(artifact.filename, quote=True)}">{html.escape(artifact.filename)}</a><br><code>{artifact.sha256}</code>'
                      if artifact else html.escape(outcomes[lane].reason or outcomes[lane].status))
            rows.append(f"<tr><th>{identity}</th><td>{outcomes[lane].status}</td><td>{detail}</td></tr>")
    page = f'''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>EnZIME current build</title>
<style>body{{font:16px system-ui;background:#111827;color:#f1f5f9;margin:3rem auto;max-width:1100px;padding:1rem}}a{{color:#93c5fd}}table{{width:100%;border-collapse:collapse}}th,td{{text-align:left;padding:1rem;border-bottom:1px solid #475569;overflow-wrap:anywhere}}code{{font-size:.75rem}}th{{white-space:nowrap}}</style>
<h1>EnZIME Reader v{html.escape(build.version)}</h1><p>Latest build attempt: <strong>{manifest['status']}</strong>. This is a development build, not a commercial release.</p>
<p>Every download below belongs to this version. Failed targets do not link older builds.</p><p><a href="current-build-manifest.json">Build manifest</a></p>
<table><thead><tr><th>Target</th><th>Result</th><th>Artifact or failure</th></tr></thead><tbody>{''.join(rows)}</tbody></table></html>'''
    pending = build.artifact_dir / "index.html.writing"
    pending.write_text(page)
    pending.replace(build.artifact_dir / "index.html")
    if complete:
        atomic_json(build.artifact_dir / "current-complete-build-manifest.json", manifest)
    report("build-set", f"v{build.version}: {manifest['status']}; {path}")
    return 0 if complete else 1


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jobs", type=int, default=3)
    parser.add_argument("--source-ref", default="HEAD")
    args = parser.parse_args(argv)
    if not 1 <= args.jobs <= 8:
        parser.error("--jobs must be between 1 and 8")
    repo = Path(__file__).resolve().parent.parent
    lock = repo / "target/build-set/build.lock"
    lock.parent.mkdir(parents=True, exist_ok=True)
    with lock.open("a") as guard:
        try:
            fcntl.flock(guard, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            parser.error("Another unified build holds this repository's build lock")
        signal.signal(signal.SIGINT, stop)
        signal.signal(signal.SIGTERM, stop)
        try:
            build, preparation = prepare_snapshot(repo, args.source_ref)
        except Exception as error:
            report("preparation", f"Could not start build attempt: {error}")
            return 1
        if preparation.status == "success":
            results = [preparation, *run_lanes(build, args.jobs)]
        else:
            results = [preparation, *[LaneResult(lane, reason="Source preparation failed", log_path=preparation.log_path)
                                      for lane in TARGETS if lane != "preparation"]]
        return finalize(build, results)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
