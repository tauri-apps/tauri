# Updater Example

This example showcases the App Updater feature.

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

- Run the app in development mode (Run inside of this folder `examples/updater/`)
```bash
$ cargo tauri dev
```

- Build an run the release app (Run inside of this folder `examples/updater/`)
```bash
$ cargo tauri build
$ ./src-tauri/target/release/app
```
