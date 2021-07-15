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

// we move some basic commands to a separate module just to show it works
mod commands;
use commands::{cmd, invoke, message, resolver};

use serde::Deserialize;
use tauri::{command, State, Window};

#[derive(Debug)]
pub struct MyState {
  value: u64,
  label: String,
}

#[derive(Debug, serde::Serialize)]
enum MyError {
  FooError,
}

// ------------------------ Commands using Window ------------------------
#[command]
fn window_label(window: Window) {
  println!("window label: {}", window.label());
}

// Async commands

#[command]
async fn async_simple_command(argument: String) {
  println!("{}", argument);
}

#[command]
async fn async_stateful_command(
  argument: Option<String>,
  state: State<'_, MyState>,
) -> Result<(), ()> {
  println!("{:?} {:?}", argument, state.inner());
  Ok(())
}

// Raw future commands
#[command(async)]
fn future_simple_command(argument: String) -> impl std::future::Future<Output = ()> {
  println!("{}", argument);
  std::future::ready(())
}

#[command(async)]
fn future_simple_command_with_return(
  argument: String,
) -> impl std::future::Future<Output = String> {
  println!("{}", argument);
  std::future::ready(argument)
}

#[command(async)]
fn future_simple_command_with_result(
  argument: String,
) -> impl std::future::Future<Output = Result<String, ()>> {
  println!("{}", argument);
  std::future::ready(Ok(argument))
}

#[command(async)]
fn force_async(argument: String) -> String {
  argument
}

#[command(async)]
fn force_async_with_result(argument: &str) -> Result<&str, MyError> {
  (!argument.is_empty())
    .then(|| argument)
    .ok_or(MyError::FooError)
}

// ------------------------ Commands returning Result ------------------------

#[command]
fn simple_command_with_result(argument: String) -> Result<String, MyError> {
  println!("{}", argument);
  (!argument.is_empty())
    .then(|| argument)
    .ok_or(MyError::FooError)
}

#[command]
fn stateful_command_with_result(
  argument: Option<String>,
  state: State<'_, MyState>,
) -> Result<String, MyError> {
  println!("{:?} {:?}", argument, state.inner());
  dbg!(argument.ok_or(MyError::FooError))
}

// Async commands

#[command]
async fn async_simple_command_with_result(argument: String) -> Result<String, MyError> {
  println!("{}", argument);
  Ok(argument)
}

#[command]
async fn async_stateful_command_with_result(
  argument: Option<String>,
  state: State<'_, MyState>,
) -> Result<String, MyError> {
  println!("{:?} {:?}", argument, state.inner());
  Ok(argument.unwrap_or_else(|| "".to_string()))
}

// Non-Ident command function arguments

#[command]
fn command_arguments_wild(_: Window) {
  println!("we saw the wildcard!")
}

#[derive(Deserialize)]
struct Person<'a> {
  name: &'a str,
  age: u8,
}

#[command]
fn command_arguments_struct(Person { name, age }: Person) {
  println!("received person struct with name: {} | age: {}", name, age)
}

#[derive(Deserialize)]
struct InlinePerson<'a>(&'a str, u8);

#[command]
fn command_arguments_tuple_struct(InlinePerson(name, age): InlinePerson) {
  println!("received person tuple with name: {} | age: {}", name, age)
}

#[command]
fn borrow_cmd(argument: &str) -> &str {
  argument
}

#[command]
fn borrow_cmd_async(argument: &str) -> &str {
  argument
}

fn main() {
  tauri::Builder::default()
    .manage(MyState {
      value: 0,
      label: "Tauri!".into(),
    })
    .invoke_handler(tauri::generate_handler![
      borrow_cmd,
      borrow_cmd_async,
      window_label,
      force_async,
      force_async_with_result,
      commands::simple_command,
      commands::stateful_command,
      cmd,
      invoke,
      message,
      resolver,
      async_simple_command,
      future_simple_command,
      async_stateful_command,
      command_arguments_wild,
      command_arguments_struct,
      simple_command_with_result,
      stateful_command_with_result,
      command_arguments_tuple_struct,
      async_simple_command_with_result,
      future_simple_command_with_return,
      future_simple_command_with_result,
      async_stateful_command_with_result,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
