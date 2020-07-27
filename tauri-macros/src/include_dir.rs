use crate::error::Error;
use proc_macro2::TokenStream;
use quote::quote;
use quote::TokenStreamExt;
use std::collections::{HashMap, HashSet};
use std::env::var;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use tauri_api::assets::{AssetCompression, Assets};
use walkdir::WalkDir;

enum Asset {
  Identity(PathBuf),
  Compressed(PathBuf, PathBuf),
}

pub(crate) struct IncludeDir {
  assets: HashMap<String, Asset>,
  filter: HashSet<String>,
  prefix: PathBuf,
}

impl IncludeDir {
  pub fn new(prefix: impl Into<PathBuf>) -> Self {
    Self {
      assets: HashMap::new(),
      filter: HashSet::new(),
      prefix: prefix.into(),
    }
  }

  /// get a relative path based on the `IncludeDir`'s prefix
  fn relative<'p>(&self, path: &'p Path) -> Result<&'p Path, Error> {
    path
      .strip_prefix(&self.prefix)
      .map_err(|_| Error::IncludeDirPrefix)
  }

  pub fn file(mut self, path: impl Into<PathBuf>, comp: AssetCompression) -> Result<Self, Error> {
    let path = path.into();
    let relative = self.relative(&path)?;
    let key = Assets::format_key(&relative);

    let asset = match comp {
      AssetCompression::None => Asset::Identity(path),
      AssetCompression::Brotli => {
        let cache = var("OUT_DIR")
          .map_err(|_| Error::EnvOutDir)
          .map(|out| Path::new(&out).join(".tauri-assets").join(relative))
          .and_then(|mut out| {
            let filename = out.file_name().ok_or(Error::IncludeDirEmptyFilename)?;
            let filename = format!("{}.br", filename.to_string_lossy());
            out.set_file_name(&filename);
            Ok(out)
          })?;

        // make sure the parent directory is created
        let cache_parent = cache.parent().ok_or(Error::IncludeDirCacheDir)?;
        create_dir_all(&cache_parent).map_err(|e| Error::Io(cache_parent.into(), e))?;

        // open original asset path
        let reader = File::open(&path).map_err(|e| Error::Io(path.to_path_buf(), e))?;
        let mut reader = BufReader::new(reader);

        // open cache path
        let writer = File::create(&cache).map_err(|e| Error::Io(cache.to_path_buf(), e))?;
        let mut writer = BufWriter::new(writer);

        let _ = brotli::BrotliCompress(&mut reader, &mut writer, &Default::default())
          .map_err(|e| Error::Io(cache.to_path_buf(), e))?;

        Asset::Compressed(path, cache)
      }
    };

    self.assets.insert(key, asset);
    Ok(self)
  }

  pub fn dir(mut self, path: impl AsRef<Path>, comp: AssetCompression) -> Result<Self, Error> {
    let path = path.as_ref();
    let walker = WalkDir::new(&path).follow_links(true);
    for entry in walker.into_iter() {
      match entry {
        Ok(e) => {
          if !e.file_type().is_dir() {
            self = self.file(e.path(), comp)?
          }
        }
        Err(e) => return Err(Error::Io(path.into(), e.into())),
      }
    }
    Ok(self)
  }

  /// Set list of files to not embed. Paths should be relative to the dist dir
  pub fn set_filter(mut self, filter: HashSet<PathBuf>) -> Result<Self, Error> {
    self.filter = filter
      .iter()
      .map(|path| {
        let path = if path.starts_with(&self.prefix) {
          self.relative(path)?
        } else {
          &path
        };
        Ok(Assets::format_key(path))
      })
      .collect::<Result<_, _>>()?;

    Ok(self)
  }

  pub fn build(self) -> Result<TokenStream, Error> {
    let mut matches = TokenStream::new();
    for (key, asset) in self.assets {
      if self.filter.contains(&key) {
        continue;
      }

      let value = match asset {
        Asset::Identity(path) => {
          let path = path.display().to_string();
          quote! {
            (AssetCompression::None, include_bytes!(#path))
          }
        }
        Asset::Compressed(path, cache) => {
          let path = path.display().to_string();
          let cache = cache.display().to_string();
          quote! {
            {
              // make compiler check asset file for re-run.
              // rely on dead code elimination to remove it from target binary
              const _: &[u8] = include_bytes!(#path);

              (AssetCompression::Brotli, include_bytes!(#cache))
            }
          }
        }
      };

      matches.append_all(quote! {
        #key => #value,
      })
    }

    Ok(quote! {
      phf_map! {
        #matches
      }
    })
  }
}
