use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::io::Error as IoError;
use std::path::PathBuf;
use Error::*;

pub(crate) enum Error {
  EnvOutDir,
  EnvCargoManifestDir,
  IncludeDirPrefix,
  IncludeDirCacheDir,
  IncludeDirEmptyFilename,
  ConfigDir,
  Serde(PathBuf, serde_json::Error),
  Io(PathBuf, IoError),
}

impl Error {
  /// Output a compiler error to the ast being transformed
  pub(crate) fn into_compile_error(self, struct_: &Ident) -> TokenStream {
    let error: String = match self {
      EnvOutDir => "Unable to find OUT_DIR environmental variable from tauri-macros".into(),
      EnvCargoManifestDir => {
        "Unable to find CARGO_MANIFEST_DIR environmental variable from tauri-macros".into()
      }
      IncludeDirPrefix => "Invalid directory prefix encountered while including assets".into(),
      IncludeDirCacheDir => {
        "Unable to find cache directory to compress assets into during tauri-macros".into()
      }
      IncludeDirEmptyFilename => "Asset included during tauri-macros has empty filename".into(),
      ConfigDir => {
        "Unable to get the directory the config file was found in during tauri-macros".into()
      }
      Serde(path, error) => format!(
        "{:?} encountered for {} during tauri-macros",
        error,
        path.display()
      ),
      Io(path, error) => format!(
        "{:?} encountered for {} during tauri-macros",
        error.kind(),
        path.display()
      ),
    };

    quote! {
      compile_error!(#error);

      impl ::tauri::api::private::AsTauriContext for #struct_ {
        fn config_path() -> &'static std::path::Path {
          unimplemented!()
        }

        /// Make the file a dependency for the compiler
        fn raw_config() -> &'static str {
          unimplemented!()
        }

        fn assets() -> &'static ::tauri::api::assets::Assets {
          unimplemented!()
        }

        /// Make the index.tauri.html a dependency for the compiler
        fn raw_index() -> &'static str {
          unimplemented!()
        }

        /// Make the __tauri.js a dependency for the compiler
        fn raw_tauri_script() -> &'static str {
          unimplemented!()
        }
      }
    }
  }
}
