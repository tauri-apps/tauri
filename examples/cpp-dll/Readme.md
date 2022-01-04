# Readme

This is an example of running tauri within a windows dll library.
This could typically be used for interop with existing C++ / C projects.

  * src-dll1 is an example build of a dll containing code to launch a tauri webview window.
  * src-app1 is a small example of calling the function within the dll.

Note that bundling of resources via tauri.conf.json may not work in some cases due to the nature of the build.
So you have to be aware of copying any files needed to the correct paths for html / js etc.

## Rust libraries

Typically there are 3 types of libraries rust can generate

  * rlib - rust static libraries
  * dylib - rust shared libraries
  * cdylib - used to build shared libraries that can be used by C / C++ (typically a dll under windows)

The dll project uses the cdylib crate type

## Building / Running

``` bash
# First build the dll
cd src-dll1
cargo build
cd ..

# Next build the app
cd src-app1
cargo build

# Next run the app
cargo run
```
