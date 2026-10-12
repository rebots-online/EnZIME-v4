#!/usr/bin/env python3
"""Read-only validation of an EnZIME build attempt and its adjacent artifacts."""
import argparse
from datetime import datetime
import hashlib
import json
from pathlib import Path
import re
import stat
import sys

SCHEMA = 'mba.robin.enzime.build-set.v1'
QUALIFIERS = {
    'linux-appimage': 'linux-amd64.AppImage',
    'linux-deb': 'linux-amd64.deb',
    'linux-rpm': 'linux-amd64.rpm',
    'windows-nsis': 'windows-x64-setup.exe',
    'windows-msi': 'windows-x64.msi',
    'windows-msix': 'windows-x64.msix',
    'android-apk': 'android-universal-release-signed.apk',
    'android-aab': 'android-universal-release-signed.aab',
    'android-symbols': 'android-native-debug-symbols.zip',
    'web': 'web.tar.gz', 'source': 'source.tar.gz', 'sbom': 'sbom.cdx.json',
}
LANES = {
    'preparation': {'web', 'source'},
    'linux': {'linux-appimage', 'linux-deb', 'linux-rpm'},
    'windows-nsis': {'windows-nsis'},
    'windows-native': {'windows-msi', 'windows-msix'},
    'android': {'android-apk', 'android-aab', 'android-symbols'},
    'sbom': {'sbom'},
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def nonempty(value):
    return isinstance(value, str) and bool(value.strip())


def unique_ids(records, label):
    require(isinstance(records, list), f'{label} must be a list')
    result = {}
    for record in records:
        require(isinstance(record, dict), f'{label} record must be an object')
        key = record.get('id')
        require(nonempty(key), f'{label} record requires id')
        require(key not in result, f'Duplicate {label} id: {key}')
        result[key] = record
    return result


def time_range(record, label):
    values = []
    for field in ('started_at', 'finished_at'):
        value = record.get(field)
        require(nonempty(value), f'{label}.{field} requires an ISO timestamp')
        try:
            parsed = datetime.fromisoformat(value.replace('Z', '+00:00'))
        except ValueError as error:
            raise ValueError(f'{label}.{field}: invalid timestamp') from error
        require(parsed.tzinfo is not None, f'{label}.{field} requires timezone')
        values.append(parsed)
    require(values[0] <= values[1], f'{label}: finished before starting')


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            value.update(chunk)
    return value.hexdigest()


def read_json(path):
    def no_duplicates(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, f'Duplicate JSON key: {key}')
            result[key] = value
        return result
    try:
        return json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=no_duplicates)
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f'Invalid manifest JSON: {error}') from error


