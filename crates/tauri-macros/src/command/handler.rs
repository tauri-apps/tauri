// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use quote::format_ident;
use syn::{
  parse::{Parse, ParseBuffer, ParseStream},
  punctuated::Punctuated,
  Attribute, Ident, Path, Token,
};

struct CommandDef {
  path: Path,
  attrs: Vec<Attribute>,
}

impl Parse for CommandDef {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let path = input.parse()?;

    Ok(CommandDef { path, attrs })
  }
}

/// The items parsed from [`generate_handle!`](crate::generate_handle).
pub struct Handler {
  command_defs: Vec<CommandDef>,
  commands: Vec<Ident>,
  wrappers: Vec<Path>,
}

impl Parse for Handler {
  fn parse(input: &ParseBuffer<'_>) -> syn::Result<Self> {
    let plugin_name = try_get_plugin_name(input)?;

    let command_defs = input.parse_terminated(CommandDef::parse, Token![,])?;

    let command_defs = filter_unused_commands(plugin_name, command_defs);
    let mut commands = Vec::new();
    let mut wrappers = Vec::new();

    // parse the command names and wrappers from the passed paths
    for command_def in &command_defs {
      let mut wrapper = command_def.path.clone();
      let last = super::path_to_command(&mut wrapper);

      // the name of the actual command function
      let command = last.ident.clone();

      // set the path to the command function wrapper
      last.ident = super::format_command_wrapper(&command);

      commands.push(command);
      wrappers.push(wrapper);
    }

    Ok(Self {
      command_defs,
      commands,
      wrappers,
    })
  }
}

/// Try to get the plugin name by parsing the input for a `#![plugin(...)]` attribute,
/// if it's not present, try getting it from `CARGO_PKG_NAME` enviroment variable
fn try_get_plugin_name(input: &ParseBuffer<'_>) -> Result<Option<String>, syn::Error> {
  if let Ok(attrs) = input.call(Attribute::parse_inner) {
    for attr in attrs {
      if attr.path().is_ident("plugin") {
        // Parse the content inside #![plugin(...)]
        return Ok(Some(
          attr.parse_args::<Ident>()?.to_string().replace("_", "-"),
        ));
      }
    }
  }
  Ok(
    std::env::var("CARGO_PKG_NAME")
      .ok()
      .and_then(|var| var.strip_prefix("tauri-plugin-").map(String::from)),
  )
}

fn filter_unused_commands(
  plugin_name: Option<String>,
  command_defs: Punctuated<CommandDef, syn::token::Comma>,
) -> Vec<CommandDef> {
  let Some(plugin_name) = &plugin_name else {
    return command_defs.into_iter().collect();
  };
  let allowed_commands = tauri_utils::acl::read_allowed_commands();
  let Some(allowed_commands) = allowed_commands else {
    return command_defs.into_iter().collect();
  };
  command_defs
    .into_iter()
    .filter(move |command_def| {
      let mut wrapper = command_def.path.clone();
      let last = super::path_to_command(&mut wrapper);

      // the name of the actual command function
      let command = &last.ident;

      let command = format!("plugin:{plugin_name}|{command}");
      allowed_commands.contains(&command)
    })
    .collect()
}

impl From<Handler> for proc_macro::TokenStream {
  fn from(
    Handler {
      command_defs,
      commands,
      wrappers,
    }: Handler,
  ) -> Self {
    let cmd = format_ident!("__tauri_cmd__");
    let invoke = format_ident!("__tauri_invoke__");
    let (paths, attrs): (Vec<Path>, Vec<Vec<Attribute>>) = command_defs
      .into_iter()
      .map(|def| (def.path, def.attrs))
      .unzip();
    quote::quote!(move |#invoke| {
      let #cmd = #invoke.message.command();
      match #cmd {
        #(#(#attrs)* stringify!(#commands) => #wrappers!(#paths, #invoke),)*
        _ => {
          return false;
        },
      }
    })
    .into()
  }
}
