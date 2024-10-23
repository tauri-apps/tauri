# API example

This example demonstrates Tauri's API capabilities using the `@tauri-apps/api` package. It's used as the main validation app, serving as the testbed of our development process.
In the future, this app will be used on Tauri's integration tests.

![App screenshot](./screenshot.png?raw=true)

## Running the example

- Compile Tauri
  go to root of the Tauri repo and run:
  Linux / Mac:

```
# choose to install node cli (1)
bash .scripts/setup.sh
```

Windows:

```
./.scripts/setup.ps1
```

- Install dependencies (Run inside of this folder `examples/api/`)

```bash
$ pnpm i
```

- Run the app in development mode (Run inside of this folder `examples/api/`)

```bash
$ pnpm tauri dev
```

- Build an run the release app (Run inside of this folder `examples/api/`)

```bash
$ pnpm tauri build
$ ./src-tauri/target/release/app
```
