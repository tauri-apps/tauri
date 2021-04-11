use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::{
  collections::HashMap,
  env::var,
  fs::File,
  io::BufReader,
  path::{Path, PathBuf},
};
use tauri_api::assets::AssetKey;
use thiserror::Error;
use walkdir::WalkDir;

/// (key, (original filepath, compressed bytes))
type Asset = (AssetKey, (String, Vec<u8>));

/// All possible errors while reading and compressing an [`EmbeddedAssets`] directory
#[derive(Debug, Error)]
pub enum EmbeddedAssetsError {
  #[error("failed to read asset at {path} because {error}")]
  AssetRead {
    path: PathBuf,
    error: std::io::Error,
  },

  #[error("failed to write asset from {path} to Vec<u8> because {error}")]
  AssetWrite {
    path: PathBuf,
    error: std::io::Error,
  },

  #[error("invalid prefix {prefix} used while including path {path}")]
  PrefixInvalid { prefix: PathBuf, path: PathBuf },

  #[error("failed to walk directory {path} because {error}")]
  Walkdir {
    path: PathBuf,
    error: walkdir::Error,
  },
}

/// Represent a directory of assets that are compressed and embedded.
///
/// This is the compile time generation of [`tauri_api::assets::Assets`] from a directory. Assets
/// from the directory are added as compiler dependencies by dummy including the original,
/// uncompressed assets.
///
/// The assets are compressed during this runtime, and can only be represented as a [`TokenStream`]
/// through [`ToTokens`]. The generated code is meant to be injected into an application to include
/// the compressed assets in that application's binary.
pub struct EmbeddedAssets(HashMap<AssetKey, (String, Vec<u8>)>);

impl EmbeddedAssets {
  #[cfg(not(debug_assertions))]
  /// Compress a directory of assets, ready to be generated into a [`tauri_api::assets::Assets`].
  pub fn new(path: &Path) -> Result<Self, EmbeddedAssetsError> {
    WalkDir::new(&path)
      .follow_links(true)
      .into_iter()
      .filter_map(|entry| match entry {
        // we only serve files, not directory listings
        Ok(entry) if entry.file_type().is_dir() => None,

        // compress all files encountered
        Ok(entry) => Some(Self::compress_file(path, entry.path())),

        // pass down error through filter to fail when encountering any error
        Err(error) => Some(Err(EmbeddedAssetsError::Walkdir {
          path: path.to_owned(),
          error,
        })),
      })
      .collect::<Result<_, _>>()
      .map(Self)
  }

  #[cfg(debug_assertions)]
  /// A dummy EmbeddedAssets for use during development builds.
  /// Compressing + including the bytes of assets during development takes a long time.
  /// On development builds, assets will simply be resolved & fetched from the configured dist folder.
  pub fn new(_: &Path) -> Result<Self, EmbeddedAssetsError> {
    Ok(EmbeddedAssets(HashMap::new()))
  }

  /// Use highest compression level for release, the fastest one for everything else
  fn compression_level() -> i32 {
    match var("PROFILE").as_ref().map(String::as_str) {
      Ok("release") => 22,
      _ => -5,
    }
  }

  /// Compress a file and spit out the information in a [`HashMap`] friendly form.
  fn compress_file(prefix: &Path, path: &Path) -> Result<Asset, EmbeddedAssetsError> {
    let reader =
      File::open(&path)
        .map(BufReader::new)
        .map_err(|error| EmbeddedAssetsError::AssetRead {
          path: path.to_owned(),
          error,
        })?;

    // entirely read compressed asset into bytes
    let bytes = zstd::encode_all(reader, Self::compression_level()).map_err(|error| {
      EmbeddedAssetsError::AssetWrite {
        path: path.to_owned(),
        error,
      }
    })?;

    // get a key to the asset path without the asset directory prefix
    let key = path
      .strip_prefix(prefix)
      .map(AssetKey::from) // format the path for use in assets
      .map_err(|_| EmbeddedAssetsError::PrefixInvalid {
        prefix: prefix.to_owned(),
        path: path.to_owned(),
      })?;

    Ok((key, (path.display().to_string(), bytes)))
  }
}

impl ToTokens for EmbeddedAssets {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let mut map = TokenStream::new();
    for (key, (original, bytes)) in &self.0 {
      let key: &str = key.as_ref();

      // add original asset as a compiler dependency, rely on dead code elimination to clean it up
      map.append_all(quote!(#key => {
        const _: &[u8] = include_bytes!(#original);
        &[#(#bytes),*]
      },));
    }

    // we expect phf related items to be in path when generating the path code
    tokens.append_all(quote! {
        use ::tauri::api::assets::{EmbeddedAssets, phf, phf::phf_map};
        static ASSETS: EmbeddedAssets = EmbeddedAssets::from_zstd(phf_map! { #map });
        &ASSETS
    });
  }
}
