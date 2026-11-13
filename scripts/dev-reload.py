#!/usr/bin/env python3
"""Build opt-in Rust workers and atomically publish immutable generations."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import uuid


def snapshot(root):
    result = {}
    for directory, folders, files in os.walk(root):
        folders[:] = [p for p in folders if p not in {'target', '.git', '.reload'}]
        for name in files:
            path = Path(directory) / name
            try:
                stat = path.stat()
                result[str(path)] = (stat.st_mtime_ns, stat.st_size)
            except FileNotFoundError:
                pass  # Editors can atomically replace files while we scan.
    return result


def build(args):
    command = [args.runner, 'build', *args.runner_arg, '--manifest-path', str(args.manifest),
               '--bin', args.bin, '--message-format=json-render-diagnostics']
    started = time.monotonic()
    print('Building reload worker:', ' '.join(command), flush=True)
    artifacts = []
    with subprocess.Popen(command, stdout=subprocess.PIPE, text=True) as child:
        for line in child.stdout:
            try:
                event = json.loads(line)
            except ValueError:
                print(line, end='', flush=True)
                continue
            if event.get('reason') == 'compiler-message':
                print(event['message'].get('rendered', ''), end='', file=sys.stderr)
            if (event.get('reason') == 'compiler-artifact'
                    and event.get('target', {}).get('name') == args.bin
                    and event.get('executable')):
                artifacts.append(Path(event['executable']))
        status = child.wait()
    if status or not artifacts:
        print('Build rejected; previous generation remains active.', flush=True)
        return False
    generation = args.output / uuid.uuid4().hex
    generation.mkdir(parents=True)
    executable = generation / artifacts[-1].name
    shutil.copy2(artifacts[-1], executable)
    # The running host only observes fully copied artifacts. Keep older files:
    # Windows may still have them open, and an existing host may still use them.
    temporary = generation / 'current.tmp'
    temporary.write_text(str(executable.resolve()) + '\n')
    os.replace(temporary, args.output / 'current')
    print(f'Published {generation.name} in {time.monotonic() - started:.2f}s', flush=True)
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--bin', required=True)
    parser.add_argument('--runner', default='cargo', help='cargo or a Cargo-compatible builder such as justrust')
    parser.add_argument('--runner-arg', action='append', default=[], help='builder option, e.g. --runner-arg=--local')
    parser.add_argument('--watch', type=Path, action='append', default=[], help='extra local dependency source root')
    parser.add_argument('--output', type=Path, required=True, help='place outside watched source roots')
    parser.add_argument('--once', action='store_true')
    args = parser.parse_args()
    args.manifest = args.manifest.resolve()
    args.output = args.output.resolve()
    roots = [args.manifest.parent, *(p.resolve() for p in args.watch)]
    if any(args.output.is_relative_to(root) and '.reload' not in args.output.relative_to(root).parts
           and 'target' not in args.output.relative_to(root).parts for root in roots):
        parser.error('--output must be outside watched roots, or under target/ or .reload/')
    args.output.mkdir(parents=True, exist_ok=True)
    def current():
        return [snapshot(root) for root in roots]
    previous = current()
    success = build(args)
    if args.once:
        return 0 if success else 1
    # Snapshot before each build so edits during compilation trigger another build.
    while True:
        time.sleep(0.15)
        changed = current()
        if changed != previous:
            time.sleep(0.15)
            previous = current()
            build(args)


if __name__ == '__main__':
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        sys.exit(130)
