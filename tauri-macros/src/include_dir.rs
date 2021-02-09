use crate::error::Error;
use flate2::bufread::GzEncoder;
use proc_macro2::TokenStream;
use quote::quote;
use quote::TokenStreamExt;
use std::collections::{HashMap, HashSet};
use std::env::var;
use std::fs::{canonicalize, create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use tauri_utils::assets::{AssetCompression, Assets};
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
      AssetCompression::Gzip => {
        let cache = var("OUT_DIR")
          .map_err(|_| Error::EnvOutDir)
          .and_then(|out| canonicalize(&out).map_err(|e| Error::Io(PathBuf::from(out), e)))
          .map(|out| out.join(".tauri-assets"))?;

        // normalize path separators
        let relative: PathBuf = relative.components().collect();
        let cache = cache.join(relative);

        // append .br extension to filename
        let filename = cache.file_name().ok_or(Error::IncludeDirEmptyFilename)?;
        let filename = format!("{}.br", filename.to_string_lossy());

        // remove filename from cache
        let cache = cache.parent().ok_or(Error::IncludeDirCacheDir)?;

        // append the filename to the canonical path
        let cache_file = cache.join(filename);

        // make sure the cache directory is created
        create_dir_all(&cache).map_err(|e| Error::Io(cache.to_path_buf(), e))?;

        // open original asset path
        let reader = File::open(&path).map_err(|e| Error::Io(path.to_path_buf(), e))?;
        let reader = BufReader::new(reader);
        let mut reader = GzEncoder::new(reader, flate2::Compression::best());

        // open cache path
        let writer =
          File::create(&cache_file).map_err(|e| Error::Io(cache_file.to_path_buf(), e))?;
        let mut writer = BufWriter::new(writer);

        std::io::copy(&mut reader, &mut writer).map_err(|e| Error::Io(path.to_path_buf(), e))?;

        Asset::Compressed(path, cache_file)
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

              (AssetCompression::Gzip, include_bytes!(#cache))
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
