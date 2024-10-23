// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// we move some basic commands to a separate module just to show it works
mod commands;
use commands::{cmd, invoke, message, resolver};

use serde::Deserialize;
use tauri::{
  command,
  ipc::{Request, Response},
  State, Window,
};

#[derive(Debug)]
pub struct MyState {
  #[allow(dead_code)]
  value: u64,
  #[allow(dead_code)]
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
async fn async_simple_command(the_argument: String) {
  println!("{the_argument}");
}

#[command(rename_all = "snake_case")]
async fn async_simple_command_snake(the_argument: String) {
  println!("{the_argument}");
}

#[command]
async fn async_stateful_command(
  the_argument: Option<String>,
  state: State<'_, MyState>,
) -> Result<(), ()> {
  println!("{:?} {:?}", the_argument, state.inner());
  Ok(())
}
// ------------------------ Raw future commands ------------------------

#[command(async)]
fn future_simple_command(the_argument: String) -> impl std::future::Future<Output = ()> {
  println!("{the_argument}");
  std::future::ready(())
}

#[command(async)]
fn future_simple_command_with_return(
  the_argument: String,
) -> impl std::future::Future<Output = String> {
  println!("{the_argument}");
  std::future::ready(the_argument)
}

#[command(async)]
fn future_simple_command_with_result(
  the_argument: String,
) -> impl std::future::Future<Output = Result<String, ()>> {
  println!("{the_argument}");
  std::future::ready(Ok(the_argument))
}

#[command(async)]
fn force_async(the_argument: String) -> String {
  the_argument
}

#[command(async)]
fn force_async_with_result(the_argument: &str) -> Result<&str, MyError> {
  (!the_argument.is_empty())
    .then_some(the_argument)
    .ok_or(MyError::FooError)
}

// ------------------------ Raw future commands - snake_case ------------------------

#[command(async, rename_all = "snake_case")]
fn future_simple_command_snake(the_argument: String) -> impl std::future::Future<Output = ()> {
  println!("{the_argument}");
  std::future::ready(())
}

#[command(async, rename_all = "snake_case")]
fn future_simple_command_with_return_snake(
  the_argument: String,
) -> impl std::future::Future<Output = String> {
  println!("{the_argument}");
  std::future::ready(the_argument)
}

#[command(async, rename_all = "snake_case")]
fn future_simple_command_with_result_snake(
  the_argument: String,
) -> impl std::future::Future<Output = Result<String, ()>> {
  println!("{the_argument}");
  std::future::ready(Ok(the_argument))
}

#[command(async, rename_all = "snake_case")]
fn force_async_snake(the_argument: String) -> String {
  the_argument
}

#[command(rename_all = "snake_case", async)]
fn force_async_with_result_snake(the_argument: &str) -> Result<&str, MyError> {
  (!the_argument.is_empty())
    .then_some(the_argument)
    .ok_or(MyError::FooError)
}

// ------------------------ Commands returning Result ------------------------

#[command]
fn simple_command_with_result(the_argument: String) -> Result<String, MyError> {
  println!("{the_argument}");
  (!the_argument.is_empty())
    .then_some(the_argument)
    .ok_or(MyError::FooError)
}

#[command]
fn stateful_command_with_result(
  the_argument: Option<String>,
  state: State<'_, MyState>,
) -> Result<String, MyError> {
  println!("{:?} {:?}", the_argument, state.inner());
  dbg!(the_argument.ok_or(MyError::FooError))
}

// ------------------------ Commands returning Result - snake_case ------------------------

#[command(rename_all = "snake_case")]
fn simple_command_with_result_snake(the_argument: String) -> Result<String, MyError> {
  println!("{the_argument}");
  (!the_argument.is_empty())
    .then_some(the_argument)
    .ok_or(MyError::FooError)
}

#[command(rename_all = "snake_case")]
fn stateful_command_with_result_snake(
  the_argument: Option<String>,
  state: State<'_, MyState>,
) -> Result<String, MyError> {
  println!("{:?} {:?}", the_argument, state.inner());
  dbg!(the_argument.ok_or(MyError::FooError))
}

// Async commands

#[command]
async fn async_simple_command_with_result(the_argument: String) -> Result<String, MyError> {
  println!("{the_argument}");
  Ok(the_argument)
}

#[command]
async fn async_stateful_command_with_result(
  the_argument: Option<String>,
  state: State<'_, MyState>,
) -> Result<String, MyError> {
  println!("{:?} {:?}", the_argument, state.inner());
  Ok(the_argument.unwrap_or_default())
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
fn command_arguments_struct(Person { name, age }: Person<'_>) {
  println!("received person struct with name: {name} | age: {age}")
}

#[derive(Deserialize)]
struct InlinePerson<'a>(&'a str, u8);

#[command]
fn command_arguments_tuple_struct(InlinePerson(name, age): InlinePerson<'_>) {
  println!("received person tuple with name: {name} | age: {age}")
}

#[command]
fn borrow_cmd(the_argument: &str) -> &str {
  the_argument
}

#[command]
fn borrow_cmd_async(the_argument: &str) -> &str {
  the_argument
}

#[command]
fn raw_request(request: Request<'_>) -> Response {
  println!("{request:?}");
  Response::new(include_bytes!("./README.md").to_vec())
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
      raw_request,
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
      async_simple_command_snake,
      future_simple_command_snake,
      future_simple_command_with_return_snake,
      future_simple_command_with_result_snake,
      force_async_snake,
      force_async_with_result_snake,
      simple_command_with_result_snake,
      stateful_command_with_result_snake,
      stateful_command_with_result,
      command_arguments_tuple_struct,
      async_simple_command_with_result,
      future_simple_command_with_return,
      future_simple_command_with_result,
      async_stateful_command_with_result,
    ])
    .run(tauri::generate_context!(
      "../../examples/commands/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
