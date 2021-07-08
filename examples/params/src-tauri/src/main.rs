// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
#![allow(
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
    clippy::nonstandard_macro_braces,
)]

use serde::Serialize;
use std::fmt;
use std::str::FromStr;
use tauri::{api::assets::EmbeddedAssets, command, Params, Wry};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Event {
  Foo,
  Bar,
  Unknown(String),
}

impl fmt::Display for Event {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      Self::Foo => "foo",
      Self::Bar => "bar",
      Self::Unknown(s) => s,
    })
  }
}

impl FromStr for Event {
  type Err = std::convert::Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "foo" => Self::Foo,
      "bar" => Self::Bar,
      other => Self::Unknown(other.to_string()),
    })
  }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Window {
  Main,
}

impl fmt::Display for Window {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      Self::Main => "main",
    })
  }
}

impl FromStr for Window {
  type Err = Box<dyn std::error::Error>;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s == "main" {
      Ok(Self::Main)
    } else {
      Err(format!("only expect main window label, found: {}", s).into())
    }
  }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
pub enum Menu {
  MenuFoo,
  MenuBar,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
pub enum SystemMenu {
  SystemFoo,
  SystemBar,
}

#[command]
fn log_window_label(window: tauri::Window<CustomArgs>) {
  dbg!(window.label());
}

#[command]
fn send_foo(window: tauri::Window<CustomArgs>) {
  window
    .emit(&Event::Foo, ())
    .expect("couldn't send Event::Foo");
}

pub struct CustomArgs {}

impl Params for CustomArgs {
  type Event = Event;
  type Label = Window;
  type MenuId = Menu;
  type SystemTrayMenuId = SystemMenu;
  type Assets = EmbeddedAssets;
  type Runtime = Wry;
}

fn main() {
  tauri::Builder::<CustomArgs>::new()
    .invoke_handler(tauri::generate_handler![log_window_label, send_foo])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
