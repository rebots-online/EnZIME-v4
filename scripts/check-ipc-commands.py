#!/usr/bin/env python3
"""Fail when the frontend names a Tauri command that the Rust side does not register.

Release standard R1/R4: no frontend command name is unregistered. Compares command
names used by apps/reader/src (first string argument of invoke(...) or of a service's
call(...) wrapper, and the values of `const COMMANDS = {...}` tables) with the
`tauri::generate_handler![...]` list in apps/reader/src-tauri/src/lib.rs.
"""
import argparse
from pathlib import Path
import re
import sys

CALL = re.compile(r'\b(?:invoke|call)\s*(?:<[^()]*?>)?\s*\(\s*"([A-Za-z0-9_]+)"', re.S)
TABLE = re.compile(r'\bconst\s+COMMANDS\s*=\s*\{(.*?)\}', re.S)
VALUE = re.compile(r':\s*"([A-Za-z0-9_]+)"')
HANDLERS = re.compile(r'generate_handler!\s*\[(.*?)\]', re.S)


def uncomment(text):
    text = re.sub(r'/\*.*?\*/', '', text, flags=re.S)
    return re.sub(r'(?m)^\s*//.*$|(?<=[\s;,{(])//.*$', '', text)


def frontend(source):
    used = {}
    for path in sorted(source.rglob('*.ts*')):
        if '.test.' in path.name or path.name.endswith('.d.ts'):
            continue
        text = uncomment(path.read_text(encoding='utf-8'))
        names = set(CALL.findall(text))
        for table in TABLE.findall(text):
            names.update(VALUE.findall(table))
        for name in names:
            used.setdefault(name, []).append(path)
    return used


def registered(lib):
    lists = HANDLERS.findall(uncomment(lib.read_text(encoding='utf-8')))
    if len(lists) != 1:
        raise ValueError(f'Expected exactly one generate_handler! list in {lib}')
    names = [entry.strip().rsplit('::', 1)[-1] for entry in lists[0].split(',') if entry.strip()]
    if len(names) != len(set(names)):
        raise ValueError('Duplicate command name in generate_handler!')
    return set(names)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path, nargs='?', default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args(argv)
    try:
        used = frontend(args.root / 'apps/reader/src')
        known = registered(args.root / 'apps/reader/src-tauri/src/lib.rs')
    except (OSError, ValueError) as error:
        print(f'Command check failed: {error}', file=sys.stderr)
        return 1
    if not used:
        print('Command check failed: no frontend command names found', file=sys.stderr)
        return 1
    missing = sorted(set(used) - known)
    for name in missing:
        places = ', '.join(str(p.relative_to(args.root)) for p in used[name])
        print(f'Unregistered command "{name}" used by {places}', file=sys.stderr)
    unused = sorted(known - set(used))
    print(f'Commands: {len(used)} used by the frontend, {len(known)} registered, '
          f'{len(missing)} unregistered, {len(unused)} registered but not called'
          + (f' ({", ".join(unused)})' if unused else ''))
    return 1 if missing else 0


if __name__ == '__main__':
    raise SystemExit(main())
