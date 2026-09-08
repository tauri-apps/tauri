// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Forwards a declaration to one of `cef`'s `wrap_*!` macros and adds a
/// named-args constructor next to the positional `new()` they generate.
///
/// `new()` takes one argument per struct field, so the wrappers that carry a
/// lot of state — `TauriCefBrowserClient` most of all — are built from a long
/// positional list in which two same-typed neighbours (`devtools_enabled` and
/// `drag_drop_handler_enabled`, say) can be swapped without the compiler
/// noticing. `build()` takes a single args struct instead, so every value is
/// named at the call site, and the forwarding to `new()` is generated from the
/// very field list the wrapper was declared with.
///
/// ```ignore
/// wrap_with_args! {
///   wrap_client => TauriCefBrowserClientArgs;
///
///   pub(crate) struct TauriCefBrowserClient<T: UserEvent> {
///     pub(crate) context: RuntimeContext<T>,
///     devtools_enabled: bool,
///   }
///
///   impl Client {
///     // ... same method bodies as a plain `wrap_client!` block
///   }
/// }
///
/// TauriCefBrowserClient::build(TauriCefBrowserClientArgs {
///   context,
///   devtools_enabled: true,
/// });
/// ```
///
/// The args fields are `pub(crate)` whatever visibility the wrapper gives its
/// own fields: anything that can name the args struct has to be able to fill in
/// every one of them, while the wrapper keeps its own fields as private as it
/// declared them.
macro_rules! wrap_with_args {
  (
    $wrap:ident => $args:ident;

    $vis:vis struct $name:ident$(<
      $($generic_type:ident : $first_generic_type_bound:tt $(+ $generic_type_bound:tt)*),+ $(,)?
    >)? {
      $($field_vis:vis $field_name:ident: $field_type:ty),* $(,)?
    }

    impl $interface:ident {
      $($methods:tt)*
    }
  ) => {
    $vis struct $args$(<$($generic_type,)+>)?
    $(where
      $($generic_type: $first_generic_type_bound $(+ $generic_type_bound)*,)+
    )?
    {
      $(pub(crate) $field_name: $field_type,)*
    }

    $wrap! {
      $vis struct $name$(<$($generic_type: $first_generic_type_bound $(+ $generic_type_bound)*),+>)? {
        $($field_vis $field_name: $field_type,)*
      }

      impl $interface {
        $($methods)*
      }
    }

    impl$(<$($generic_type,)+>)? $name$(<$($generic_type,)+>)?
    $(where
      $($generic_type: $first_generic_type_bound $(+ $generic_type_bound)*,)+
    )?
    {
      $vis fn build(args: $args$(<$($generic_type,)+>)?) -> $interface {
        Self::new($(args.$field_name),*)
      }
    }
  };
}

pub(crate) use wrap_with_args;
