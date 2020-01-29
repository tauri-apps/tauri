#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod command;
pub mod dir;
pub mod file;
pub mod rpc;
pub mod version;

pub use tauri_utils::*;

use error_chain::error_chain;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        ZipError(::zip::result::ZipError);
        SemVer(::semver::SemVerError);
        Platform(::tauri_utils::Error);
    }
    errors {
        Extract(t: String) {
            description("Extract Error")
            display("Extract Error: '{}'", t)
        }
        Command(t: String) {
            description("Command Execution Error")
            display("Command Error: '{}'", t)
        }
        File(t: String) {
            description("File function Error")
            display("File Error: {}", t)
        }
    }
}
