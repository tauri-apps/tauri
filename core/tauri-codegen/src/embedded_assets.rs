// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use sha2::{Digest, Sha256};
use std::{
  collections::HashMap,
  fmt::Write,
  fs::File,
  path::{Path, PathBuf},
};
use tauri_utils::config::PatternKind;
use tauri_utils::{assets::AssetKey, config::DisabledCspModificationKind};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

#[cfg(feature = "compression")]
use brotli::enc::backward_references::BrotliEncoderParams;

/// The subdirectory inside the target directory we want to place assets.
const TARGET_PATH: &str = "tauri-codegen-assets";

/// (key, (original filepath, compressed bytes))
type Asset = (AssetKey, (PathBuf, PathBuf));

/// All possible errors while reading and compressing an [`EmbeddedAssets`] directory
#[derive(Debug, Error)]
#[non_exhaustive]
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

  #[error("failed to create hex from bytes because {0}")]
  Hex(std::fmt::Error),

  #[error("invalid prefix {prefix} used while including path {path}")]
  PrefixInvalid { prefix: PathBuf, path: PathBuf },

  #[error("failed to walk directory {path} because {error}")]
  Walkdir {
    path: PathBuf,
    error: walkdir::Error,
  },

  #[error("OUT_DIR env var is not set, do you have a build script?")]
  OutDir,

  #[error("version error: {0}")]
  Version(#[from] semver::Error),
}

/// Represent a directory of assets that are compressed and embedded.
///
/// This is the compile time generation of [`tauri_utils::assets::Assets`] from a directory. Assets
/// from the directory are added as compiler dependencies by dummy including the original,
/// uncompressed assets.
///
/// The assets are compressed during this runtime, and can only be represented as a [`TokenStream`]
/// through [`ToTokens`]. The generated code is meant to be injected into an application to include
/// the compressed assets in that application's binary.
#[derive(Default)]
pub struct EmbeddedAssets {
  assets: HashMap<AssetKey, (PathBuf, PathBuf)>,
  csp_hashes: CspHashes,
}

pub struct EmbeddedAssetsInput(Vec<PathBuf>);

impl From<PathBuf> for EmbeddedAssetsInput {
  fn from(path: PathBuf) -> Self {
    Self(vec![path])
  }
}

impl From<Vec<PathBuf>> for EmbeddedAssetsInput {
  fn from(paths: Vec<PathBuf>) -> Self {
    Self(paths)
  }
}

/// Holds a list of (prefix, entry)
struct RawEmbeddedAssets {
  paths: Vec<(PathBuf, DirEntry)>,
  csp_hashes: CspHashes,
}

impl RawEmbeddedAssets {
  /// Creates a new list of (prefix, entry) from a collection of inputs.
  fn new(input: EmbeddedAssetsInput, options: &AssetOptions) -> Result<Self, EmbeddedAssetsError> {
    let mut csp_hashes = CspHashes::default();

    input
      .0
      .into_iter()
      .flat_map(|path| {
        let prefix = if path.is_dir() {
          path.clone()
        } else {
          path
            .parent()
            .expect("embedded file asset has no parent")
            .to_path_buf()
        };

        WalkDir::new(&path)
          .follow_links(true)
          .contents_first(true)
          .into_iter()
          .map(move |entry| (prefix.clone(), entry))
      })
      .filter_map(|(prefix, entry)| {
        match entry {
          // we only serve files, not directory listings
          Ok(entry) if entry.file_type().is_dir() => None,

          // compress all files encountered
          Ok(entry) => {
            if let Err(error) = csp_hashes
              .add_if_applicable(&entry, &options.dangerous_disable_asset_csp_modification)
            {
              Some(Err(error))
            } else {
              Some(Ok((prefix, entry)))
            }
          }

          // pass down error through filter to fail when encountering any error
          Err(error) => Some(Err(EmbeddedAssetsError::Walkdir {
            path: prefix,
            error,
          })),
        }
      })
      .collect::<Result<Vec<(PathBuf, DirEntry)>, _>>()
      .map(|paths| Self { paths, csp_hashes })
  }
}

/// Holds all hashes that we will apply on the CSP tag/header.
#[derive(Debug, Default)]
pub struct CspHashes {
  /// Scripts that are part of the asset collection (JS or MJS files).
  pub(crate) scripts: Vec<String>,
  /// Inline scripts (`<script>code</script>`). Maps a HTML path to a list of hashes.
  pub(crate) inline_scripts: HashMap<String, Vec<String>>,
  /// A list of hashes of the contents of all `style` elements.
  pub(crate) styles: Vec<String>,
}

