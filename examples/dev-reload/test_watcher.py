"""Actual Cargo edit/failure/recovery acceptance test; no GUI dependencies."""
import os
from pathlib import Path
import queue
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'scripts/dev-reload.py'


def make_ssh_loopback(bin_dir: Path, remote_home: Path) -> Path:
    """Create an `ssh` shim that executes remote commands locally under `remote_home`."""
    cargo_home = os.environ.get('CARGO_HOME', str(Path.home() / '.cargo'))
    rustup_home = os.environ.get('RUSTUP_HOME', str(Path.home() / '.rustup'))
    ssh_shim = bin_dir / 'ssh'
    ssh_shim.write_text(
        '#!/usr/bin/env python3\n'
        'import os, subprocess, sys\n'
        'argv = sys.argv[1:]\n'
        'while argv and argv[0].startswith("-"):\n'
        '    flag = argv.pop(0)\n'
        '    if flag == "--":\n'
        '        break\n'
        '    if flag in {"-l", "-p", "-i", "-o", "-F", "-S", "-b", "-c", "-D", "-E", "-e", "-I", "-J", "-L", "-m", "-O", "-Q", "-R", "-W", "-w"}:\n'
        '        argv.pop(0)\n'
        'argv.pop(0)  # discard host\n'
        'env = dict(os.environ,\n'
        f'           HOME={str(remote_home)!r},\n'
        f'           CARGO_HOME={cargo_home!r},\n'
        f'           RUSTUP_HOME={rustup_home!r})\n'
        'sys.exit(subprocess.run(" ".join(argv), shell=True, cwd=env["HOME"], env=env).returncode)\n'
    )
    ssh_shim.chmod(0o755)
    return ssh_shim


