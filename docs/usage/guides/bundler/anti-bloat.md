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
    opt-level = "s"

<Alert title="Note">

There is also `opt-level = "z"` available to try to reduce the resulting binary size. `"s"` and `"z"` can sometimes be smaller than the other, so test it with your own application!

We've seen smaller binary sizes from `"s"` for Tauri example applications, but real world applications can always differ.
</Alert>

#### Unstable Rust Compression Features

<Alert type="warning" title="Warning" icon="alert">
The following suggestions are all unstable features and require a nightly toolchain. See the <a href="https://doc.rust-lang.org/cargo/reference/unstable.html#unstable-features">Unstable Features</a> documentation for more information of what this involves.
</Alert>

The following methods involve using unstable compiler features and require having a rust nightly toolchain installed. If you don't have the nightly toolchain + `rust-src` nightly component added, try the following:

    $ rustup toolchain install nightly
    $ rustup component add rust-src --toolchain nightly

The Rust Standard Library comes precompiled. You can instead apply the optimization options used for the rest of your binary + dependencies to the std with an unstable flag. This flag requires specifying your target, so know the target triple that you are targeting.

    $ cargo +nightly build --release -Z build-std --target x86_64-unknown-linux-gnu

If you are using `panic = "abort"` in your release profile optimizations, then you need to make sure the `panic_abort` crate is compiled with std. Additionally, an extra std feature can be used to further reduce the binary size. The following applies both:

    $ cargo +nightly build --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu

See the unstable documentation for more details about [`-Z build-std`](https://doc.rust-lang.org/cargo/reference/unstable.html#build-std) and [`-Z build-std-features`](https://doc.rust-lang.org/cargo/reference/unstable.html#build-std-features).

### Stripping

Binary size can easily be reduced by stripping out debugging information from binaries that ship to end users. This is not good for debuggable builds, but means good binary size savings for end user binaries. The easiest way is to use the famous `strip` utility to remove this debugging information.

    $ strip target/release/my_application

See your local `strip` manpage for more information and flags that can be used to specify what information gets stripped out from the binary.


### UPX

UPX, **Ultimate Packer for eXecutables**, is a dinosaur amongst the binary packers. This 23-year old, well-maintained piece of kit is GPL-v2 licensed with a pretty liberal usage declaration. Our understanding of the licensing is that you can use it for any purposes (commercial or otherwise) without needing to change your license unless you modify the source code of UPX.

Basically it compresses the binary and decompresses it at runtime. It should work for pretty much any binary type out there. Read more: https://github.com/upx/upx

<Alert type="warning" title="Warning" icon="alert">
You should know that this technique might flag your binary as a virus on Windows and macOS - so use at your own discretion, and as always validate with [Frida](https://frida.re/docs/home/) and do real distribution testing!
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