def verify(manifest_path, require_complete=False):
    manifest = read_json(manifest_path)
    require(isinstance(manifest, dict), 'Manifest must be an object')
    require(manifest.get('schema') == SCHEMA, 'Unsupported build-set schema')
    version = manifest.get('version')
    require(isinstance(version, str) and re.fullmatch(r'\d+\.\d+\.\d+', version), 'Invalid version')
    major, minor, _ = map(int, version.split('.'))
    require(type(manifest.get('version_code')) is int and manifest['version_code'] == major * 100000 + minor,
            'version_code does not match MAJOR * 100000 + MINOR')
    status = manifest.get('status')
    require(status in ('complete', 'incomplete'), 'Invalid build-set status')
    time_range(manifest, 'build-set')
    required = manifest.get('required_artifact_ids')
    require(isinstance(required, list) and all(isinstance(item, str) for item in required), 'Invalid required_artifact_ids')
    require(len(required) == len(QUALIFIERS) and set(required) == set(QUALIFIERS), 'Required artifact matrix differs from the twelve contracted outputs')
    lanes = unique_ids(manifest.get('lanes'), 'lane')
    require(set(lanes) == set(LANES), 'Required lane records missing or unexpected')
    artifacts = unique_ids(manifest.get('artifacts'), 'artifact')
    require(set(artifacts) <= set(QUALIFIERS), 'Unexpected artifact id')
    referenced = set()
    for lane_id, lane in lanes.items():
        time_range(lane, f'lane {lane_id}')
        lane_status = lane.get('status')
        require(lane_status in ('success', 'failed', 'unavailable'), f'{lane_id}: invalid status')
        require(nonempty(lane.get('log_path')), f'{lane_id}: missing log_path')
        claimed = lane.get('artifacts')
        require(isinstance(claimed, list) and all(isinstance(item, str) for item in claimed), f'{lane_id}: invalid artifact list')
        require(len(claimed) == len(set(claimed)) and set(claimed) <= LANES[lane_id], f'{lane_id}: unexpected or duplicate artifact ownership')
        require(set(claimed) <= set(artifacts), f'{lane_id}: artifact record missing')
        exit_code = lane.get('exit_code')
        if lane_status == 'success':
            require(type(exit_code) is int and exit_code == 0, f'{lane_id}: successful lane requires exit_code 0')
            require(set(claimed) == LANES[lane_id], f'{lane_id}: success requires every owned artifact')
        else:
            require(not claimed, f'{lane_id}: failed/unavailable lane must not link artifacts')
            require(nonempty(lane.get('reason')), f'{lane_id}: failure requires reason')
            require(exit_code is None or (type(exit_code) is int and exit_code != 0), f'{lane_id}: failure cannot have successful exit code')
        referenced.update(claimed)
    require(referenced == set(artifacts), 'Artifact without successful owning lane')
    for artifact_id, artifact in artifacts.items():
        filename = artifact.get('filename')
        expected = f'enzime-reader-v{version}-{QUALIFIERS[artifact_id]}'
        require(filename == expected, f'{artifact_id}: expected exact filename {expected}')
        require(nonempty(artifact.get('platform')) and nonempty(artifact.get('kind')), f'{artifact_id}: platform/kind required')
        signing = artifact.get('signing')
        require(signing in ('verified', 'unsigned', 'not-applicable'), f'{artifact_id}: invalid signing state')
        if artifact_id in ('android-apk', 'android-aab', 'windows-msix'):
            require(signing == 'verified', f'{artifact_id}: required signature verification absent')
        size = artifact.get('size_bytes')
        require(type(size) is int and size > 0, f'{artifact_id}: invalid size')
        sha = artifact.get('sha256')
        require(isinstance(sha, str) and re.fullmatch(r'[0-9a-f]{64}', sha), f'{artifact_id}: invalid SHA256')
        path = manifest_path.parent / filename
        metadata = path.lstat()
        require(stat.S_ISREG(metadata.st_mode) and metadata.st_size == size, f'{artifact_id}: missing, nonregular or wrong-size file')
        require(digest(path) == sha, f'{artifact_id}: SHA256 mismatch')
        locations = artifact.get('locations')
        require(isinstance(locations, list) and bool(locations), f'{artifact_id}: locations required')
        for location in locations:
            require(isinstance(location, dict), f'{artifact_id}: invalid location')
            require(all(nonempty(location.get(key)) for key in ('role', 'transport', 'host', 'path')), f'{artifact_id}: incomplete location')
            require(location.get('public') is False, f'{artifact_id}: unsupported public availability claim')
    source = manifest.get('source')
    require(isinstance(source, dict), 'Source provenance required')
    commit = source.get('commit')
    require(commit is None or (isinstance(commit, str) and re.fullmatch(r'[0-9a-f]{40}|[0-9a-f]{64}', commit)), 'Invalid source commit')
    if 'source' in artifacts:
        require(commit is not None, 'Prepared source requires parent commit')
        require(source.get('archive_filename') == artifacts['source']['filename'], 'Source archive filename mismatch')
        require(source.get('archive_sha256') == artifacts['source']['sha256'], 'Source archive digest mismatch')
    else:
        require(not artifacts, 'Built artifacts require prepared source provenance')
        require(lanes['preparation']['status'] != 'success', 'Successful preparation requires source')
        require(source.get('archive_filename') is None and source.get('archive_sha256') is None, 'Absent source cannot claim an archive')
    complete = len(artifacts) == len(QUALIFIERS) and all(lane['status'] == 'success' for lane in lanes.values())
    require((status == 'complete') == complete, 'Build status does not match complete target matrix')
    require(not require_complete or complete, 'Build set is honestly incomplete; full-complement acceptance failed')
    return version, status, len(artifacts)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifest', type=Path, help='Build manifest; artifacts must reside alongside it')
    parser.add_argument('--require-complete', action='store_true', help='Fail unless all twelve outputs succeeded')
    args = parser.parse_args(argv)
    try:
        version, status, count = verify(args.manifest, args.require_complete)
    except (ValueError, OSError, TypeError, KeyError) as error:
        print(f'Build-set verification failed: {error}', file=sys.stderr)
        return 1
    print(f'Verified EnZIME {version}: {status}; {count}/{len(QUALIFIERS)} current artifacts')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
