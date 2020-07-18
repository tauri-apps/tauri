extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use std::path::{Path, PathBuf};
use syn::Lit::Str;
use syn::Meta::NameValue;
use syn::{parse_macro_input, DeriveInput, MetaNameValue};
use tauri_api::config::Config;

const DEFAULT_CONFIG_FILE: &str = "tauri.conf.json";
const GENERATED_ASSETS_FILE: &str = "tauri_assets.rs";

// just showcasing possibilities, ignore unwraps

#[proc_macro_derive(FromTauriConfig, attributes(tauri_config_path))]
pub fn from_tauri_config(ast: TokenStream) -> TokenStream {
  let input = parse_macro_input!(ast as DeriveInput);
  let name = input.ident;
  // because we are running this env var at **runtime** (which is during compilation)
  // it takes the env var from the project we are building. If we were to use env! instead then
  // it would show the manifest directory of this impl crate.
  let app_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

  // quick way of parsing #[tauri_config_path = "path_goes_here"]
  let mut config_file_path = DEFAULT_CONFIG_FILE.into();
  let tauri_config_path_attr = input
    .attrs
    .iter()
    .find(|attr| attr.path.is_ident("tauri_config_path"));
  if let Some(attr) = tauri_config_path_attr {
    if let Ok(meta) = attr.parse_meta() {
      if let NameValue(MetaNameValue { lit: Str(path), .. }) = meta {
        config_file_path = path.value()
      }
    }
  }

  let full_config_path = Path::new(&app_dir).join(config_file_path);
  let config = Config::read(&full_config_path).unwrap();
  let config_dir = full_config_path.parent().unwrap();
  let dist_dir = config_dir.join(config.build.dist);

  // generate the assets into a perfect hash function
  let assets_codegen = generate_assets(&dist_dir).unwrap();

  // copy over the build tauri index file from `tauri dev/build`
  // should be possible to do the manipulations during this codegen too in the future, if wanted
  let tauri_index_html = dist_dir.join("index.tauri.html");

  // format paths at string to use them in quote!
  let tauri_config_path = full_config_path.display().to_string();
  let assets_codegen = assets_codegen.display().to_string();
  let tauri_index_html = tauri_index_html.display().to_string();

  let output = quote! {
      use tauri::tauri_includedir;
      include!(#assets_codegen);
      impl tauri::api::private::AsTauriConfig for #name {
          fn config_path() -> &'static std::path::Path {
              std::path::Path::new(#tauri_config_path)
          }

          /// Make the file a dependency for the compiler
          fn raw_config() -> &'static str {
            include_str!(#tauri_config_path)
          }

          fn assets() -> &'static tauri_includedir::Files {
            &_TAURI_CONFIG_ASSETS
          }

          /// Make the index.tauri.html a dependency for the compiler
          fn raw_index() -> &'static str {
            include_str!(#tauri_index_html)
          }
      }
  };
  output.into()
}

fn generate_assets(dist: &Path) -> anyhow::Result<PathBuf> {
  let out = std::env::var("OUT_DIR")
    .as_ref()
    .map(Path::new)
    .map(|p| p.join(GENERATED_ASSETS_FILE))
    .unwrap();

  let mut inlined_assets = match std::env::var("TAURI_INLINED_ASSETS") {
    Ok(assets) => assets
      .split('|')
      .map(|s| s.to_string())
      .filter(|s| s != "")
      .collect(),
    Err(_) => Vec::new(),
  };

  // the index.html is parsed so we always ignore it
  inlined_assets.push(
    dist
      .join("index.html")
      .into_os_string()
      .into_string()
      .expect("failed to convert dist path to string"),
  );

  // disabled because i dont know how this case should be handled yet (cfg)
  //
  /*  if cfg!(feature = "no-server") {
    // on no-server we include_str() the index.tauri.html on the runner
    inlined_assets.push(
      dist
        .join("index.tauri.html")
        .into_os_string()
        .into_string()
        .expect("failed to convert dist path to string"),
    );
  }*/

  // include assets
  tauri_includedir_codegen::start("_TAURI_CONFIG_ASSETS")
    .dir(&dist, tauri_includedir_codegen::Compression::None)
    .build(GENERATED_ASSETS_FILE, inlined_assets)
    .expect(&format!(
      "failed to build OUT_DIR/{}",
      GENERATED_ASSETS_FILE
    ));

  Ok(out)
}
