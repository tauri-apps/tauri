use crate::assets::Error;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::path::{Path, PathBuf};

/// Represent a directory of assets to fetch at runtime.
pub struct DiskAssets(PathBuf);

impl DiskAssets {
  /// Create a new [`DiskAssets`] from the canonical version of the passed path.
  pub fn new(path: &Path) -> Result<Self, Error> {
    path
      .canonicalize()
      .map(DiskAssets)
      .map_err(|error| Error::Canonicalize {
        path: path.into(),
        error,
      })
  }
}

impl ToTokens for DiskAssets {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let dist_dir = self.0.display().to_string();
    tokens.append_all(quote! (::tauri::api::assets::DiskAssets::from_dist_dir(#dist_dir)));
  }
}
