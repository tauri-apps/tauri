// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! dialoguer is a library for Rust that helps you build useful small
//! interactive user inputs for the command line.  It provides utilities
//! to render various simple dialogs like confirmation prompts, text
//! inputs and more.
//!
//! Best paired with other libraries in the family:
//!
//! * [indicatif](https://docs.rs/indicatif)
//! * [console](https://docs.rs/console)
//!
//! # Crate Contents
//!
//! * Confirmation prompts
//! * Input prompts (regular and password)
//! * Input validation
//! * Selections prompts (single and multi)
//! * Other kind of prompts
//! * Editor launching

pub use edit::Editor;
pub use prompts::{
  confirm::Confirm, input::Input, multi_select::MultiSelect, password::Password, select::Select,
  sort::Sort,
};
pub use validate::Validator;

mod edit;
mod prompts;
pub mod theme;
mod validate;
