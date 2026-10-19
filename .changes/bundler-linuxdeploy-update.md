---
'tauri-bundler': 'patch:enhance'
---

Update linuxdeploy used to build AppImage bundles to [`07333c6`](https://github.com/linuxdeploy/linuxdeploy/commit/07333c6). The bundler now verifies the downloaded binary and caches it under a versioned file name (`linuxdeploy-07333c6-<arch>.AppImage`), so it is downloaded again even if an older version is cached.

- Its updated built-in excludelist no longer bundles `libwayland-client.so.0` and `libpipewire-0.3.so.0`; they are loaded from the host system instead.
- If you use `TAURI_BUNDLER_TOOLS_GITHUB_MIRROR` or `TAURI_BUNDLER_TOOLS_GITHUB_MIRROR_TEMPLATE` with a self-hosted mirror, mirror the `linuxdeploy-07333c6` release of `tauri-apps/binary-releases`.
- If your offline builds pre-populate the tools cache, provide the binary under its new file name.
