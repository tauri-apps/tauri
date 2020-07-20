extern crate proc_macro;
use crate::error::Error;
use proc_macro::TokenStream;
use std::env::var;
use std::path::PathBuf;
use syn::{parse_macro_input, DeriveInput};

mod error;
mod expand;
mod include_dir;

const DEFAULT_CONFIG_FILE: &str = "tauri.conf.json";

struct Env {
  #[allow(dead_code)] // we will need this for gzip compression
  out: PathBuf,
  manifest: PathBuf,
}

impl Env {
  fn new() -> Result<Self, Error> {
    let out = var("OUT_DIR")
      .map(PathBuf::from)
      .map_err(|_| Error::EnvOutDir)?;

    let manifest = var("CARGO_MANIFEST_DIR")
      .map(PathBuf::from)
      .map_err(|_| Error::EnvCargoManifestDir)?;

    Ok(Self { out, manifest })
  }
}

#[proc_macro_derive(FromTauriConfig, attributes(tauri_config_path))]
pub fn from_tauri_config(ast: TokenStream) -> TokenStream {
  let input = parse_macro_input!(ast as DeriveInput);
  match expand::from_tauri_config(input) {
    Ok(ast) => ast.into(),
    Err(err) => err.into(),
  }
}
