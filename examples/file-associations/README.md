# File Associations Example

This example demonstrates how to make associations between the application and certain file types.

This feature is commonly used for functionality such as previewing or editing files.

## Running the example

1. Run the following inside `examples/file-associations/src-tauri`

   ```
   cargo build --features tauri/protocol-asset
   ```

## Associations

This example creates associations with PNG, JPG, JPEG and GIF files.

Additionally, it defines two new extensions - `taurid` (derives from a raw data file) and `taurijson` (derives from JSON). They have special treatment on macOS (see `exportedType` in `src-tauri/tauri.conf.json`).
