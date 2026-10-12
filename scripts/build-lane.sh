#!/usr/bin/env bash
# Internal build-set worker. The public command is scripts/build-all.sh.
set -euo pipefail
exec python3 - "$@" <<'PY'
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import zipfile


def fail(message, code=1):
    print(message, file=sys.stderr)
    raise SystemExit(code)


def read_json(path):
    try:
        return json.loads(Path(path).read_text())
    except (OSError, ValueError) as exc:
        fail(f'Invalid JSON at {path}: {exc}')


if len(sys.argv) != 5:
    fail('Usage: build-lane.sh LANE SNAPSHOT TARGET_DIR OUTPUT_DIR', 64)
lane, snapshot_arg, target_arg, output_arg = sys.argv[1:]
if lane not in ('linux', 'windows-nsis', 'android', 'sbom', 'windows-native'):
    fail('Unknown build lane', 64)
snapshot = Path(snapshot_arg).resolve(strict=True)
target = Path(target_arg).resolve()
output = Path(output_arg).resolve()
if output == snapshot or output == target or snapshot in output.parents:
    fail('Output must be an independent attempt-specific directory')
output.mkdir(parents=True, exist_ok=True)
if any(output.iterdir()):
    fail('Output directory must be empty')
target.mkdir(parents=True, exist_ok=True)
version_data = read_json(snapshot / 'version.json')
version = version_data.get('version', '')
if not re.fullmatch(r'\d+\.\d+\.\d+', version):
    fail('Invalid prepared version')
config_data = read_json(snapshot / 'apps/reader/src-tauri/tauri.conf.json')
if config_data.get('version') != version:
    fail('Prepared Tauri version mismatch')
app = snapshot / 'apps/reader'
frontend = app / 'dist'
if not (frontend / 'index.html').is_file():
    fail('Prepared frontend is absent')
config = target / 'prepared-tauri.json'
config.write_text(json.dumps({'build': {'beforeBuildCommand': 'echo Using prepared frontend',
                                       'frontendDist': str(frontend)}}))
env = os.environ.copy()
env['CARGO_TARGET_DIR'] = str(target)
env.setdefault('CARGO_BUILD_JOBS', '2')
if lane == 'android' and not env.get('NDK_HOME'):
    sdk = Path(env.get('ANDROID_HOME', env.get('ANDROID_SDK_ROOT', str(Path.home() / 'Android/Sdk'))))
    known_ndk = sdk / 'ndk/27.0.12077973'
    if known_ndk.is_dir():
        env['NDK_HOME'] = str(known_ndk)
artifacts = []
prefix = f'enzime-reader-v{version}-'


def run(args, cwd=app, capture=False):
    # Never print the environment or command arguments containing credentials.
    result = subprocess.run([str(arg) for arg in args], cwd=cwd, env=env,
                            stdout=subprocess.PIPE if capture else None,
                            text=True, check=True)
    return result.stdout.strip() if capture else ''


def one(root, pattern):
    matches = [p for p in root.glob(pattern) if p.is_file() and p.stat().st_size]
    if len(matches) != 1:
        fail(f'Expected exactly one {pattern} under {root}; found {len(matches)}')
    return matches[0]


def emit(source, artifact_id, platform, kind, qualifier, signing='not-applicable'):
    destination = output / (prefix + qualifier)
    if source.resolve() != destination.resolve():
        shutil.copy2(source, destination)
    if destination.stat().st_size == 0:
        fail(f'Empty output: {artifact_id}')
    artifacts.append(dict(id=artifact_id, platform=platform, kind=kind,
                          filename=destination.name, signing=signing))


def pe(path):
    with path.open('rb') as stream:
        if stream.read(2) != b'MZ':
            fail('NSIS output is not a PE executable')
        stream.seek(60)
        offset = struct.unpack('<I', stream.read(4))[0]
        stream.seek(offset)
        if stream.read(4) != b'PE\0\0':
            fail('NSIS PE signature invalid')


def sdk_tool(name):
    direct = shutil.which(name)
    if direct:
        return direct
    sdk = Path(env.get('ANDROID_HOME', env.get('ANDROID_SDK_ROOT', str(Path.home() / 'Android/Sdk'))))
    matches = sorted(sdk.glob(f'build-tools/*/{name}'), reverse=True)
    if not matches:
        fail(f'Android SDK tool missing: {name}')
    return str(matches[0])


