---
'tauri-bundler': 'patch'
---

- Fix NSIS installer not using the old installation path as a default when using `perMachine` or `currentUser` install modes.
- NSIS will now respect the `/D` flag which used to set the installation directory from command line.
