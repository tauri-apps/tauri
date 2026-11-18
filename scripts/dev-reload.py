#!/usr/bin/env python3
"""Build opt-in Rust workers and atomically publish immutable generations."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import uuid

CANCELLED = 'cancelled'
UUID_HEX_RE = re.compile(r'^[0-9a-f]{32}$')


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


def terminate_child(child):
    if child.poll() is not None:
        return
    if os.name != 'nt':
        try:
            os.killpg(child.pid, signal.SIGTERM)
        except ProcessLookupError:
            return
    else:
        child.terminate()
    try:
        child.wait(timeout=2)
    except subprocess.TimeoutExpired:
        if os.name != 'nt':
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        else:
            child.kill()
        child.wait(timeout=2)


def run_cargo_stream(command, bin_name, should_cancel=None):
    """Run a cargo-compatible build command and return (exit_code_or_CANCELLED, artifact_paths)."""
    artifacts = []
    try:
        child = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            text=True,
            start_new_session=(os.name != 'nt'),
        )
    except OSError as err:
        print(f'Failed to start builder ({command[0]}): {err}', file=sys.stderr, flush=True)
        return 127, []

    lines = queue.Queue()

    def drain():
        try:
            for line in child.stdout:
                lines.put(line)
        finally:
            lines.put(None)

    reader = threading.Thread(target=drain, daemon=True)
    reader.start()
    cancelled = False
    try:
        while True:
            if should_cancel is not None and should_cancel():
                cancelled = True
                terminate_child(child)
                break
            try:
                line = lines.get(timeout=0.1)
            except queue.Empty:
                continue
            if line is None:
                break
            try:
                event = json.loads(line)
            except ValueError:
                print(line, end='', flush=True)
                continue
            if event.get('reason') == 'compiler-message':
                print(event['message'].get('rendered', ''), end='', file=sys.stderr)
            if (event.get('reason') == 'compiler-artifact'
                    and event.get('target', {}).get('name') == bin_name
                    and event.get('executable')):
                artifacts.append(event['executable'])
        status = child.wait()
    finally:
        terminate_child(child)
        reader.join(timeout=2)
        if child.stdout and not child.stdout.closed:
            child.stdout.close()

    if cancelled:
        return CANCELLED, []
    return status, artifacts


def verify_worker(executable, is_remote=False):
    """Verify the built executable runs on this host and emits protocol v1 handshake."""
    hint = (
        ' (if remote OS/arch/libc differs from this host, pass '
        '--runner-arg=--target=<local-triple>)'
        if is_remote else ''
    )
    try:
        child = subprocess.Popen(
            [str(executable)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    except OSError as err:
        raise RuntimeError(f'Worker preflight failed to start executable: {err}{hint}') from err

    line_box = []
    err_box = []

    def read_hello():
        try:
            line_box.append(child.stdout.readline())
        except Exception as exc:  # noqa: BLE001
            err_box.append(str(exc))

    try:
        reader = threading.Thread(target=read_hello, daemon=True)
        reader.start()
        reader.join(timeout=3.0)
        if child.stdin:
            try:
                child.stdin.close()
            except OSError:
                pass
        if reader.is_alive():
            child.kill()
            child.wait(timeout=2)
            reader.join(timeout=1)
            raise RuntimeError('Worker preflight timed out waiting for protocol handshake')
        try:
            child.wait(timeout=1.0)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=2)
        stderr_out = child.stderr.read().strip() if child.stderr else ''
    finally:
        if child.poll() is None:
            child.kill()
            child.wait(timeout=2)
        for pipe in (child.stdin, child.stdout, child.stderr):
            if pipe and not pipe.closed:
                pipe.close()

    if err_box:
        raise RuntimeError(f'Worker preflight failed reading stdout: {err_box[0]}')
    raw = line_box[0] if line_box else ''
    try:
        hello = json.loads(raw) if raw else None
    except ValueError:
        hello = None
    if hello != {'protocol': 1}:
        detail = stderr_out.splitlines()[0] if stderr_out else (raw.strip() or f'exit code {child.returncode}')
        raise RuntimeError(f'Worker preflight rejected handshake ({detail}){hint}')


def prune_generations(output_dir, keep):
    """Best-effort cleanup of older generation directories while keeping `current` and newest `keep`."""
    if keep <= 0:
        return
    active_dir = None
    try:
        active_path = Path((output_dir / 'current').read_text().strip())
        active_dir = active_path.parent.resolve()
    except OSError:
        pass

    candidates = []
    try:
        entries = list(output_dir.iterdir())
    except OSError:
        return
    for entry in entries:
        if not UUID_HEX_RE.match(entry.name):
            continue
        try:
            resolved = entry.resolve()
            if not entry.is_dir() or (active_dir is not None and resolved == active_dir):
                continue
            candidates.append((entry.stat().st_mtime_ns, entry))
        except OSError:
            continue

    candidates.sort(key=lambda item: item[0], reverse=True)
    max_other = max(0, keep - (1 if active_dir is not None else 0))
    for _, old_dir in candidates[max_other:]:
        shutil.rmtree(old_dir, ignore_errors=True)


def publish_generation(output_dir, artifact_name, place_fn, started, keep=5, is_remote=False):
    """Create an immutable generation directory, verify it, atomically update `current`, and prune old dirs."""
    generation = output_dir / uuid.uuid4().hex
    generation.mkdir(parents=True)
    executable = generation / artifact_name
    try:
        place_fn(executable)
        if os.name != 'nt':
            executable.chmod(executable.stat().st_mode | 0o111)
        verify_worker(executable, is_remote=is_remote)
        # The running host only observes fully copied and verified artifacts. Keep recent
        # generations: Windows may still have an active binary open during switchover.
        temporary = generation / 'current.tmp'
        temporary.write_text(str(executable.resolve()) + '\n')
        os.replace(temporary, output_dir / 'current')
    except Exception:
        shutil.rmtree(generation, ignore_errors=True)
        raise
    prune_generations(output_dir, keep)
    print(f'Published {generation.name} in {time.monotonic() - started:.2f}s', flush=True)
    return True


def normalize_remote_dir(raw):
    value = raw.strip()
    if value.startswith('-'):
        raise ValueError('--remote-dir must not start with -')
    if value in {'~', '~/'}:
        raise ValueError('--remote-dir must not be home root (~)')
    if value.startswith('~/'):
        value = value[2:]
    if value != '/':
        value = value.rstrip('/')
    parts = [p for p in value.split('/') if p]
    if not parts or value in {'', '.', '..', '/'} or '.' in parts or '..' in parts:
        raise ValueError('--remote-dir must be a dedicated directory path, not root or relative traversal')
    return ('/' + '/'.join(parts)) if value.startswith('/') else '/'.join(parts)


def default_remote_dir(sync_root):
    safe_name = re.sub(r'[^A-Za-z0-9._-]+', '_', sync_root.name).strip('_') or 'worker'
    digest = hashlib.sha256(str(sync_root).encode('utf-8')).hexdigest()[:12]
    return f'.cache/tauri-dev-reload/{safe_name}-{digest}'


def remote_shell_path(path_str):
    if path_str.startswith('/'):
        return shlex.quote(path_str)
    return '"$HOME/"' + shlex.quote(path_str)


def artifact_basename(path_str):
    return path_str.replace('\\', '/').rsplit('/', 1)[-1]


def rsync_rsh_args(ssh_opts):
    if not ssh_opts or 'RSYNC_RSH' in os.environ:
        return []
    return ['-e', ' '.join(['ssh', *(shlex.quote(o) for o in ssh_opts)])]


def build_local(args, started, should_cancel=None):
    command = [args.runner, 'build', *args.runner_arg, '--manifest-path', str(args.manifest),
               '--bin', args.bin, '--message-format=json-render-diagnostics']
    print('Building reload worker:', ' '.join(command), flush=True)
    status, artifacts = run_cargo_stream(command, args.bin, should_cancel=should_cancel)
    if status == CANCELLED:
        print('Source changed during build; restarting...', flush=True)
        return False
    if status or not artifacts:
        print('Build rejected; previous generation remains active.', flush=True)
        return False
    artifact = Path(artifacts[-1])
    try:
        return publish_generation(
            args.output, artifact.name, lambda dst: shutil.copy2(artifact, dst),
            started, keep=args.keep,
        )
    except (OSError, RuntimeError) as err:
        print(f'{err}; previous generation remains active.', flush=True)
        return False


def build_remote(args, started, should_cancel=None):
    ssh_opts = getattr(args, '_ssh_opts', [])
    rsh_args = rsync_rsh_args(ssh_opts)
    remote_workspace = f'{args.remote_dir}/workspace'
    rel_manifest = args.manifest.relative_to(args.sync_root).as_posix()
    remote_manifest = f'{remote_workspace}/{rel_manifest}'
    if not getattr(args, '_remote_ready', False):
        mkdir_cmd = f'mkdir -p {remote_shell_path(remote_workspace)}'
        if subprocess.run(['ssh', *ssh_opts, '--', args.remote, mkdir_cmd], check=False).returncode != 0:
            print('Remote directory setup failed; previous generation remains active.', flush=True)
            return False
        args._remote_ready = True

    if should_cancel is not None and should_cancel():
        print('Source changed during build; restarting...', flush=True)
        return False

    print(f'Syncing source to {args.remote}:{remote_workspace}...', flush=True)
    excludes = ['--exclude', '/target/', '--exclude', '/.git/', '--exclude', '.reload/']
    if args.output.is_relative_to(args.sync_root):
        excludes.extend(['--exclude', f'/{args.output.relative_to(args.sync_root).as_posix()}/'])
    rsync_cmd = [
        'rsync', '-az', '--delete', *rsh_args, *excludes,
        f'{args.sync_root}/', f'{args.remote}:{remote_workspace}/',
    ]
    if subprocess.run(rsync_cmd, check=False).returncode != 0:
        args._remote_ready = False
        print('Remote sync failed; previous generation remains active.', flush=True)
        return False

    remote_cargo_cmd = 'export PATH="$PATH:$HOME/.cargo/bin"; ' + ' '.join([
        shlex.quote(args.remote_cargo), 'build',
        *(shlex.quote(a) for a in args.runner_arg),
        '--manifest-path', remote_shell_path(remote_manifest),
        '--bin', shlex.quote(args.bin),
        '--message-format=json-render-diagnostics',
    ])
    print(f'Building reload worker on {args.remote}: {args.remote_cargo} build ...', flush=True)
    status, artifacts = run_cargo_stream(
        ['ssh', *ssh_opts, '--', args.remote, remote_cargo_cmd],
        args.bin,
        should_cancel=should_cancel,
    )
    if status == CANCELLED:
        print('Source changed during build; restarting...', flush=True)
        return False
    if status or not artifacts:
        print('Build rejected; previous generation remains active.', flush=True)
        return False
    remote_artifact = artifacts[-1]
    name = artifact_basename(remote_artifact)

    def fetch(dst):
        res = subprocess.run(
            ['rsync', '-az', *rsh_args, f'{args.remote}:{remote_artifact}', str(dst)],
            check=False,
        )
        if res.returncode != 0:
            raise RuntimeError('Failed to fetch remote artifact')

    try:
        return publish_generation(args.output, name, fetch, started, keep=args.keep, is_remote=True)
    except (OSError, RuntimeError) as err:
        print(f'{err}; previous generation remains active.', flush=True)
        return False


def build(args, should_cancel=None):
    started = time.monotonic()
    if args.remote:
        return build_remote(args, started, should_cancel=should_cancel)
    return build_local(args, started, should_cancel=should_cancel)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--bin', required=True)
    parser.add_argument('--runner', default='cargo', help='cargo or a Cargo-compatible builder such as justrust')
    parser.add_argument('--runner-arg', action='append', default=[], help='builder option, e.g. --runner-arg=--local')
    parser.add_argument('--remote', help='optional SSH host for remote Cargo builds (e.g. user@buildbox)')
    parser.add_argument('--remote-dir', help='remote staging directory (defaults to ~/.cache/tauri-dev-reload/<name>-<hash>)')
    parser.add_argument('--remote-sync-root', type=Path, help='local source root to sync for --remote (defaults to --manifest directory)')
    parser.add_argument('--remote-cargo', default='cargo', help='cargo binary on the remote host')
    parser.add_argument('--no-ssh-multiplex', action='store_true', help='disable automatic SSH ControlMaster multiplexing')
    parser.add_argument('--watch', type=Path, action='append', default=[], help='extra local dependency source root')
    parser.add_argument('--output', type=Path, required=True, help='place outside watched source roots')
    parser.add_argument('--keep', type=int, default=5, help='max published generations to keep locally (default: 5, 0 keeps all)')
    parser.add_argument('--once', action='store_true')
    args = parser.parse_args()
    if args.keep < 0:
        parser.error('--keep must be >= 0')
    args.manifest = args.manifest.resolve()
    args.output = args.output.resolve()
    if not args.manifest.is_file():
        parser.error(f'--manifest does not exist: {args.manifest}')
    roots = [args.manifest.parent, *(p.resolve() for p in args.watch)]
    check_roots = list(roots)
    if args.remote:
        if args.remote.startswith('-'):
            parser.error('--remote must not start with -')
        for tool in ('ssh', 'rsync'):
            if shutil.which(tool) is None:
                parser.error(f'--remote requires `{tool}` in PATH')
        args.sync_root = args.remote_sync_root.resolve() if args.remote_sync_root else args.manifest.parent
        if not args.sync_root.is_dir():
            parser.error(f'--remote-sync-root is not a directory: {args.sync_root}')
        if not args.manifest.is_relative_to(args.sync_root):
            parser.error('--manifest must be inside --remote-sync-root')
        for watch_root in roots:
            if not watch_root.is_relative_to(args.sync_root):
                parser.error(
                    '--watch paths outside --manifest directory require '
                    '--remote-sync-root covering them when using --remote'
                )
        try:
            args.remote_dir = normalize_remote_dir(args.remote_dir) if args.remote_dir else default_remote_dir(args.sync_root)
        except ValueError as err:
            parser.error(str(err))
        check_roots.append(args.sync_root)
    if any(args.output.is_relative_to(root) and '.reload' not in args.output.relative_to(root).parts
           and 'target' not in args.output.relative_to(root).parts for root in check_roots):
        parser.error('--output must be outside watched/synced roots, or under target/ or .reload/')
    args.output.mkdir(parents=True, exist_ok=True)

    ssh_ctl_dir = None
    args._ssh_opts = []
    if args.remote and os.name != 'nt' and not args.no_ssh_multiplex:
        short_tmp = '/tmp' if Path('/tmp').is_dir() else None
        ssh_ctl_dir = Path(tempfile.mkdtemp(prefix='tauri-ssh-', dir=short_tmp))
        ctl_path = ssh_ctl_dir / 'c'
        args._ssh_opts = [
            '-o', 'ControlMaster=auto',
            '-o', f'ControlPath={ctl_path}',
            '-o', 'ControlPersist=60',
        ]

    def current():
        return [snapshot(root) for root in roots]

    try:
        previous = current()
        success = build(args)
        if args.once:
            return 0 if success else 1
        # Snapshot before each build; interrupt stale builds immediately if new edits arrive.
        while True:
            time.sleep(0.15)
            changed = current()
            if changed != previous:
                time.sleep(0.15)
                previous = current()
                build(args, should_cancel=lambda: current() != previous)
    finally:
        if ssh_ctl_dir is not None:
            ctl_path = ssh_ctl_dir / 'c'
            if ctl_path.exists():
                subprocess.run(
                    ['ssh', '-O', 'exit', '-o', f'ControlPath={ctl_path}', '--', args.remote],
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    check=False,
                )
            shutil.rmtree(ssh_ctl_dir, ignore_errors=True)


if __name__ == '__main__':
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        sys.exit(130)