def require_abis(path, root):
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        for abi in ('arm64-v8a', 'armeabi-v7a', 'x86', 'x86_64'):
            if not any(name.startswith(f'{root}{abi}/') and not name.endswith('/')
                       and archive.getinfo(name).file_size for name in names):
                fail(f'{path.name} lacks nonempty {abi} contents')


try:
    if lane == 'linux':
        shutil.rmtree(target / 'release/bundle', ignore_errors=True)
        run(['cargo', 'tauri', 'build', '--bundles', 'deb,appimage,rpm', '--config', config, '--', '--locked'])
        bundle = target / 'release/bundle'
        deb = one(bundle / 'deb', '*.deb')
        for field, expected in [('Package', 'en-zime-reader'), ('Version', version), ('Architecture', 'amd64')]:
            if run(['dpkg-deb', '--field', deb, field], capture=True) != expected:
                fail(f'Debian {field} mismatch')
        rpm = one(bundle / 'rpm', '*.rpm')
        actual = run(['rpm', '-qp', '--queryformat', '%{NAME}\n%{VERSION}\n%{ARCH}', rpm], capture=True)
        if actual.splitlines() != ['en-zime-reader', version, 'x86_64']:
            fail('RPM identity/version/architecture mismatch')
        appimage = one(bundle / 'appimage', '*.AppImage')
        if not os.access(appimage, os.X_OK):
            fail('AppImage is not executable')
        emit(deb, 'linux-deb', 'linux', 'deb', 'linux-amd64.deb')
        emit(rpm, 'linux-rpm', 'linux', 'rpm', 'linux-amd64.rpm')
        emit(appimage, 'linux-appimage', 'linux', 'appimage', 'linux-amd64.AppImage')
    elif lane == 'windows-nsis':
        shutil.rmtree(target / 'x86_64-pc-windows-msvc/release/bundle', ignore_errors=True)
        run(['cargo', 'tauri', 'build', '--runner', 'cargo-xwin', '--target',
             'x86_64-pc-windows-msvc', '--bundles', 'nsis', '--config', config, '--', '--locked'])
        installer = one(target / 'x86_64-pc-windows-msvc/release/bundle/nsis', '*.exe')
        if version not in installer.name:
            fail('NSIS filename does not contain prepared version')
        pe(installer)
        signing = 'unsigned'
        if shutil.which('osslsigncode'):
            checked = subprocess.run(['osslsigncode', 'verify', '-in', str(installer)], env=env)
            if checked.returncode == 0:
                signing = 'verified'
        emit(installer, 'windows-nsis', 'windows', 'nsis', 'windows-x64-setup.exe', signing)
    elif lane == 'android':
        keystore = Path(env.get('ENZIME_KEYSTORE_PATH', str(snapshot / 'signing/production.keystore')))
        if not keystore.is_file():
            fail('Production keystore unavailable; set ENZIME_KEYSTORE_PATH')
        if not env.get('ENZIME_KEYSTORE_PASSWORD') or not env.get('ENZIME_KEY_PASSWORD'):
            fail('Production signing password environment is not provisioned')
        alias = env.get('ENZIME_KEYSTORE_ALIAS', 'upload')
        run(['cargo', 'tauri', 'android', 'build', '--apk', '--aab', '--config', config, '--', '--locked'])
        android = app / 'src-tauri/gen/android/app/build/outputs'
        apk = one(android / 'apk', '**/*universal*release*unsigned.apk')
        aab = one(android / 'bundle', '**/*release.aab')
        require_abis(apk, 'lib/')
        require_abis(aab, 'base/lib/')
        signed_apk = output / (prefix + 'android-universal-release-signed.apk')
        signed_aab = output / (prefix + 'android-universal-release-signed.aab')
        aligned = target / 'release-aligned.apk'
        run([sdk_tool('zipalign'), '-f', '-p', '4', apk, aligned])
        run([sdk_tool('apksigner'), 'sign', '--ks', keystore, '--ks-key-alias', alias,
             '--ks-pass', 'env:ENZIME_KEYSTORE_PASSWORD', '--key-pass', 'env:ENZIME_KEY_PASSWORD',
             '--out', signed_apk, aligned])
        run([sdk_tool('apksigner'), 'verify', '--verbose', signed_apk])
        shutil.copy2(aab, signed_aab)
        run(['jarsigner', '-keystore', keystore, '-storepass:env', 'ENZIME_KEYSTORE_PASSWORD',
             '-keypass:env', 'ENZIME_KEY_PASSWORD', signed_aab, alias])
        verified = run(['jarsigner', '-verify', signed_aab], capture=True)
        if 'jar verified.' not in verified:
            fail('AAB signature did not verify')
        analyzer = shutil.which('apkanalyzer')
        bundletool = shutil.which('bundletool')
        if not analyzer or not bundletool:
            fail('apkanalyzer and bundletool are required for version validation')
        for field, expected in [('version-name', version), ('version-code', str(version_data['versionCode'])),
                                ('application-id', config_data['identifier'])]:
            if run([analyzer, 'manifest', field, signed_apk], capture=True) != expected:
                fail(f'APK {field} mismatch')
        import xml.etree.ElementTree as ET
        manifest = ET.fromstring(run([bundletool, 'dump', 'manifest', '--bundle=' + str(signed_aab),
                                      '--module=base'], capture=True))
        ns = '{http://schemas.android.com/apk/res/android}'
        if (manifest.attrib.get(ns + 'versionName') != version or
            manifest.attrib.get(ns + 'versionCode') != str(version_data['versionCode']) or
            manifest.attrib.get('package') != config_data['identifier']):
            fail('AAB manifest identity/version mismatch')
        symbols = one(android / 'native-debug-symbols', '**/native-debug-symbols.zip')
        require_abis(symbols, '')
        mapping = one(android / 'mapping', '**/release/mapping.txt')
        symbol_out = output / (prefix + 'android-native-debug-symbols.zip')
        shutil.copy2(symbols, symbol_out)
        with zipfile.ZipFile(symbol_out, 'a', compression=zipfile.ZIP_DEFLATED) as archive:
            archive.write(mapping, 'mapping.txt')
        emit(signed_apk, 'android-apk', 'android', 'apk', 'android-universal-release-signed.apk', 'verified')
        emit(signed_aab, 'android-aab', 'android', 'aab', 'android-universal-release-signed.aab', 'verified')
        emit(symbol_out, 'android-symbols', 'android', 'symbols', 'android-native-debug-symbols.zip')
    elif lane == 'windows-native':
        executor = env.get('ENZIME_WINDOWS_BUILD_EXECUTOR', '')
        if not executor:
            fail('Native Windows MSI/MSIX executor is not configured', 69)
        if not Path(executor).is_absolute() or not os.access(executor, os.X_OK):
            fail('Native Windows executor must be an absolute executable')
        source = env.get('ENZIME_PREPARED_SOURCE_ARCHIVE', '')
        if not source or not Path(source).is_file():
            fail('Prepared source archive path missing: ENZIME_PREPARED_SOURCE_ARCHIVE')
        run([executor, source, output, version])
        # Executor evidence schema: version plus artifacts keyed by windows-msi/windows-msix.
        # Each artifact requires version, identity, sha256, packageVerified, signingVerified.
        report = read_json(output / 'native-verification.json')
        if report.get('version') != version:
            fail('Native Windows verification version mismatch')
        for artifact_id, extension in [('windows-msi', 'msi'), ('windows-msix', 'msix')]:
            path = output / (prefix + 'windows-x64.' + extension)
            evidence = report.get('artifacts', {}).get(artifact_id, {})
            if (evidence.get('version') != version or evidence.get('identity') != config_data['identifier']
                    or evidence.get('sha256') != hashlib.sha256(path.read_bytes()).hexdigest()
                    or evidence.get('packageVerified') is not True):
                fail(f'Native package evidence incomplete: {artifact_id}')
            signed = evidence.get('signingVerified') is True
            if extension == 'msix' and not signed:
                fail('MSIX signature verification required')
            emit(path, artifact_id, 'windows', extension, 'windows-x64.' + extension,
                 'verified' if signed else 'unsigned')
    elif lane == 'sbom':
        run(['cargo', 'cyclonedx', '--all', '--format', 'json', '--spec-version', '1.5', '--target', 'all'], cwd=snapshot)
        documents = [read_json(path) for path in snapshot.rglob('*.cdx.json')
                     if 'node_modules' not in path.relative_to(snapshot).parts
                     and 'target' not in path.relative_to(snapshot).parts]
        if not documents:
            fail('cargo cyclonedx produced no documents')
        components, dependencies, cargo_roots = {}, {}, set()
        for document in documents:
            if document.get('bomFormat') != 'CycloneDX':
                fail('Invalid generated CycloneDX document')
            cargo_components = list(document.get('components', []))
            metadata_component = document.get('metadata', {}).get('component')
            if metadata_component:
                cargo_components.append(metadata_component)
                cargo_roots.add(metadata_component.get('purl') or metadata_component.get('bom-ref'))
            original_aliases = {c.get('bom-ref', c.get('purl')): c.get('purl', c.get('bom-ref'))
                                for c in cargo_components}
            for component in cargo_components:
                identity = component.get('purl') or component.get('bom-ref')
                if not identity:
                    fail('SBOM component lacks identity')
                old_ref = component.get('bom-ref', identity)
                component['bom-ref'] = identity
                component.setdefault('scope', 'required')
                components[identity] = component
                # Cargo refs are normally purls; preserve aliases for dependency remapping below.
                component.setdefault('properties', []).append({'name': 'enzime:original-bom-ref', 'value': old_ref})
            aliases = original_aliases
            for dependency in document.get('dependencies', []):
                ref = aliases.get(dependency['ref'], dependency['ref'])
                dependencies.setdefault(ref, set()).update(aliases.get(d, d) for d in dependency.get('dependsOn', []))
        inventory_text = run(['pnpm', 'list', '--prod', '--json', '--depth', 'Infinity', '-r'], cwd=snapshot, capture=True)
        try:
            inventory = json.loads(inventory_text)
        except ValueError as exc:
            fail(f'Invalid pnpm inventory JSON: {exc}')
        root_ref = 'pkg:generic/enzime-reader@' + version
        dependencies.setdefault(root_ref, set()).update(cargo_roots)
        def npm_deps(items, parent):
            for name, item in items.items():
                package_version = item.get('version', '')
                if package_version.startswith(('link:', 'file:')):
                    npm_deps(item.get('dependencies', {}), parent)
                    continue
                from urllib.parse import quote
                ref = 'pkg:npm/' + quote(name, safe='/') + '@' + quote(package_version, safe='')
                components[ref] = {'type': 'library', 'name': name, 'version': package_version,
                                   'purl': ref, 'bom-ref': ref, 'scope': 'required'}
                dependencies.setdefault(parent, set()).add(ref)
                dependencies.setdefault(ref, set())
                npm_deps(item.get('dependencies', {}), ref)
        for project in inventory:
            npm_deps(project.get('dependencies', {}), root_ref)
        # Resolve preserved refs after all Cargo documents have been combined.
        aliases = {p['value']: ref for ref, c in components.items() for p in c.get('properties', [])
                   if p['name'] == 'enzime:original-bom-ref'}
        merged = {}
        for ref, children in dependencies.items():
            merged.setdefault(aliases.get(ref, ref), set()).update(aliases.get(c, c) for c in children)
        known = set(components) | {root_ref}
        if any(ref not in known or not children <= known for ref, children in merged.items()):
            fail('CycloneDX dependency graph contains unresolved references')
        sbom = output / (prefix + 'sbom.cdx.json')
        sbom.write_text(json.dumps({'bomFormat': 'CycloneDX', 'specVersion': '1.5', 'version': 1,
            'metadata': {'component': {'type': 'application', 'name': 'EnZIME Reader', 'version': version, 'bom-ref': root_ref}},
            'components': list(components.values()),
            'dependencies': [{'ref': ref, 'dependsOn': sorted(children)} for ref, children in merged.items()]}, indent=2) + '\n')
        emit(sbom, 'sbom', 'all', 'sbom', 'sbom.cdx.json')
    (output / 'artifacts.json').write_text(json.dumps(artifacts, indent=2) + '\n')
except subprocess.CalledProcessError as exc:
    fail(f'{lane} command failed with exit {exc.returncode}', exc.returncode or 1)
except (OSError, ValueError, KeyError, zipfile.BadZipFile) as exc:
    fail(f'{lane} validation failed: {exc}')
PY