impl CspHashes {
  /// Only add a CSP hash to the appropriate category if we think the file matches
  ///
  /// Note: this only checks the file extension, much like how a browser will assume a .js file is
  /// a JavaScript file unless HTTP headers tell it otherwise.
  pub fn add_if_applicable(
    &mut self,
    entry: &DirEntry,
    dangerous_disable_asset_csp_modification: &DisabledCspModificationKind,
  ) -> Result<(), EmbeddedAssetsError> {
    let path = entry.path();

    // we only hash JavaScript files for now, may expand to other CSP hashable types in the future
    if let Some("js") | Some("mjs") = path.extension().and_then(|os| os.to_str()) {
      if dangerous_disable_asset_csp_modification.can_modify("script-src") {
        let mut hasher = Sha256::new();
        hasher.update(
          &std::fs::read(path).map_err(|error| EmbeddedAssetsError::AssetRead {
            path: path.to_path_buf(),
            error,
          })?,
        );
        let hash = hasher.finalize();
        self
          .scripts
          .push(format!("'sha256-{}'", base64::encode(hash)));
      }
    }

    Ok(())
  }
}

/// Options used to embed assets.
#[derive(Default)]
pub struct AssetOptions {
  pub(crate) csp: bool,
  pub(crate) pattern: PatternKind,
  pub(crate) freeze_prototype: bool,
  pub(crate) dangerous_disable_asset_csp_modification: DisabledCspModificationKind,
  #[cfg(feature = "isolation")]
  pub(crate) isolation_schema: String,
}

impl AssetOptions {
  /// Creates the default asset options.
  pub fn new(pattern: PatternKind) -> Self {
    Self {
      csp: false,
      pattern,
      freeze_prototype: false,
      dangerous_disable_asset_csp_modification: DisabledCspModificationKind::Flag(false),
      #[cfg(feature = "isolation")]
      isolation_schema: format!("isolation-{}", uuid::Uuid::new_v4()),
    }
  }

  /// Instruct the asset handler to inject the CSP token to HTML files (Linux only) and add asset nonces and hashes to the policy.
  #[must_use]
  pub fn with_csp(mut self) -> Self {
    self.csp = true;
    self
  }

  /// Instruct the asset handler to include a script to freeze the `Object.prototype` on all HTML files.
  #[must_use]
  pub fn freeze_prototype(mut self, freeze: bool) -> Self {
    self.freeze_prototype = freeze;
    self
  }

  /// Instruct the asset handler to **NOT** modify the CSP. This is **NOT** recommended.
  pub fn dangerous_disable_asset_csp_modification(
    mut self,
    dangerous_disable_asset_csp_modification: DisabledCspModificationKind,
  ) -> Self {
    self.dangerous_disable_asset_csp_modification = dangerous_disable_asset_csp_modification;
    self
  }
}

impl EmbeddedAssets {
  /// Compress a collection of files and directories, ready to be generated into [`Assets`].
  ///
  /// [`Assets`]: tauri_utils::assets::Assets
  pub fn new(
    input: impl Into<EmbeddedAssetsInput>,
    options: &AssetOptions,
    map: impl Fn(&AssetKey, &Path, &mut Vec<u8>, &mut CspHashes) -> Result<(), EmbeddedAssetsError>,
  ) -> Result<Self, EmbeddedAssetsError> {
    // we need to pre-compute all files now, so that we can inject data from all files into a few
    let RawEmbeddedAssets { paths, csp_hashes } = RawEmbeddedAssets::new(input.into(), options)?;

    struct CompressState {
      csp_hashes: CspHashes,
      assets: HashMap<AssetKey, (PathBuf, PathBuf)>,
    }

    let CompressState { assets, csp_hashes } = paths.into_iter().try_fold(
      CompressState {
        csp_hashes,
        assets: HashMap::new(),
      },
      move |mut state, (prefix, entry)| {
        let (key, asset) = Self::compress_file(&prefix, entry.path(), &map, &mut state.csp_hashes)?;
        state.assets.insert(key, asset);
        Result::<_, EmbeddedAssetsError>::Ok(state)
      },
    )?;

    Ok(Self { assets, csp_hashes })
  }

  /// Use highest compression level for release, the fastest one for everything else
  #[cfg(feature = "compression")]
  fn compression_settings() -> BrotliEncoderParams {
    let mut settings = BrotliEncoderParams::default();

    // the following compression levels are hand-picked and are not min-maxed.
    // they have a good balance of runtime vs size for the respective profile goals.
    // see the "brotli" section of this comment https://github.com/tauri-apps/tauri/issues/3571#issuecomment-1054847558
    if cfg!(debug_assertions) {
      settings.quality = 2
    } else {
      settings.quality = 9
    }

    settings
  }

