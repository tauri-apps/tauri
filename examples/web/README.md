# Desktop / Web Example

This example showcases an application that has shares code between a desktop and a Web target.

The Web application uses WASM to communicate with the Rust backend, while the desktop app leverages Tauri commands.

## Architecture

The Rust code lives in the `core/` folder and it is a Cargo workspace with three crates:

- tauri: desktop application. Contains the commands that are used by the frontend to access the Rust APIs;
- wasm: library that is compiled to WASM to be used by the Web application;
- api: code shared between the Tauri and the WASM crates. Most of the logic should live here, with only the specifics in the tauri and wasm crates.

The Rust code bridge is defined in the `src/api/` folder, which defines `desktop/index.js` and a `web/index.js` interfaces.
To access the proper interface according to the build target, a resolve alias is defined in vite.config.js, so the API can be imported
with `import * as api from '$api'`.

## Running the desktop application

Use the following commands to run the desktop application:

```
yarn
yarn tauri dev
```

## Running the Web application

Use the following commands to run the Web application:

```
yarn
yarn dev:web
```
