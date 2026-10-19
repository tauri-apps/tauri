# Resource example

This example demonstrates the Tauri bundle resources functionality. The example adds `src-tauri/assets/index.js` as a resource (defined on `tauri.conf.json > bundle > resources`) and executes it using `Node.js`, locating the JavaScript file using the `tauri::App::path_resolver` APIs.

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

- Install dependencies (Run inside of this folder `examples/resources/`)

```bash
$ pnpm i
```

- Run the app in development mode (Run inside of this folder `examples/resources/`)

```bash
$ pnpm tauri dev
```

- Build an run the release app (Run inside of this folder `examples/resources/`)

```bash
$ pnpm tauri build
$ ./src-tauri/target/release/app
```
