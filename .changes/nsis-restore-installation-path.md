---
'tauri-bundler': 'patch:bug'
---

Fix NSIS installer not using the old installation path as a default when using `perMachine` or `currentUser` install modes. Also fixes NSIS not respecting the `/D` flag which used to set the installation directory from command line.
