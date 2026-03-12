## Description

When new files are added to resource directories configured in `tauri.conf.json`, Cargo doesn't know to rerun the build script because it only tracks changes to files that were already known during the previous build.

This fix emits `cargo:rerun-if-changed` for each resource directory path, ensuring that when new files are added to resource directories, the build script will be re-executed and the new files will be copied.

## Fixes

Fixes #14992

## Testing

1. Create a new Tauri project with resource directories configured
2. Add new files to the resource directory after initial build
3. Run `cargo build` or `tauri dev` - the new files should now be picked up

## Checklist

- [x] I have read the Contribution Guide
- [x] I have tested the changes
- [x] My code follows the code style of this project
