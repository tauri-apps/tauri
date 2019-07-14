# web-view &emsp; [![Build Status]][travis] [![Latest Version]][crates.io]

[Build Status]: https://api.travis-ci.org/Boscop/web-view.svg?branch=master
[travis]: https://travis-ci.org/Boscop/web-view
[Latest Version]: https://img.shields.io/crates/v/web-view.svg
[crates.io]: https://crates.io/crates/web-view

This library provides Rust bindings for the [webview](https://github.com/zserge/webview) library to allow easy creation of cross-platform Rust desktop apps with GUIs based on web technologies.

It supports two-way bindings for communication between the Rust backend and JavaScript frontend.

It uses Cocoa/WebKit on macOS, gtk-webkit2 on Linux and MSHTML (IE10/11) on Windows, so your app will be **much** leaner than with Electron.

For usage info please check out [the examples](../../tree/master/examples) and the [original readme](https://github.com/zserge/webview/blob/master/README.md).

Contributions and feedback welcome :)

<p align="center"><img alt="screenshot" src="https://i.imgur.com/Z3c2zwD.png"></p>

## Notes:
Requires Rust 1.30 stable or newer.

---

## Suggestions:
- If you like type safety, write your frontend in [Elm](http://elm-lang.org/) or [PureScript](http://www.purescript.org/)<sup>[*](#n1)</sup>, or use a Rust frontend framework that compiles to asm.js, like [yew](https://github.com/DenisKolodin/yew).
- Use [parcel](https://parceljs.org/) to bundle & minify your frontend code.
- Use [inline-assets](https://www.npmjs.com/package/inline-assets) to inline all your assets (css, js, html) into one index.html file and embed it in your Rust app using `include_str!()`.
- If your app runs on windows, [add an icon](https://github.com/mxre/winres) to your Rust executable to make it look more professional™
- Use custom npm scripts or [just](https://github.com/casey/just) or [cargo-make](https://github.com/sagiegurari/cargo-make) to automate the build steps.
- Make your app state persistent between sessions using localStorage in the frontend or [rustbreak](https://crates.io/crates/rustbreak) in the backend.
- Btw, instead of injecting app resources via the js api, you can also serve them from a local http server (e.g. bound to an ephemeral port).
- Happy coding :)

<a name="n1">*</a> The free [PureScript By Example](https://leanpub.com/purescript/read) book contains several practical projects for PureScript beginners.

---

## Contribution opportunities:
- Create an issue for any question you have
- Docs
- Feedback on this library's API and code
- Test it on non-windows platforms, report any issues you find
- Showcase your app
- Add an example that uses Elm or Rust compiled to asm.js
- Add a PureScript example that does two-way communication with the backend
- Contribute to the original webview library: E.g. [add HDPI support on Windows](https://github.com/zserge/webview/issues/54)
- Make it possible to create the webview window as a child window of a given parent window. This would allow webview to be used for the GUIs of [VST audio plugins in Rust](https://github.com/rust-dsp/rust-vst).

---

### Ideas for apps:
- Rust IDE (by porting [xi-electron](https://github.com/acheronfail/xi-electron) to web-view)
- Data visualization / plotting lib for Rust, to make Rust more useful for data science
- Crypto coin wallet
- IRC client, or client for other chat protocols
- Midi song editor, VJ controller
- Rust project template wizard: Generate new Rust projects from templates with user-friendly steps
- GUI for [pijul](https://pijul.org/)

## Showcase
*Feel free to open a PR if you want your app to be listed here!*  

- [Juggernaut](https://github.com/ShashankaNataraj/Juggernaut) - The unstoppable programmers editor
- [FrakeGPS](https://github.com/frafra/frakegps) - Simulate a simple GPS device
