use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{capability::CapabilityContext, Value};

pub type ScopeKey = usize;

pub struct ResolvedCommand {
  pub windows: Vec<String>,
  pub scope: ScopeKey,
}

pub struct ResolvedScope<T>
where
  T: Serialize,
  for<'de> T: Deserialize<'de>,
{
  pub allow: Vec<T>,
  pub deny: Vec<T>,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct CommandKey {
  pub name: String,
  pub context: CapabilityContext,
}

pub struct Resolved {
  pub allowed_commands: BTreeMap<CommandKey, ResolvedCommand>,
  pub denied_commands: BTreeMap<CommandKey, ResolvedCommand>,
  pub scopes: BTreeMap<ScopeKey, ResolvedScope<Value>>,
}

#[cfg(feature = "build")]
mod build {
  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};
  use std::convert::identity;

  use super::*;
  use crate::{acl::Number, tokens::*};

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

  impl ToTokens for CapabilityContext {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::acl::capability::CapabilityContext };

      tokens.append_all(match self {
        Self::Local => {
          quote! { #prefix::Local }
        }
        Self::Remote { dangerous_remote } => {
          let dangerous_remote = vec_lit(dangerous_remote, str_lit);
          quote! { #prefix::Remote { dangerous_remote } }
        }
      });
    }
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

  impl ToTokens for Number {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::acl:::Number };

      tokens.append_all(match self {
        Self::Int(i) => {
          quote! { #prefix::Int(i) }
        }
        Self::Float(f) => {
          quote! { #prefix::Float (f) }
        }
      });
    }
  }

  impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::acl::Value };

      tokens.append_all(match self {
        Value::Bool(bool) => quote! { #prefix::Bool(#bool) },
        Value::Number(number) => quote! { #prefix::Number(#number) },
        Value::String(str) => {
          let s = str_lit(str);
          quote! { #prefix::String(#s) }
        }
        Value::List(vec) => {
          let items = vec_lit(vec, identity);
          quote! { #prefix::Array(#items) }
        }
        Value::Map(map) => {
          let map = map_lit(quote! { ::tauri::acl::Map }, map, str_lit, identity);
          quote! { #prefix::Map(#map) }
        }
      });
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
