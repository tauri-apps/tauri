//! Resolved ACL for runtime usage.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{ExecutionContext, Value};

/// A key for a scope, used to link a [`ResolvedCommand#structfield.scope`] to the store [`Resolved#structfield.scopes`].
pub type ScopeKey = usize;

/// A resolved command permission.
#[derive(Debug)]
pub struct ResolvedCommand {
  /// The list of window label patterns that is allowed to run this command.
  pub windows: Vec<String>,
  /// The reference of the scope that is associated with this command. See [`Resolved#structfield.scopes`].
  pub scope: ScopeKey,
}

/// A resolved scope. Contains all scopes defined for a single command.
pub struct ResolvedScope<T>
where
  T: Serialize,
  for<'de> T: Deserialize<'de>,
{
  /// Allows something on the command.
  pub allow: Vec<T>,
  /// Denies something on the command.
  pub deny: Vec<T>,
}

/// A command key for the map of allowed and denied commands.
/// Takes into consideration the command name and the execution context.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct CommandKey {
  /// The full command name.
  pub name: String,
  /// The context of the command.
  pub context: ExecutionContext,
}

/// Resolved access control list.
pub struct Resolved {
  /// The commands that are allowed. Map each command with its context to a [`ResolvedCommand`].
  pub allowed_commands: BTreeMap<CommandKey, ResolvedCommand>,
  /// The commands that are denied. Map each command with its context to a [`ResolvedCommand`].
  pub denied_commands: BTreeMap<CommandKey, ResolvedCommand>,
  /// The store of scopes referenced by a [`ResolvedCommand`].
  pub scopes: BTreeMap<ScopeKey, ResolvedScope<Value>>,
}

#[cfg(feature = "build")]
mod build {
  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};
  use std::convert::identity;

  use super::*;
  use crate::tokens::*;

  /// Write a `TokenStream` of the `$struct`'s fields to the `$tokens`.
  ///
  /// All fields must represent a binding of the same name that implements `ToTokens`.
  macro_rules! literal_struct {
    ($tokens:ident, $struct:ident, $($field:ident),+) => {
      $tokens.append_all(quote! {
        ::tauri::utils::acl::resolved::$struct {
          $($field: #$field),+
        }
      })
    };
  }

  impl ToTokens for CommandKey {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let name = str_lit(&self.name);
      let context = &self.context;
      literal_struct!(tokens, CommandKey, name, context)
    }
  }

  impl ToTokens for ResolvedCommand {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let windows = vec_lit(&self.windows, str_lit);
      let scope = self.scope;
      literal_struct!(tokens, ResolvedCommand, windows, scope)
    }
  }

  impl ToTokens for ResolvedScope<Value> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let allow = vec_lit(&self.allow, identity);
      let deny = vec_lit(&self.deny, identity);
      literal_struct!(tokens, ResolvedScope, allow, deny)
    }
  }

  impl ToTokens for Resolved {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let allowed_commands = map_lit(
        quote! { ::std::collections::BTreeMap },
        &self.allowed_commands,
        identity,
        identity,
      );

      let denied_commands = map_lit(
        quote! { ::std::collections::BTreeMap },
        &self.denied_commands,
        identity,
        identity,
      );

      let scopes = map_lit(
        quote! { ::std::collections::BTreeMap },
        &self.scopes,
        identity,
        identity,
      );

      literal_struct!(tokens, Resolved, allowed_commands, denied_commands, scopes)
    }
  }
}
