---
tauri-bundler: minor:feat
tauri-utils: minor:feat
tauri-cli: minor:feat
---

Add macOS PKG installer support with custom signing commands.

Implements support for creating macOS PKG installers using pkgbuild and productbuild, with native signing via productsign and support for custom signing commands (useful for HSM-based signing solutions).

Features:
- Create PKG installers from .app bundles using distribution.xml from project root
- Native PKG signing with productsign using signingIdentity or APPLE_CERTIFICATE
- Custom signing command support for .app bundles, .pkg installers, and .dmg disk images
- Custom commands use %1 placeholder for artifact path and run in build directory for relative path support

Configuration fields added to MacOsSettings:
- `appSignCommand`: Custom command for signing .app bundles
- `pkgSignCommand`: Custom command for signing .pkg installers
- `dmgSignCommand`: Custom command for signing .dmg disk images
