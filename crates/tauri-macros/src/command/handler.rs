// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use quote::format_ident;
use syn::{
  parse::{Parse, ParseBuffer, ParseStream},
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
    let command_defs = input.parse_terminated(CommandDef::parse, Token![,])?;

    // parse the command names and wrappers from the passed paths
    let (commands, wrappers) = command_defs
      .iter()
      .map(|command_def| {
        let mut wrapper = command_def.path.clone();
        let last = super::path_to_command(&mut wrapper);

        // the name of the actual command function
        let command = last.ident.clone();

        // set the path to the command function wrapper
        last.ident = super::format_command_wrapper(&command);

        (command, wrapper)
      })
      .unzip();

    Ok(Self {
      command_defs: command_defs.into_iter().collect(), // remove punctuation separators
      commands,
      wrappers,
    })
  }
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
