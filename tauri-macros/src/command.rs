use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::borrow::BorrowMut;
use syn::{
  parse::Parser, parse_quote, punctuated::Punctuated, FnArg, ItemFn, NestedMeta, Path, Token, Type,
};

pub fn generate_command(attrs: Vec<NestedMeta>, mut function: ItemFn) -> TokenStream {
  // Check if "webview" attr was passed to macro
  let uses_webview = attrs.iter().any(|a| {
    if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = a {
      path
        .get_ident()
        .map(|i| *i == "with_webview")
        .unwrap_or(false)
    } else {
      false
    }
  });

  if uses_webview {
    // Check if webview type should be automatically injected
    if let FnArg::Typed(arg) = function.sig.inputs[0].borrow_mut() {
      if let Type::Infer(_) = arg.ty.as_ref() {
        let default_type: Box<syn::Type> = parse_quote!(tauri::WebviewManager<tauri::flavors::Wry>);
        arg.ty = default_type;
      }
    }
  }

  let ident = function.sig.ident.clone();
  let params = function.sig.inputs.clone();
  let (mut names, mut types): (Vec<syn::Ident>, Vec<syn::Path>) = params
    .iter()
    .map(|param| {
      let mut arg_name = None;
      let mut arg_type = None;
      if let syn::FnArg::Typed(arg) = param {
        if let syn::Pat::Ident(ident) = arg.pat.as_ref() {
          arg_name = Some(ident.ident.clone());
        }
        if let syn::Type::Path(path) = arg.ty.as_ref() {
          arg_type = Some(path.path.clone());
        }
      }
      (arg_name.unwrap(), arg_type.unwrap())
    })
    .unzip();

  // Default generic for functions that don't take the webview
  let mut application_ext_generic = quote!(<A: tauri::ApplicationExt>);
  let mut webview_arg_type = quote!(tauri::WebviewManager<A>);
  let webview_arg = match types.first() {
    Some(first_type) => {
      if uses_webview {
        // If the function takes the webview, give it a specific type
        webview_arg_type = quote!(#first_type);
        application_ext_generic = quote!();
        types.drain(0..1);
        names.drain(0..1);
        quote!(_webview,)
      } else {
        quote!()
      }
    }
    None => quote!(),
  };

  let ident_wrapper = format_ident!("{}_wrapper", ident);
  let gen = quote! {
    #function
    pub fn #ident_wrapper #application_ext_generic(_webview: #webview_arg_type, arg: serde_json::Value) -> Option<tauri::InvokeResponse> {
      #[derive(serde::Deserialize)]
      #[serde(rename_all = "camelCase")]
      struct ParsedArgs {
        #(#names: #types),*
      }
      let parsed_args: ParsedArgs = serde_json::from_value(arg).unwrap();
      Some(#ident(#webview_arg #(parsed_args.#names),*).into())
    }
  };
  gen
}

pub fn generate_handler(item: proc_macro::TokenStream) -> TokenStream {
  let paths = <Punctuated<Path, Token![,]>>::parse_terminated
    .parse(item)
    .unwrap();
  let funcs = paths
    .iter()
    .map(|p| p.segments.last().unwrap().ident.clone());
  let funcs_wrapper = paths.iter().map(|func| {
    let mut func = func.clone();
    let ident = format_ident!("{}_wrapper", func.segments.last().unwrap().ident);
    func.segments.last_mut().unwrap().ident = ident;
    func
  });
  let gen = quote! {
    |webview, arg| async move {
      let dispatch: Result<tauri::DispatchInstructions, serde_json::Error> =
      serde_json::from_str(&arg);
      match dispatch {
        Err(e) => Err(e.into()),
        Ok(dispatch) => {
          let res = match dispatch.cmd.as_str() {
            #(stringify!(#funcs) => #funcs_wrapper(webview, dispatch.args),)*
            _ => None,
          };
          Ok(res.unwrap_or(().into()))
        }
      }
    }
  };
  gen
}
