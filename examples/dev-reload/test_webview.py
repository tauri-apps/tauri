"""Headless real-WebView reload acceptance test (Linux, xvfb-run required)."""
import os
from pathlib import Path
import queue
import signal
import shutil
import subprocess
import sys
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[2]


def main():
    with tempfile.TemporaryDirectory(prefix='tauri-reload-webview-') as directory:
        root = Path(directory)
        shutil.copytree(Path(__file__).parent / 'backend', root / 'backend',
                        ignore=shutil.ignore_patterns('target', '.reload'))
        env = dict(os.environ, CARGO_NET_OFFLINE='true', TAURI_RELOAD_SMOKE='1',
                   TAURI_RELOAD_MANIFEST=str(root / 'generations/current'), GDK_BACKEND='x11')
        commands = [
            [sys.executable, str(ROOT / 'scripts/dev-reload.py'), '--manifest',
             str(root / 'backend/Cargo.toml'), '--bin', 'reload-counter', '--output', str(root / 'generations')],
            ['xvfb-run', '-a', str(ROOT / 'target/debug/examples/dev-reload')],
        ]
        processes = []
        threads = []
        messages = queue.Queue()
        def read(child):
            for line in child.stdout:
                messages.put(line)
        def start(command):
            child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                     text=True, env=env, start_new_session=True)
            processes.append(child)
            thread = threading.Thread(target=read, args=(child,), daemon=True)
            thread.start()
            threads.append(thread)
        def wait_for(message):
            deadline = time.monotonic() + 45
            while time.monotonic() < deadline:
                try:
                    line = messages.get(timeout=0.2)
                except queue.Empty:
                    if any(child.poll() is not None for child in processes):
                        raise RuntimeError('A worker watcher or WebView host exited before ' + message)
                    continue
                print(line, end='', flush=True)
                if message in line:
                    return
                if 'RELOAD_SMOKE:failed' in line:
                    raise RuntimeError('WebView state was lost')
            raise TimeoutError('Missing: ' + message)
        try:
            start(commands[0])
            wait_for('Published')
            start(commands[1])
            wait_for('RELOAD_SMOKE:ready')
            source = root / 'backend/src/main.rs'
            source.write_text(source.read_text().replace('Rust generation one', 'Rust generation two'))
            wait_for('RELOAD_SMOKE:passed')
            assert processes[1].wait(timeout=10) == 0
            print('PASS: real WebView retained document, input and Rust counter across rebuild.')
        finally:
            for child in reversed(processes):
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGTERM)
                    child.wait(timeout=5)
            for thread in threads:
                thread.join(timeout=2)
            for child in processes:
                child.stdout.close()


if __name__ == '__main__':
    main()
