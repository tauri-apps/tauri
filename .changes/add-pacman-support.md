---
"tauri-bundler": minor:feat
"tauri-cli": minor:feat
"@tauri-apps/cli": minor:feat
"tauri-utils": minor:feat
"@tauri-apps/api": minor:feat
---

Add the `pacman` bundle target for Arch Linux. It produces a `.tar.gz` archive with a `PKGBUILD` and `.SRCINFO` ready to be published to the AUR; see `bundle > linux > pacman` for the package name (`<name>-bin` by default), release, dependencies, install script, changelog and source options. `bundle > licenseFile` is installed to `/usr/share/licenses/<pkgname>/`.

The `pacman` target is part of `"all"`, so Linux builds that bundle all targets now also produce the pacman package; list the targets explicitly in `bundle > targets` to opt out.
