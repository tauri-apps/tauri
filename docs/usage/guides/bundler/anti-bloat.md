---
title: Anti Bloat
---

import Alert from '@theme/Alert'

The following links have tutorials on reducing the size of your installers:

- https://github.com/RazrFalcon/cargo-bloat
- https://lifthrasiir.github.io/rustlog/why-is-a-rust-executable-large.html
- https://doc.rust-lang.org/cargo/reference/manifest.html#the-profile-sections

### Rust Compression Features

Add this to your `src-tauri/Cargo.toml`

    [profile.release]
    panic = "abort"
    codegen-units = 1
    lto = true
    incremental = false
    opt-level = "z"

### UPX

UPX, **Ultimate Packer for eXecutables**, is a dinosaur amongst the binary packers. This 23-year old, well-maintained piece of kit is GPL-v2 licensed with a pretty liberal usage declaration. Our understanding of the licensing is that you can use it for any purposes (commercial or otherwise) without needing to change your license unless you modify the source code of UPX.

Basically it compresses the binary and decompresses it at runtime. It should work for pretty much any binary type out there. Read more: https://github.com/upx/upx

<Alert type="warning" title="Warning" icon="alert">
You should know that this technique might flag your binary as a virus on Windows and macOS - so use at your own discretion, and as always validate with Frida and do real distribution testing!
</Alert>

#### Usage on macOS

    $ brew install upx
    $ yarn tauri build
    $ upx --ultra-brute src-tauri/target/release/bundle/macos/app.app/Contents/macOS/app
                           Ultimate Packer for eXecutables
                              Copyright (C) 1996 - 2018
    UPX 3.95        Markus Oberhumer, Laszlo Molnar & John Reiser   Aug 26th 2018

            File size         Ratio      Format      Name
       --------------------   ------   -----------   -----------
        963140 ->    274448   28.50%   macho/amd64   app
