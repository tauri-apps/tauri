# The Tauri Architecture
https://tauri.studio
https://github.com/tauri-apps/tauri

## Introduction
Tauri is a polyglot and generic system that is very composable and allows engineers to make a wide variety of applications. It is used for building applications for Desktop Computers using a combination of Rust tools and HTML rendered in a Webview. Apps built with Tauri can ship with any number of pieces of an optional JS API / Rust API so that webviews can control the system via message passing. In fact, developers can extend the default API with their own functionality and bridge the Webview and Rust-based backend easily.

Tauri apps can have custom menus and have tray-type interfaces. They can be updated, and are managed by the user's operating system as expected. They are very small, because they use the system's webview. They do not ship a runtime, since the final binary is compiled from rust. This makes the reversing of Tauri apps not a trivial task.

## Major Components
The following section briefly describes the roles of the various parts of Tauri.
### Tauri Core [STABLE RUST]
#### [tauri](https://github.com/tauri-apps/tauri/tree/dev/core/tauri)
This is the glue crate that holds everything together. It brings the runtimes, macros, utilities and API into one final product. It reads the `tauri.conf.json` file at compile time in order to bring in features and undertake actual configuration of the app (and even the `Cargo.toml` file in the project's folder). It handles script injection (for polyfills / prototype revision) at runtime, hosts the API for systems interaction, and even manages updating.

#### [tauri-build](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-build)
Apply the macros at build-time in order to rig some special features needed by `cargo`.

#### [tauri-codegen](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-codegen)
- Embed, hash, and compress assets, including icons for the app as well as the system-tray.
- Parse `tauri.conf.json` at compile time and generate the Config struct.

#### [tauri-macros](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-macros)
Create macros for the context, handler, and commands by leveraging the `tauri-codegen` crate.

#### [tauri-runtime](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-runtime)
This is the glue layer between tauri itself and lower level webview libraries.

#### [tauri-runtime-wry](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-runtime-wry)
This crate opens up direct systems-level interactions specifically for WRY, such as printing, monitor detection, and other windowing related tasks. `tauri-runtime` implementation for WRY.

#### [tauri-utils](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-utils)
This is common code that is reused in many places and offers useful utilities like parsing configuration files, detecting platform triples, injecting the CSP, and managing assets.


### Tauri Tooling
#### [api](https://github.com/tauri-apps/tauri/tree/dev/tooling/api) [TS -> JS]
A typescript library that creates `cjs` and `esm` Javascript endpoints for you to import into your Frontend framework so that the Webview can call and listen to backend activity. We also ship the pure typescript, because for some frameworks this is more optimal. It uses the message passing of webviews to their hosts.

#### [bundler](https://github.com/tauri-apps/tauri/tree/dev/tooling/bundler) [RUST / SHELL]
The bundler is a library that builds a Tauri App for the platform triple it detects / is told. At the moment it currently supports macOS, Windows and Linux - but in the near future will support mobile platforms as well. May be used outside of Tauri projects.

#### [cli.js](https://github.com/tauri-apps/tauri/tree/dev/tooling/cli.js) [JS]
Written in Typescript and packaged such that it can be used with `npm`, `pnpm`, and `yarn`, this library provides a node.js runner for common tasks when using Tauri, like `yarn tauri dev`. For the most part it is a wrapper around [cli.rs](https://github.com/tauri-apps/tauri/blob/dev/tooling/cli.rs).

#### [cli.rs](https://github.com/tauri-apps/tauri/tree/dev/tooling/cli.rs) [RUST]
This rust executable provides the full interface to all of the required activities for which the CLI is required. It will run on macOS, Windows, and Linux.

#### [create-tauri-app](https://github.com/tauri-apps/tauri/tree/dev/tooling/create-tauri-app) [JS]
This is a toolkit that will enable engineering teams to rapidly scaffold out a new tauri-apps project using the frontend framework of their choice (as long as it has been configured).

## RUST API
- **app**	The App API module allows you to manage application processes.
- **assets**	The Assets module allows you to read files that have been bundled by tauri Assets handled by Tauri during compile time and runtime.
- **config**	The Tauri config definition.
- **dialog**	The Dialog API module allows you to show messages and prompt for file paths.
- **dir**	The Dir module is a helper for file system directory management.
- **file**	The File API module contains helpers to perform file operations.
- **http**	The HTTP request API.
- **notification**	The desktop notifications API module.
- **path**	The file system path operations API.
- **process**	Process helpers including the management of child processes.
- **rpc**	The RPC module includes utilities to send messages to the JS layer of the webview.
- **shell**	The shell api.
- **shortcuts**	Global shortcuts interface.
- **version**	The semver API.



# External Crates
The Tauri-Apps organisation maintains two "upstream" crates from Tauri, namely TAO for creating and managing application windows, and WRY for interfacing with the Webview that lives within the window.

## [TAO](https://github.com/tauri-apps/tao)
Cross-platform application window creation library in Rust that supports all major platforms like Windows, macOS, Linux, iOS and Android. Written in Rust, it is a fork of [winit](https://github.com/rust-windowing/winit) that we have extended for our own needs like menu bar and system tray.


## [WRY](https://github.com/tauri-apps/wry)
WRY is a cross-platform WebView rendering library in Rust that supports all major desktop platforms like Windows, macOS, and Linux.
Tauri uses WRY as the abstract layer responsible to determine which webview is used (and how interactions are made).

## [tauri-hotkey-rs](https://github.com/tauri-apps/tauri-hotkey-rs)
We needed to fix hotkey to work on all platforms, because upstream was not being responsive.

# Additional tooling

## [binary-releases](https://github.com/tauri-apps/binary-releases)
This is the delivery mechanism for tauri prebuilt binaries: currently the cli.rs (used by cli.js) and rustup binaries (used by the deps install command of cli.js). These artifacts are automatically created on release.

## [tauri-action](https://github.com/tauri-apps/tauri-action)
This is a github workflow that builds tauri binaries for all platforms. It is not the fastest out there, but it gets the job done and is highly configurable. Even allowing you to create a (very basic) tauri app even if tauri is not setup.

## [create-pull-request](https://github.com/tauri-apps/create-pull-request)
Because this is a very risky (potentially destructive) github action, we forked it in order to have strong guarantees that the code we think is running is actually the code that is running.

## [vue-cli-plugin-tauri](https://github.com/tauri-apps/vue-cli-plugin-tauri)
This plugin allows you to very quickly install tauri in a vue-cli project.

## [tauri-vscode](https://github.com/tauri-apps/tauri-vscode)
This project enhances the VS Code interface with several nice-to-have features.

# Tauri Plugins [documentation](https://tauri.studio/en/docs/usage/guides/plugin)

Generally speaking, plugins are authored by third parties (even though there may be official, supported plugins). A plugin generally does 3 things:
1. It provides rust code to do "something".
2. It provides interface glue to make it easy to integrate into an app.
3. It provides a JS API for interfacing with the rust code.

Here are several examples of Tauri Plugins:
- https://github.com/tauri-apps/tauri-plugin-sql
- https://github.com/tauri-apps/tauri-plugin-stronghold
- https://github.com/tauri-apps/tauri-plugin-authenticator

# Workflows
## What does the Development flow look like?
A developer must first install the prerequisite toolchains for creating a Tauri app. At the very least this will entail installing rust & cargo, and most likely also a modern version of node.js and potentially another package manager. Some platforms may also require other tooling and libraries, but this has been documented carefully in the respective platform docs.

Because of the many ways to build front-ends, we will stick with a common node.js based approach for development. (Note: Tauri does not by default ship a node.js runtime.)

The easiest way to do this is to run the following:
```
npx create-tauri-app
```

Which will ask you a bunch of questions about the framework you want to install and then create everything you need in a single folder - some via the placement of template files and some through normal installation procedures of your framework.

> If you don't use this process, you will have to manually install the tauri cli, initialise tauri and manually configure the `tauri.conf.json` file.

Once everything is installed, you can run:
```
yarn tauri dev
-or-
npm run tauri dev
```
This will do several things:
1. start the JS Framework devserver
2. begin the long process of downloading and compiling the rust libraries
3. open an application window with devtools enabled
4. keep a long-lived console alive 

If you change your HTML/CSS/TS/JS, your framework devserver should give you its best shot at instant hot module reloading and you will see the changes instantly.

If you modify your rust code or anything in the Cargo.toml, the window will close while rust recompiles. When finished it will reload.

If you need to get deeper insight into your current project, or triage requires investigation of installed components, just run:

```
yarn tauri info
```


## What does the Release flow look like?

The release flow begins with proper configuration in the `tauri.conf.json` file. In this file, the developer can configure not only the basic behaviour of the application (like window size and decoration), they can also provide settings for signing and updating.

Depending upon the operating system that the developer (or CI) is building the application on, there will be an app built for them for that system. (Cross compilation is not currently available, however there is an official [GitHub Action](https://github.com/tauri-apps/tauri-action) that can be used to build for all platforms.)

To kick off this process, just:
```
yarn tauri build
```

After some time, the process will end and you can see the results in the `./src-tauri/target/release` folder.

## What does the End-User flow look like?
End users will be provided with binaries in ways that are appropriate for their systems. Whether macOS, Linux, or Windows, direct download or store installations - they will be able to follow procedures for installing and removing that they are used to.

## What does the Updating flow look like?
When a new version is ready, the developer publishes the new signed artifacts to a server (that they have configured within `tauri.conf.json`). 

The application can poll this server to see if there is a new release. When there is a new release, the user is prompted to update. The application update is downloaded, verified (checksum & signature), updated, closed, and restarted.

## License
Tauri itself is licensed under MIT or Apache-2.0. If you repackage it and modify any source code, it is your responsibility to verify that you are complying with all upstream licenses. Tauri is provided AS-IS with no explicit claim for suitability for any purpose.

Here you may peruse our [Software Bill of Materials](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri).
