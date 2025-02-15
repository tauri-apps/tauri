// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Create macros for `tauri::Context`, invoke handler and commands leveraging the `tauri-codegen` crate.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]

use std::path::PathBuf;

use crate::context::ContextItems;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse2, parse_macro_input, LitStr};
use tauri_codegen::image::CachedIcon;

mod command;
mod menu;
mod mobile;
mod runtime;

#[macro_use]
mod context;

/// Mark a function as a command handler. It creates a wrapper function with the necessary glue code.
///
/// # Stability
/// The output of this macro is managed internally by Tauri,
/// and should not be accessed directly on normal applications.
/// It may have breaking changes in the future.
#[proc_macro_attribute]
pub fn command(attributes: TokenStream, item: TokenStream) -> TokenStream {
  command::wrapper(attributes, item)
}

#[proc_macro_attribute]
pub fn mobile_entry_point(attributes: TokenStream, item: TokenStream) -> TokenStream {
  mobile::entry_point(attributes, item)
}

/// Accepts a list of command functions. Creates a handler that allows commands to be called from JS with invoke().
///
/// # Examples
/// ```rust,ignore
/// use tauri_macros::{command, generate_handler};
/// #[command]
/// fn command_one() {
///   println!("command one called");
/// }
/// #[command]
/// fn command_two() {
///   println!("command two called");
/// }
/// fn main() {
///   let _handler = generate_handler![command_one, command_two];
/// }
/// ```
/// # Stability
/// The output of this macro is managed internally by Tauri,
/// and should not be accessed directly on normal applications.
/// It may have breaking changes in the future.
#[proc_macro]
pub fn generate_handler(item: TokenStream) -> TokenStream {
  parse_macro_input!(item as command::Handler).into()
}

/// Reads a Tauri config file and generates a `::tauri::Context` based on the content.
///
/// # Stability
/// The output of this macro is managed internally by Tauri,
/// and should not be accessed directly on normal applications.
/// It may have breaking changes in the future.
#[proc_macro]
pub fn generate_context(items: TokenStream) -> TokenStream {
  // this macro is exported from the context module
  let path = parse_macro_input!(items as ContextItems);
  context::generate_context(path).into()
}

/// Adds the default type for the last parameter (assumed to be runtime) for a specific feature.
///
/// e.g. To default the runtime generic to type `crate::Wry` when the `wry` feature is enabled, the
/// syntax would look like `#[default_runtime(crate::Wry, wry)`. This is **always** set for the last
/// generic, so make sure the last generic is the runtime when using this macro.
#[doc(hidden)]
#[proc_macro_attribute]
pub fn default_runtime(attributes: TokenStream, input: TokenStream) -> TokenStream {
  let attributes = parse_macro_input!(attributes as runtime::Attributes);
  let input = parse_macro_input!(input as runtime::Input);
  runtime::default_runtime(attributes, input).into()
}

/// Accepts a closure-like syntax to call arbitrary code on a menu item
/// after matching against `kind` and retrieving it from `resources_table` using `rid`.
///
/// You can optionally pass a 5th parameter to select which item kinds
/// to match against, by providing a `|` separated list of item kinds
/// ```ignore
/// do_menu_item!(resources_table, rid, kind, |i| i.set_text(text), Check | Submenu);
/// ```
/// You could also provide a negated list
/// ```ignore
/// do_menu_item!(resources_table, rid, kind, |i| i.set_text(text), !Check);
/// do_menu_item!(resources_table, rid, kind, |i| i.set_text(text), !Check | !Submenu);
/// ```
/// but you can't have mixed negations and positive kinds.
/// ```ignore
/// do_menu_item!(resources_table, rid, kind, |i| i.set_text(text), !Check | Submenu);
/// ```
///
/// #### Example
///
/// ```ignore
///  let rid = 23;
///  let kind = ItemKind::Check;
///  let resources_table = app.resources_table();
///  do_menu_item!(resources_table, rid, kind, |i| i.set_text(text))
/// ```
/// which will expand into:
/// ```ignore
///  let rid = 23;
///  let kind = ItemKind::Check;
///  let resources_table = app.resources_table();
///  match kind {
///  ItemKind::Submenu => {
///    let i = resources_table.get::<Submenu<R>>(rid)?;
///    i.set_text(text)
///  }
///  ItemKind::MenuItem => {
///    let i = resources_table.get::<MenuItem<R>>(rid)?;
///    i.set_text(text)
///  }
///  ItemKind::Predefined => {
///    let i = resources_table.get::<PredefinedMenuItem<R>>(rid)?;
///    i.set_text(text)
///  }
///  ItemKind::Check => {
///    let i = resources_table.get::<CheckMenuItem<R>>(rid)?;
///    i.set_text(text)
///  }
///  ItemKind::Icon => {
///    let i = resources_table.get::<IconMenuItem<R>>(rid)?;
///    i.set_text(text)
///  }
///  _ => unreachable!(),
///  }
/// ```
#[proc_macro]
pub fn do_menu_item(input: TokenStream) -> TokenStream {
  let tokens = parse_macro_input!(input as menu::DoMenuItemInput);
  menu::do_menu_item(tokens).into()
}

/// Convert a .png or .ico icon to an Image
/// for things like `tauri::tray::TrayIconBuilder` to consume,
/// relative paths are resolved from `CARGO_MANIFEST_DIR`, not current file
///
/// ### Examples
///
/// ```ignore
/// const APP_ICON: Image<'_> = include_image!("./icons/32x32.png");
///
/// // then use it with tray
/// TrayIconBuilder::new().icon(APP_ICON).build().unwrap();
///
/// // or with window
/// WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
///     .icon(APP_ICON)
///     .unwrap()
///     .build()
///     .unwrap();
///
/// // or with any other functions that takes `Image` struct
/// ```
///
/// Note: this stores the image in raw pixels to the final binary,
/// so keep the icon size (width and height) small
/// or else it's going to bloat your final executable
#[proc_macro]
pub fn include_image(tokens: TokenStream) -> TokenStream {
  let path = match parse2::<LitStr>(tokens.into()) {
    Ok(path) => path,
    Err(err) => return err.into_compile_error().into(),
  };
  let path = PathBuf::from(path.value());
  let resolved_path = if path.is_relative() {
    if let Ok(base_dir) = std::env::var("CARGO_MANIFEST_DIR").map(PathBuf::from) {
      base_dir.join(path)
    } else {
      return quote!(compile_error!("$CARGO_MANIFEST_DIR is not defined")).into();
    }
  } else {
    path
  };
  if !resolved_path.exists() {
    let error_string = format!(
      "Provided Image path \"{}\" doesn't exists",
      resolved_path.display()
    );
    return quote!(compile_error!(#error_string)).into();
  }

  match CachedIcon::new(&quote!(::tauri), &resolved_path).map_err(|error| error.to_string()) {
    Ok(icon) => icon.into_token_stream(),
    Err(error) => quote!(compile_error!(#error)),
  }
  .into()
}
