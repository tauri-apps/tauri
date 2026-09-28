// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Create macros for `tauri::Context`, invoke handler and commands leveraging the `tauri-codegen` crate.
//!
//! Don't depend on this crate directly, use the re-exported types from tauri instead.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]

use std::path::PathBuf;

use crate::context::ContextItems;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{LitStr, parse_macro_input, parse2};
use tauri_codegen::image::CachedIcon;

mod command;
mod menu;
mod mobile;
mod runtime;

#[macro_use]
mod context;

/// Mark a function as a command handler. It creates a wrapper function with the necessary glue code.
///
/// The wrapped function can then be passed to [`generate_handler!`] so it can be called
/// from the frontend with `invoke()`.
///
/// ```rust,ignore
/// #[tauri::command]
/// fn greet(name: String) -> String {
///   format!("Hello, {name}!")
/// }
/// ```
///
/// # Options
///
/// The attribute accepts a comma separated list of the following options:
///
/// ## `async`
///
/// Runs the command on the async runtime instead of blocking the main thread.
///
/// `async fn` commands are always executed asynchronously, so this option is only needed
/// for synchronous functions that should not run on the main thread:
///
/// ```rust,ignore
/// #[tauri::command(async)]
/// fn expensive_computation() -> u64 {
///   // the body runs on the async runtime, so the main thread is not blocked
///   42
/// }
/// ```
///
/// ## `rename_all`
///
/// Sets the case convention used to match the command arguments with the keys of the
/// payload sent by the frontend. Either `"camelCase"` (default) or `"snake_case"`.
///
/// ```rust,ignore
/// // called from JavaScript with `invoke("send_message", { messageBody: "Hello" })`
/// #[tauri::command]
/// fn send_message(message_body: String) {}
///
/// // called from JavaScript with `invoke("send_message", { message_body: "Hello" })`
/// #[tauri::command(rename_all = "snake_case")]
/// fn send_message_snake(message_body: String) {}
/// ```
///
/// ## `rename`
///
/// Changes the name used to call the command from the frontend.
/// By default it is the name of the function.
///
/// ```rust,ignore
/// // called from JavaScript with `invoke("greetUser")`
/// // and still registered as `generate_handler![greet_user]`
/// #[tauri::command(rename = "greetUser")]
/// fn greet_user() {}
/// ```
///
/// ## `root`
///
/// Path to the `tauri` crate, used when it is renamed in `Cargo.toml` or re-exported
/// by another crate. Defaults to `::tauri`, and the special value `"crate"` resolves
/// to `$crate` (used internally by Tauri itself).
///
/// ```rust,ignore
/// // Cargo.toml: tauri_framework = { package = "tauri", version = "2" }
/// #[tauri::command(root = "tauri_framework")]
/// fn my_command() {}
/// ```
///
/// # Inline plugins
///
/// When the command belongs to a plugin that is part of your application instead of a
/// standalone crate, annotate it in the [`generate_handler!`] list with the
/// `#![plugin(your_plugin_name)]` inner attribute so `build > removeUnusedCommands` can
/// match it against the plugin permissions.
///
/// # Stability
/// The output of this macro is managed internally by Tauri,
/// and should not be accessed directly on normal applications.
/// It may have breaking changes in the future.
#[proc_macro_attribute]
pub fn command(attributes: TokenStream, item: TokenStream) -> TokenStream {
  command::wrapper(attributes, item)
}

/// Marks a function as the entry point of a mobile application.
///
/// It must be applied to the function that builds and runs your Tauri application on the
/// library target (`run()` on `src-tauri/src/lib.rs` for apps created by the Tauri CLI),
/// which is the function the generated Android and iOS projects call on startup.
///
/// The canonical usage only applies it on mobile targets, so the same function can be called
/// by the `main.rs` of the desktop binary:
///
/// ```rust,ignore
/// #[cfg_attr(mobile, tauri::mobile_entry_point)]
/// pub fn run() {
///   tauri::Builder::default()
///     .run(tauri::generate_context!())
///     .expect("error while running tauri application");
/// }
/// ```
///
/// The macro generates a `start_app` C symbol (checked by the Tauri CLI) that catches
/// panics instead of unwinding across the FFI boundary, blocks on the function when it is
/// `async`, sets up the stdout logger on iOS and the JNI bindings on Android using the
/// package name derived from the `identifier` in your Tauri configuration.
///
/// Because the Android package name is read from environment variables set by `tauri-build`,
/// your application must have a build script calling [`tauri_build::build`] - otherwise the
/// macro fails to compile with a `env var not set` error.
///
/// [`tauri_build::build`]: https://docs.rs/tauri-build/latest/tauri_build/fn.build.html
#[proc_macro_attribute]
pub fn mobile_entry_point(attributes: TokenStream, item: TokenStream) -> TokenStream {
  mobile::entry_point(attributes, item)
}

