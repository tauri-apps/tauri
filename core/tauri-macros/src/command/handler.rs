// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
  parse::{Parse, ParseBuffer},
  Path, PathSegment, Token,
};

/// The items passed to [`generate_handle!`](crate::generate_handle).
pub struct Handler {
  paths: Vec<Path>,
  commands: Vec<Ident>,
  wrappers: Vec<Path>,
}

impl Parse for Handler {
  fn parse(input: &ParseBuffer) -> syn::Result<Self> {
    let paths = input.parse_terminated::<Path, Token![,]>(Path::parse)?;

    // parse the command names and wrappers from the passed paths
    let (commands, wrappers) = paths
      .iter()
      .map(|path| {
        let mut wrapper = path.clone();
        let last = path_to_command(&mut wrapper);

        // the name of the actual command function
        let command = last.ident.clone();

        // set the path to the command function wrapper
        last.ident = super::format_command_wrapper(&command);

        (command, wrapper)
      })
      .unzip();

    Ok(Self {
      paths: paths.into_iter().collect(), // remove punctuation separators
      commands,
      wrappers,
    })
  }
}

impl ToTokens for Handler {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let Self {
      paths,
      commands,
      wrappers,
    } = self;

    tokens.append_all(quote!(move |invoke| {
      let cmd = invoke.message.command();
      match cmd {
        #(stringify!(#commands) => #wrappers!(#paths, invoke),)*
        _ => {
          invoke.resolver.reject(format!("command {} not found", cmd))
        },
      }
    }));
  }
}

impl From<Handler> for proc_macro::TokenStream {
  #[inline(always)]
  fn from(handler: Handler) -> Self {
    handler.to_token_stream().into()
  }
}

/// This function will panic if the passed [`syn::Path`] does not have any segments.
fn path_to_command(path: &mut Path) -> &mut PathSegment {
  path
    .segments
    .last_mut()
    .expect("parsed syn::Path has no segment")
}