  /// Compress a file and spit out the information in a [`HashMap`] friendly form.
  fn compress_file(
    prefix: &Path,
    path: &Path,
    map: &impl Fn(&AssetKey, &Path, &mut Vec<u8>, &mut CspHashes) -> Result<(), EmbeddedAssetsError>,
    csp_hashes: &mut CspHashes,
  ) -> Result<Asset, EmbeddedAssetsError> {
    let mut input = std::fs::read(path).map_err(|error| EmbeddedAssetsError::AssetRead {
      path: path.to_owned(),
      error,
    })?;

    // get a key to the asset path without the asset directory prefix
    let key = path
      .strip_prefix(prefix)
      .map(AssetKey::from) // format the path for use in assets
      .map_err(|_| EmbeddedAssetsError::PrefixInvalid {
        prefix: prefix.to_owned(),
        path: path.to_owned(),
      })?;

    // perform any caller-requested input manipulation
    map(&key, path, &mut input, csp_hashes)?;

    // we must canonicalize the base of our paths to allow long paths on windows
    let out_dir = std::env::var("OUT_DIR")
      .map_err(|_| EmbeddedAssetsError::OutDir)
      .map(PathBuf::from)
      .and_then(|p| p.canonicalize().map_err(|_| EmbeddedAssetsError::OutDir))
      .map(|p| p.join(TARGET_PATH))?;

    // make sure that our output directory is created
    std::fs::create_dir_all(&out_dir).map_err(|_| EmbeddedAssetsError::OutDir)?;

    // get a hash of the input - allows for caching existing files
    let hash = {
      let mut hasher = crate::vendor::blake3_reference::Hasher::default();
      hasher.update(&input);

      let mut bytes = [0u8; 32];
      hasher.finalize(&mut bytes);

      let mut hex = String::with_capacity(2 * bytes.len());
      for b in bytes {
        write!(hex, "{:02x}", b).map_err(EmbeddedAssetsError::Hex)?;
      }
      hex
    };

    // use the content hash to determine filename, keep extensions that exist
    let out_path = if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
      out_dir.join(format!("{}.{}", hash, ext))
    } else {
      out_dir.join(hash)
    };

    // only compress and write to the file if it doesn't already exist.
    if !out_path.exists() {
      #[allow(unused_mut)]
      let mut out_file =
        File::create(&out_path).map_err(|error| EmbeddedAssetsError::AssetWrite {
          path: out_path.clone(),
          error,
        })?;

      #[cfg(not(feature = "compression"))]
      {
        use std::io::Write;
        out_file
          .write_all(&input)
          .map_err(|error| EmbeddedAssetsError::AssetWrite {
            path: path.to_owned(),
            error,
          })?;
      }

      #[cfg(feature = "compression")]
      {
        let mut input = std::io::Cursor::new(input);
        // entirely write input to the output file path with compression
        brotli::BrotliCompress(&mut input, &mut out_file, &Self::compression_settings()).map_err(
          |error| EmbeddedAssetsError::AssetWrite {
            path: path.to_owned(),
            error,
          },
        )?;
      }
    }

    Ok((key, (path.into(), out_path)))
  }
}

impl ToTokens for EmbeddedAssets {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let mut assets = TokenStream::new();
    for (key, (input, output)) in &self.assets {
      let key: &str = key.as_ref();
      let input = input.display().to_string();
      let output = output.display().to_string();

      // add original asset as a compiler dependency, rely on dead code elimination to clean it up
      assets.append_all(quote!(#key => {
        const _: &[u8] = include_bytes!(#input);
        include_bytes!(#output)
      },));
    }

    let mut global_hashes = TokenStream::new();
    for script_hash in &self.csp_hashes.scripts {
      let hash = script_hash.as_str();
      global_hashes.append_all(quote!(CspHash::Script(#hash),));
    }

    for style_hash in &self.csp_hashes.styles {
      let hash = style_hash.as_str();
      global_hashes.append_all(quote!(CspHash::Style(#hash),));
    }

    let mut html_hashes = TokenStream::new();
    for (path, hashes) in &self.csp_hashes.inline_scripts {
      let key = path.as_str();
      let mut value = TokenStream::new();
      for script_hash in hashes {
        let hash = script_hash.as_str();
        value.append_all(quote!(CspHash::Script(#hash),));
      }
      html_hashes.append_all(quote!(#key => &[#value],));
    }

    // we expect phf related items to be in path when generating the path code
    tokens.append_all(quote! {{
        use ::tauri::utils::assets::{CspHash, EmbeddedAssets, phf, phf::phf_map};
        EmbeddedAssets::new(phf_map! { #assets }, &[#global_hashes], phf_map! { #html_hashes })
    }});
  }
}