/// Accepts a list of command functions. Creates a handler that allows commands to be called from JS with invoke().
///
/// You can optionally annotate the commands with a inner attribute tag `#![plugin(your_plugin_name)]`
/// for `build > removeUnusedCommands` to work for plugins not defined in a standalone crate like `tauri-plugin-fs`
///
/// # Examples
///
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
///
/// # Stability
///
/// The output of this macro is managed internally by Tauri,
/// and should not be accessed directly on normal applications.
/// It may have breaking changes in the future.
#[proc_macro]
pub fn generate_handler(item: TokenStream) -> TokenStream {
  parse_macro_input!(item as command::Handler).into()
}

/// Reads a Tauri config file and generates a `::tauri::Context` based on the content.
///
/// The context embeds the frontend assets, the application icons, the resolved Access Control List
/// and the parsed configuration into the binary, and is passed to
/// `tauri::Builder::run`/`tauri::Builder::build`.
///
/// ```rust,ignore
/// tauri::Builder::default()
///   .run(tauri::generate_context!())
///   .expect("error while running tauri application");
/// ```
///
/// # Options
///
/// All options are optional and can be combined in a comma separated list.
///
/// ## Configuration file path
///
/// A string literal as the first argument sets the path of the Tauri configuration file to read,
/// relative to `CARGO_MANIFEST_DIR`. Defaults to `tauri.conf.json` on the crate directory.
/// Platform specific configuration files (e.g. `tauri.windows.conf.json`) that sit next to it are
/// merged as usual.
///
/// ```rust,ignore
/// tauri::generate_context!("../tauri.conf.json");
/// ```
///
/// ## Root path
///
/// A path (any item that is not a `key = value` pair) changes the crate path the generated code
/// refers to. Defaults to `::tauri`, and is only needed when the `tauri` crate is renamed,
/// re-exported by another crate, or is the crate being compiled (`crate`).
///
/// ```rust,ignore
/// tauri::generate_context!("../tauri.conf.json", ::my_framework::tauri);
/// ```
///
/// ## `capabilities`
///
/// A list of additional capability files to include in the generated Access Control List,
/// on top of the ones defined in the `capabilities` directory and in the
/// `app > security > capabilities` configuration value.
///
/// Each item is a path (relative to the current working directory of the compiler,
/// usually the crate directory) to a JSON or TOML file containing a capability,
/// a list of capabilities or a named list of capabilities.
///
/// ```rust,ignore
/// tauri::generate_context!(capabilities = ["./capabilities/extra.json"]);
/// ```
///
/// ## `assets`
///
/// An expression resolving to a custom [`tauri::Assets`] implementation, used instead of
/// embedding the files from `build > frontendDist`. Useful for serving the frontend from a
/// custom source, or for skipping asset embedding on tests.
///
/// ```rust,ignore
/// tauri::generate_context!(assets = tauri::test::noop_assets());
/// ```
///
/// ## `test`
///
/// When `true`, skips code generation that misbehaves when the context is created inside a
/// test binary - currently the `Info.plist` embedding performed on macOS development builds.
/// Defaults to `false`.
///
/// ```rust,ignore
/// let context = tauri::generate_context!("../tauri.conf.json", test = true);
/// ```
///
/// [`tauri::Assets`]: https://docs.rs/tauri/latest/tauri/trait.Assets.html
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
/// ## Examples
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
///    ItemKind::Submenu => {
///      let i = resources_table.get::<Submenu<R>>(rid)?;
///      i.set_text(text)
///    }
///    ItemKind::MenuItem => {
///      let i = resources_table.get::<MenuItem<R>>(rid)?;
///      i.set_text(text)
///    }
///    ItemKind::Predefined => {
///      let i = resources_table.get::<PredefinedMenuItem<R>>(rid)?;
///      i.set_text(text)
///    }
///    ItemKind::Check => {
///      let i = resources_table.get::<CheckMenuItem<R>>(rid)?;
///      i.set_text(text)
///    }
///    ItemKind::Icon => {
///      let i = resources_table.get::<IconMenuItem<R>>(rid)?;
///      i.set_text(text)
///    }
///    _ => return Err(crate::Error::UnexpectedMenuKind),
///  }
/// ```
#[doc(hidden)]
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
