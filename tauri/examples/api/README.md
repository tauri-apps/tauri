# API example
This example demonstrates Tauri's API capabilities using the `@tauri-apps/api` package. It's used as the main validation app, serving as the testbed of our development process.
In the future, this app will be used on Tauri's integration tests.

![App screenshot](./screenshot.png?raw=true)

## Running the example
- Install dependencies (Run inside of this folder tauri/examples/api/)
```bash
# with yarn
$ yarn
# with npm
$ npm install
```

- Compile tauri
go to root of the tauri repo and run
```
sh .scripts/setup.sh
```

- Compile the app (Run inside of this folder tauri/examples/api/)
```bash
# with yarn
$ yarn tauri:build
# with npm
$ npm run tauri:build
```

- Run the app
```bash
$ ./src-tauri/target/release/app
```
