# Sidecar example

This example demonstrates how to use the Tauri sidecar feature. It uses [pkg](https://github.com/vercel/pkg) to compile a Node.js application and bundle it on the Tauri application.

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

- Install dependencies (Run inside of this folder `examples/sidecar/`)
```bash
# with yarn
$ yarn
# with npm
$ npm install

$ yarn tauri
$ yarn package
```

- Run the app in development mode (Run inside of this folder `examples/sidecar/`)
```bash
# with yarn
$ yarn tauri dev
# with npm
$ npm run tauri dev
```

- Build an run the release app (Run inside of this folder `examples/sidecar/`)
```bash
$ yarn tauri build
$ ./src-tauri/target/release/app
```
