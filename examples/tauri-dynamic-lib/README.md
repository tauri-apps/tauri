# Readme

This is an example of compiling tauri as a dynamic shared library and running it from another app.

  * src-tauri is an example of a library containing code to launch a tauri webview window.
  * src-app1 is a small example of calling tauri from a dynamic shared library through FFI.

Note that bundling of resources via tauri.conf.json may not work in some cases due to the nature of the build.
So you have to be aware of copying any files needed to the correct paths for html / js etc.

## Rust libraries

Typically there are 3 types of libraries rust can generate

  * rlib - rust static libraries
  * dylib - rust shared libraries
  * cdylib - dynamic shared libraries that can be used to interop with other languages.

Typically cdylib libraries are used for interop with C / C++ Projects but this can also include other languages.
The src-tauri example uses the cdylib crate type

## Building / Running

``` bash
# First build the library
cd src-tauri
cargo build
cd ..

# Next build the app
cd src-app1
cargo build

# Next run the app
cargo run
```
