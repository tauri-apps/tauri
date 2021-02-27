use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse::Parser, punctuated::Punctuated, ItemFn, NestedMeta, Path, Token};

pub fn generate_command(attrs: Vec<NestedMeta>, function: ItemFn) -> TokenStream {
  let ident = function.sig.ident.clone();
  let params = function.sig.inputs.clone();
  let (mut names, mut types): (Vec<syn::Ident>, Vec<syn::Path>) = params
    .iter()
    .map(|param| {
      let mut arg_name = None;
      let mut arg_type = None;
      match param {
        syn::FnArg::Typed(rec) => {
          match rec.pat.as_ref() {
            syn::Pat::Ident(ident) => {
              arg_name = Some(ident.ident.clone());
            }
            _ => (),
          }
          match rec.ty.as_ref() {
            syn::Type::Path(path) => {
              arg_type = Some(path.path.clone());
            }
            _ => (),
          }
        }
        _ => (),
      }
      (arg_name.unwrap(), arg_type.unwrap())
    })
    .unzip();

  // Default generic for functions that don't take the webview
  let mut application_ext_generic = quote!(<A: tauri::ApplicationExt>);
  let mut webview_arg_type = quote!(tauri::WebviewManager<A>);
  let webview_arg = match types.first() {
    Some(first_type) => {
      // Check if "webview" attr was passed to macro
      if attrs.iter().any(|a| {
        if let syn::NestedMeta::Meta(meta) = a {
          if let syn::Meta::Path(path) = meta {
            path
              .get_ident()
              .map(|i| i.to_string() == "with_webview")
              .unwrap_or(false)
          } else {
            false
          }
        } else {
          false
        }
      }) {
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
    fn #ident_wrapper #application_ext_generic(_webview: #webview_arg_type, arg: serde_json::Value) -> Option<tauri::InvokeResponse> {
      #[derive(Deserialize)]
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
  let funcs: Vec<Ident> = paths
    .iter()
    .map(|func| func.get_ident().unwrap().clone())
    .collect();
  let funcs_wrapper = funcs.iter().map(|func| format_ident!("{}_wrapper", func));
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