class WatcherTest(unittest.TestCase):
    def test_publish_failure_and_recovery(self):
        with tempfile.TemporaryDirectory(prefix='tauri-reload-') as directory:
            root = Path(directory)
            shutil.copytree(Path(__file__).parent / 'backend', root / 'backend',
                            ignore=shutil.ignore_patterns('target', '.reload'))
            output = root / 'generations'
            env = dict(os.environ, CARGO_NET_OFFLINE='true')
            child = subprocess.Popen([
                sys.executable, str(SCRIPT),
                '--manifest', str(root / 'backend/Cargo.toml'), '--bin', 'reload-counter',
                '--output', str(output),
            ], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=env)
            lines = queue.Queue()

            def read():
                for line in child.stdout:
                    lines.put(line)

            thread = threading.Thread(target=read, daemon=True)
            thread.start()

            def wait_for(message):
                deadline = time.monotonic() + 30
                seen = []
                while time.monotonic() < deadline:
                    try:
                        line = lines.get(timeout=0.2)
                    except queue.Empty:
                        continue
                    seen.append(line)
                    if message in line:
                        return
                self.fail('Missing ' + message + '\n' + ''.join(seen))

            try:
                wait_for('Published')
                first = (output / 'current').read_text()
                source = root / 'backend/src/main.rs'
                original = source.read_text()
                source.write_text('this is not valid Rust')
                wait_for('Build rejected')
                self.assertEqual((output / 'current').read_text(), first)
                source.write_text(original.replace('Rust generation one', 'Rust generation two'))
                wait_for('Published')
                second = (output / 'current').read_text()
                self.assertNotEqual(first, second)
                self.assertTrue(Path(first.strip()).is_file())
                result = subprocess.run([second.strip()], input='{"command":"increment","state":{"count":41}}\n',
                                        text=True, capture_output=True, check=True)
                self.assertIn('Rust generation two', result.stdout)
                self.assertIn('42', result.stdout)
            finally:
                child.terminate()
                child.wait(timeout=5)
                thread.join(timeout=2)
                child.stdout.close()

    @unittest.skipUnless(shutil.which('rsync'), 'rsync required for remote build test')
    def test_remote_build_default_cache_zero_local_target_and_recovery(self):
        with tempfile.TemporaryDirectory(prefix='tauri-remote-reload-') as directory:
            root = Path(directory)
            shutil.copytree(Path(__file__).parent / 'backend', root / 'backend',
                            ignore=shutil.ignore_patterns('target', '.reload'))
            # Ensure a source file named `target.rs` is NOT excluded by rsync `/target/` filter.
            (root / 'backend/src/target.rs').write_text('pub const TAG: &str = "remote-ok";\n')
            main_rs = root / 'backend/src/main.rs'
            main_rs.write_text(
                'mod target;\n' + main_rs.read_text().replace(
                    '"Rust generation one"', 'format!("Rust generation one ({})", target::TAG)'
                )
            )

            remote_home = root / 'remote_home'
            remote_home.mkdir()  # Empty: `.cache/tauri-dev-reload` does not exist yet!
            output = root / 'backend/.reload'
            bin_dir = root / 'bin'
            bin_dir.mkdir()
            ssh_shim = make_ssh_loopback(bin_dir, remote_home)

            env = dict(
                os.environ,
                CARGO_NET_OFFLINE='true',
                PATH=f'{bin_dir}{os.pathsep}{os.environ.get("PATH", "")}',
            )
            env.pop('RSYNC_RSH', None)
            child = subprocess.Popen([
                sys.executable, str(SCRIPT),
                '--manifest', str(root / 'backend/Cargo.toml'), '--bin', 'reload-counter',
                '--remote', 'user@mockbox',
                '--output', str(output),
            ], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=env)
            lines = queue.Queue()

            def read():
                for line in child.stdout:
                    lines.put(line)

            thread = threading.Thread(target=read, daemon=True)
            thread.start()

            def wait_for(message):
                deadline = time.monotonic() + 30
                seen = []
                while time.monotonic() < deadline:
                    try:
                        line = lines.get(timeout=0.2)
                    except queue.Empty:
                        continue
                    seen.append(line)
                    if message in line:
                        return
                self.fail('Missing ' + message + '\n' + ''.join(seen))

            try:
                wait_for('Published')
                # Local target/ directory must never be created during remote builds.
                self.assertFalse((root / 'backend/target').exists())
                self.assertTrue((remote_home / '.cache/tauri-dev-reload').is_dir())

                first = (output / 'current').read_text()
                original = main_rs.read_text()
                main_rs.write_text('this is not valid Rust')
                wait_for('Build rejected')
                self.assertEqual((output / 'current').read_text(), first)

                main_rs.write_text(original.replace('Rust generation one', 'Rust remote two'))
                wait_for('Published')
                second = (output / 'current').read_text()
                self.assertNotEqual(first, second)
                self.assertFalse((root / 'backend/target').exists())
                result = subprocess.run([second.strip()], input='{"command":"increment","state":{"count":41}}\n',
                                        text=True, capture_output=True, check=True)
                self.assertIn('Rust remote two (remote-ok)', result.stdout)
                self.assertIn('42', result.stdout)
            finally:
                child.terminate()
                child.wait(timeout=5)
                thread.join(timeout=2)
                child.stdout.close()

    @unittest.skipUnless(shutil.which('rsync'), 'rsync required for remote build test')
    def test_remote_preflight_rejection_and_cli_guardrails(self):
        with tempfile.TemporaryDirectory(prefix='tauri-remote-guard-') as directory:
            root = Path(directory)
            shutil.copytree(Path(__file__).parent / 'backend', root / 'backend',
                            ignore=shutil.ignore_patterns('target', '.reload'))
            output = root / 'generations'

            # CLI safety guardrails must reject dangerous arguments immediately.
            for bad_args in (
                ['--remote', '-oProxyCommand=bad'],
                ['--remote', 'box', '--remote-dir', '~'],
                ['--remote', 'box', '--remote-dir', '/'],
                ['--remote', 'box', '--remote-dir', '../escape'],
                ['--remote', 'box', '--watch', str(root)],
            ):
                res = subprocess.run(
                    [sys.executable, str(SCRIPT), '--manifest', str(root / 'backend/Cargo.toml'),
                     '--bin', 'reload-counter', '--output', str(output), '--once', *bad_args],
                    capture_output=True, text=True, check=False,
                )
                self.assertNotEqual(res.returncode, 0, f'Expected failure for {bad_args}')

            # Preflight check must reject a broken/incompatible remote binary without
            # creating `current` or leaking generation directories.
            remote_home = root / 'remote_home'
            remote_home.mkdir()
            bin_dir = root / 'bin'
            bin_dir.mkdir()
            ssh_shim = make_ssh_loopback(bin_dir, remote_home)
            fake_bad_bin = remote_home / 'bad-worker'
            fake_bad_bin.write_text('#!/bin/sh\necho "version GLIBC_2.99 not found" >&2\nexit 1\n')
            fake_bad_bin.chmod(0o755)
            fake_cargo = bin_dir / 'fake-remote-cargo'
            fake_cargo.write_text(
                '#!/usr/bin/env python3\n'
                'import json\n'
                f'print(json.dumps({{"reason": "compiler-artifact", "target": {{"name": "reload-counter"}}, "executable": {str(fake_bad_bin)!r}}}))\n'
            )
            fake_cargo.chmod(0o755)

            env = dict(
                os.environ,
                RSYNC_RSH=str(ssh_shim),
                PATH=f'{bin_dir}{os.pathsep}{os.environ.get("PATH", "")}',
            )
            res = subprocess.run(
                [sys.executable, str(SCRIPT), '--manifest', str(root / 'backend/Cargo.toml'),
                 '--bin', 'reload-counter', '--remote', 'mockbox',
                 '--remote-cargo', str(fake_cargo), '--output', str(output), '--once'],
                capture_output=True, text=True, env=env, check=False,
            )
            self.assertNotEqual(res.returncode, 0)
            self.assertIn('Worker preflight rejected handshake', res.stdout)
            self.assertIn('GLIBC_2.99', res.stdout)
            self.assertFalse((output / 'current').exists())
            self.assertEqual(list(output.iterdir()), [])

    def test_generation_pruning_and_cancel_on_edit(self):
        with tempfile.TemporaryDirectory(prefix='tauri-reload-gc-cancel-') as directory:
            root = Path(directory)
            shutil.copytree(Path(__file__).parent / 'backend', root / 'backend',
                            ignore=shutil.ignore_patterns('target', '.reload'))
            output = root / 'generations'
            bin_dir = root / 'bin'
            bin_dir.mkdir()

            # Build real worker once so our fast/slow runner wrapper can emit a valid executable.
            subprocess.run(
                ['cargo', 'build', '--offline', '--manifest-path', str(root / 'backend/Cargo.toml'),
                 '--bin', 'reload-counter'],
                check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            )
            exe_name = 'reload-counter.exe' if os.name == 'nt' else 'reload-counter'
            built_bin = root / 'backend/target/debug' / exe_name

            runner = bin_dir / 'mock-runner'
            runner.write_text(
                '#!/usr/bin/env python3\n'
                'import json, sys, time\n'
                'from pathlib import Path\n'
                f'src = Path({str(root / "backend/src/main.rs")!r}).read_text()\n'
                'if "SLOW_BUILD" in src:\n'
                '    time.sleep(10)\n'
                f'print(json.dumps({{"reason": "compiler-artifact", "target": {{"name": "reload-counter"}}, "executable": {str(built_bin)!r}}}), flush=True)\n'
            )
            runner.chmod(0o755)

            child = subprocess.Popen([
                sys.executable, str(SCRIPT),
                '--manifest', str(root / 'backend/Cargo.toml'), '--bin', 'reload-counter',
                '--runner', str(runner), '--keep', '2', '--output', str(output),
            ], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            lines = queue.Queue()

            def read():
                for line in child.stdout:
                    lines.put(line)

            thread = threading.Thread(target=read, daemon=True)
            thread.start()

            def wait_for(message, timeout=15):
                deadline = time.monotonic() + timeout
                seen = []
                while time.monotonic() < deadline:
                    try:
                        line = lines.get(timeout=0.1)
                    except queue.Empty:
                        continue
                    seen.append(line)
                    if message in line:
                        return
                self.fail('Missing ' + message + '\n' + ''.join(seen))

            try:
                source = root / 'backend/src/main.rs'
                original = source.read_text()
                wait_for('Published')  # gen 1

                # Trigger a 10s slow build, then edit again right away: cancel-on-edit must abort <4s!
                t0 = time.monotonic()
                source.write_text(original + '\n// SLOW_BUILD\n')
                wait_for('Building reload worker')
                time.sleep(0.25)
                source.write_text(original + '\n// gen 2\n')
                wait_for('Source changed during build; restarting...', timeout=4)
                wait_for('Published', timeout=4)  # gen 2
                self.assertLess(time.monotonic() - t0, 5.0)

                time.sleep(0.25)
                source.write_text(original + '\n// gen 3\n')
                wait_for('Published')  # gen 3

                gen_dirs = [p for p in output.iterdir() if p.is_dir()]
                self.assertEqual(len(gen_dirs), 2)
                active = Path((output / 'current').read_text().strip())
                self.assertTrue(active.is_file())
                self.assertIn(active.parent, gen_dirs)
            finally:
                child.terminate()
                child.wait(timeout=5)
                thread.join(timeout=2)
                child.stdout.close()


if __name__ == '__main__':
    unittest.main()
