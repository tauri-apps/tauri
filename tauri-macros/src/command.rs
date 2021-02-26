use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse::Parser, punctuated::Punctuated, Attribute, ItemFn, Path, Token};

pub fn generate_command(_attrs: Vec<Attribute>, function: ItemFn) -> TokenStream {
  let ident = function.sig.ident.clone();
  let params = function.sig.inputs.clone();
  let (names, types): (Vec<syn::Ident>, Vec<syn::Ident>) = params
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
              arg_type = Some(path.path.get_ident().unwrap().clone());
            }
            _ => (),
          }
        }
        _ => (),
      }
      (arg_name.unwrap(), arg_type.unwrap())
    })
    .unzip();
  let ident_wrapper = format_ident!("{}_wrapper", ident);
  let gen = quote! {
    #function
    fn #ident_wrapper (arg: serde_json::Value) -> Option<tauri::InvokeResponse> {
      #[derive(Deserialize)]
      #[serde(rename_all = "camelCase")]
      struct ParsedArgs {
        #(#names: #types),*
      }
      let parsed_args: ParsedArgs = serde_json::from_value(arg).unwrap();
      Some(#ident(#(parsed_args.#names),*).into())
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
    |_webview, arg| async move {
      let dispatch: Result<tauri::DispatchInstructions, serde_json::Error> =
      serde_json::from_str(&arg);
      match dispatch {
        Err(e) => Err(e.into()),
        Ok(dispatch) => {
          let res = match dispatch.cmd.as_str() {
            #(stringify!(#funcs) => #funcs_wrapper(dispatch.args),)*
            _ => None,
          };
          Ok(res.unwrap_or(().into()))
        }
      }
    }
  };
  gen
}
