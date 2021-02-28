use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
  parse::Parser, punctuated::Punctuated, FnArg, Ident, ItemFn, Meta, NestedMeta, Pat, Path, Token,
  Type,
};

pub fn generate_command(attrs: Vec<NestedMeta>, function: ItemFn) -> TokenStream {
  // Check if "webview" attr was passed to macro
  let uses_webview = attrs.iter().any(|a| {
    if let NestedMeta::Meta(Meta::Path(path)) = a {
      path
        .get_ident()
        .map(|i| *i == "with_webview")
        .unwrap_or(false)
    } else {
      false
    }
  });

  let fn_name = function.sig.ident.clone();
  let fn_wrapper = format_ident!("{}_wrapper", fn_name);

  // Split function args into names and types
  let (mut names, mut types): (Vec<Ident>, Vec<Path>) = function
    .sig
    .inputs
    .clone()
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
        arg_type.expect(&format!(
          "#[command]: Invalid type for arg \"{}\"",
          arg_name.unwrap()
        )),
      )
    })
    .unzip();

  // If function doesn't take the webview, wrapper just takes webview generically and ignores it
  // Otherwise the wrapper uses the specific type from the original function declaration
  let mut webview_arg_type = quote!(tauri::WebviewManager<A>);
  let mut application_ext_generic = quote!(<A: tauri::ApplicationExt>);
  let webview_arg_maybe = match types.first() {
    Some(first_type) if uses_webview => {
      // Give wrapper specific type
      webview_arg_type = quote!(#first_type);
      // Generic is no longer needed
      application_ext_generic = quote!();
      // Remove webview arg from list so it isn't expected as arg from JS
      types.drain(0..1);
      names.drain(0..1);
      // Tell wrapper to pass webview to original function
      quote!(_webview,)
    }
    // Tell wrapper not to pass webview to original function
    _ => quote!(),
  };

  quote! {
    #function
    pub fn #fn_wrapper #application_ext_generic(_webview: #webview_arg_type, arg: serde_json::Value) -> Option<tauri::InvokeResponse> {
      #[derive(serde::Deserialize)]
      #[serde(rename_all = "camelCase")]
      struct ParsedArgs {
        #(#names: #types),*
      }
      let parsed_args: ParsedArgs = serde_json::from_value(arg).unwrap();
      Some(#fn_name(#webview_arg_maybe #(parsed_args.#names),*).into())
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
    |webview, arg| async move {
      let dispatch: Result<tauri::DispatchInstructions, serde_json::Error> =
      serde_json::from_str(&arg);
      match dispatch {
        Err(e) => Err(e.into()),
        Ok(dispatch) => {
          let res = match dispatch.cmd.as_str() {
            #(stringify!(#fn_names) => #fn_wrappers(webview, dispatch.args),)*
            _ => None,
          };
          Ok(res.unwrap_or(().into()))
        }
      }
    }
  }
}
