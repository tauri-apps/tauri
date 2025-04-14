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
    let plugin_name = try_get_plugin_name(input)?;

    let mut command_defs = input
      .parse_terminated(CommandDef::parse, Token![,])?
      .into_iter()
      .collect();

    filter_unused_commands(plugin_name, &mut command_defs);
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
/// if it's not present, try getting it from `CARGO_PKG_NAME` environment variable
fn try_get_plugin_name(input: &ParseBuffer<'_>) -> Result<Option<String>, syn::Error> {
  if let Ok(attrs) = input.call(Attribute::parse_inner) {
    for attr in attrs {
      if attr.path().is_ident("plugin") {
        // Parse the content inside #![plugin(...)]
        let plugin_name = attr.parse_args::<Ident>()?.to_string();
        return Ok(Some(if plugin_name == "__TAURI_CHANNEL__" {
          // TODO: Remove this in v3
          plugin_name
        } else {
          plugin_name.replace("_", "-")
        }));
      }
    }
  }
  Ok(
    std::env::var("CARGO_PKG_NAME")
      .ok()
      .and_then(|var| var.strip_prefix("tauri-plugin-").map(String::from)),
  )
}

fn filter_unused_commands(plugin_name: Option<String>, command_defs: &mut Vec<CommandDef>) {
  let allowed_commands = tauri_utils::acl::read_allowed_commands();
  let Some(allowed_commands) = allowed_commands else {
    return;
  };

  // TODO: Remove this in v3
  if plugin_name.as_deref() == Some("__TAURI_CHANNEL__") {
    // Always allowed
    return;
  }

  if plugin_name.is_none() && !allowed_commands.has_app_acl {
    // All application commands are allowed if we don't have an application ACL
    //
    // note that inline plugins without the #![plugin()] attribute would also get to this check
    // which means inline plugins must have an app manifest to get proper unused command removal
    return;
  }

  let mut unused_commands = Vec::new();

  let command_prefix = if let Some(plugin_name) = &plugin_name {
    format!("plugin:{plugin_name}|")
  } else {
    "".into()
  };

  command_defs.retain(|command_def| {
    let mut wrapper = command_def.path.clone();
    let last = super::path_to_command(&mut wrapper);

    // the name of the actual command function
    let command_name = &last.ident;

    let command = format!("{command_prefix}{command_name}");
    let is_allowed = allowed_commands.commands.contains(&command);

    if !is_allowed {
      unused_commands.push(command_name.to_string());
    }

    is_allowed
  });

  if !unused_commands.is_empty() {
    let plugin_display_name = plugin_name.as_deref().unwrap_or("application");
    let unused_commands_display = unused_commands.join(", ");
    println!("Removed unused commands from {plugin_display_name}: {unused_commands_display}",);
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
