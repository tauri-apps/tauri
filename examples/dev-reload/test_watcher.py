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


class WatcherTest(unittest.TestCase):
    def test_publish_failure_and_recovery(self):
        with tempfile.TemporaryDirectory(prefix='tauri-reload-') as directory:
            root = Path(directory)
            shutil.copytree(Path(__file__).parent / 'backend', root / 'backend',
                            ignore=shutil.ignore_patterns('target', '.reload'))
            output = root / 'generations'
            env = dict(os.environ, CARGO_NET_OFFLINE='true')
            child = subprocess.Popen([
                sys.executable, str(ROOT / 'scripts/dev-reload.py'),
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


if __name__ == '__main__':
    unittest.main()
