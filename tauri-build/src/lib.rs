#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

extern crate tauri;

use std::env;
use std::io::Write;

pub mod config;
mod api;
mod salt;
pub mod event;

#[cfg(feature = "embedded-server")]
mod tcp;

mod app;
pub use app::*;
