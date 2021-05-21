# Splashscreen example

This example demonstrates how a splashscreen can be implemented when waiting on an initialization code on Rust or on the UI.

## Running the example

Run the following scripts on the root directory of the repository:

```bash
# runs the example that wait on a Rust initialization script to show the main window
$ cargo run --bin splashscreen
# runs the example that wait on the UI to load to show the main window
$ cargo run --bin splashscreen --features ui
```
