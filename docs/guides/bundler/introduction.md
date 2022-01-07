---
title: Introduction
---

The Tauri Bundler is a Rust harness for compiling your binary, packaging assets, and preparing a final bundle.

It will detect your operating system and build a bundle accordingly. It currently supports:

- Linux: .deb, .appimage
- macOS: .app, .dmg
- Windows: .exe, .msi