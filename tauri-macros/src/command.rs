use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
  parse::Parser, punctuated::Punctuated, FnArg, Ident, ItemFn, Meta, NestedMeta, Pat, Path,
  ReturnType, Token, Type,
};

pub fn generate_command(attrs: Vec<NestedMeta>, function: ItemFn) -> TokenStream {
  // Check if "with_window" attr was passed to macro
  let with_window = attrs.iter().any(|a| {
    if let NestedMeta::Meta(Meta::Path(path)) = a {
      path
        .get_ident()
        .map(|i| *i == "with_window")
        .unwrap_or(false)
    } else {
      false
    }
  });

  let fn_name = function.sig.ident.clone();
  let fn_name_str = fn_name.to_string();
  let fn_wrapper = format_ident!("{}_wrapper", fn_name);
  let returns_result = match function.sig.output {
    ReturnType::Type(_, ref ty) => match &**ty {
      Type::Path(type_path) => {
        type_path
          .path
          .segments
          .first()
          .map(|seg| seg.ident.to_string())
          == Some("Result".to_string())
      }
      _ => false,
    },
    ReturnType::Default => false,
  };

  // Split function args into names and types
  let (mut names, mut types): (Vec<Ident>, Vec<Path>) = function
    .sig
    .inputs
    .iter()
    .map(|param| {
      let mut arg_name = None;
      let mut arg_type = None;
      if let FnArg::Typed(arg) = param {
        if let Pat::Ident(ident) = arg.pat.as_ref() {
          arg_name = Some(ident.ident.clone());
        }
        if let Type::Path(path) = arg.ty.as_ref() {
          arg_type = Some(path.path.clone());
        }
      }
      (
        arg_name.clone().unwrap(),
        arg_type.unwrap_or_else(|| panic!("Invalid type for arg \"{}\"", arg_name.unwrap())),
      )
    })
    .unzip();

  let window_arg_maybe = match types.first() {
    Some(_) if with_window => {
      // Remove window arg from list so it isn't expected as arg from JS
      types.drain(0..1);
      names.drain(0..1);
      // Tell wrapper to pass `window` to original function
      quote!(_window,)
    }
    // Tell wrapper not to pass `window` to original function
    _ => quote!(),
  };
  let await_maybe = if function.sig.asyncness.is_some() {
    quote!(.await)
  } else {
    quote!()
  };

  // if the command handler returns a Result,
  // we just map the values to the ones expected by Tauri
  // otherwise we wrap it with an `Ok()`, converting the return value to tauri::InvokeResponse
  // note that all types must implement `serde::Serialize`.
  let return_value = if returns_result {
    quote! {
      match #fn_name(#window_arg_maybe #(parsed_args.#names),*)#await_maybe {
        Ok(value) => ::core::result::Result::Ok(value),
        Err(e) => ::core::result::Result::Err(e),
      }
    }
  } else {
    quote! { ::core::result::Result::<_, ()>::Ok(#fn_name(#window_arg_maybe #(parsed_args.#names),*)#await_maybe) }
  };

  quote! {
    #function
    pub fn #fn_wrapper<P: ::tauri::Params>(message: ::tauri::InvokeMessage<P>) {
      #[derive(::serde::Deserialize)]
      #[serde(rename_all = "camelCase")]
      struct ParsedArgs {
        #(#names: #types),*
      }
      let _window = message.window();
      match ::serde_json::from_value::<ParsedArgs>(message.payload()) {
        Ok(parsed_args) => message.respond_async(async move {
          #return_value
        }),
        Err(e) => message.reject(::core::result::Result::<(), String>::Err(::tauri::Error::InvalidArgs(#fn_name_str, e).to_string())),
      }
    }
  }
}

pub fn generate_handler(item: proc_macro::TokenStream) -> TokenStream {
  // Get paths of functions passed to macro
  let paths = <Punctuated<Path, Token![,]>>::parse_terminated
    .parse(item)
    .expect("generate_handler!: Failed to parse list of command functions");

  // Get names of functions, used for match statement
  let fn_names = paths
    .iter()
    .map(|p| p.segments.last().unwrap().ident.clone());

  // Get paths to wrapper functions
  let fn_wrappers = paths.iter().map(|func| {
    let mut func = func.clone();
    let mut last_segment = func.segments.last_mut().unwrap();
    last_segment.ident = format_ident!("{}_wrapper", last_segment.ident);
    func
  });

  quote! {
    move |message| {
      match message.command() {
        #(stringify!(#fn_names) => #fn_wrappers(message),)*
        _ => {},
      }
    }
  }
}
