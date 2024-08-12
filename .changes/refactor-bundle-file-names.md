---
'tauri-bundler': 'patch:breaking'
'tauri-cli': 'patch:breaking'
'@tauri-apps/cli': 'patch:breaking'
---

Change bundle file names to a consistent `productName-version-arch.ext` format.
  - AppImage
    - productName_version_i386.AppImage => productName-version-x86.AppImage
    - productName_version_amd64.AppImage => productName-version-x86_64.AppImage
  - Debian
    - productName_version_i386.deb => productName-version-x86.deb
    - productName_version_amd64.deb => productName-version-x86_64.deb
    - productName_version_armhf.deb => productName-version-arm.deb
    - productName_version.arm64.deb => productName-version-aarch64.deb
  - RPM
    - productName-version-$release.arch.rpm => productName-version-aarch64-$release.rpm
  - DMG
    - productName_version_x64.dmg => productName-version-x86_64.dmg
    - productName_version_aarch64.dmg => productName-version-aarch64.dmg
  - macOS 
    - still keeps the productName.app format as that is the name of the file that is actually installed on user machines.
  - MSI
    - productName_version_x86_$language.msi => productName-version-x86-$language.msi
    - productName_version_x64_$language.msi => productName-version-x86_64-$language.msi
    - productName_version_arm64_$language.msi => productName-version-aarch64-$language.msi
  - NSIS
    - productName_version_x86_setup.exe => productName-version-x86-setup.exe
    - productName_version_x64_setup.exe => productName-version-x86_64-setup.exe
    - productName_version_arm64_setup.exe => productName-version-aarch64-setup.exe
