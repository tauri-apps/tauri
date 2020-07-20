use crate::error::Error;
use proc_macro2::TokenStream;
use quote::quote;
use quote::TokenStreamExt;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use tauri_api::assets::Compression;
use walkdir::WalkDir;

fn to_key(path: &Path, prefix: &Path) -> Result<String, Error> {
  // strip the prefix to remove the manifest + dist dir
  let path = path
    .strip_prefix(prefix)
    .map_err(|_| Error::IncludeDirPrefix)?;

  // add in root to mimic how it is used from a server url
  let path = if path.has_root() {
    Cow::Borrowed(path)
  } else {
    Cow::Owned(Path::new(&Component::RootDir).join(path))
  };

  #[cfg(not(windows))]
  let path = path.to_string_lossy().to_string();

  // change windows type paths to the unix counterparts
  #[cfg(windows)]
  let path = {
    let mut buf = String::new();
    for component in path.components() {
      match component {
        Component::RootDir => buf.push('/'),
        Component::CurDir => buf.push_str("./"),
        Component::ParentDir => buf.push_str("../"),
        Component::Prefix(prefix) => buf.push_str(&prefix.as_os_str().to_string_lossy()),
        Component::Normal(s) => {
          buf.push_str(&s.to_string_lossy());
          buf.push('/')
        }
      }
    }

    // remove the last slash
    if buf != "/" {
      buf.pop();
    }

    buf
  };

  Ok(path)
}

pub(crate) struct IncludeDir {
  files: HashMap<String, (Compression, PathBuf)>,
  filter: HashSet<String>,
  prefix: PathBuf,
}

impl IncludeDir {
  pub fn new(prefix: impl Into<PathBuf>) -> Self {
    Self {
      files: HashMap::new(),
      filter: HashSet::new(),
      prefix: prefix.into(),
    }
  }

  pub fn file(mut self, path: impl Into<PathBuf>, comp: Compression) -> Result<Self, Error> {
    let path = path.into();
    let key = to_key(&path, &self.prefix)?;

    match comp {
      Compression::None => self.files.insert(key, (comp, path)),
      Compression::Gzip => todo!(),
    };

    Ok(self)
  }

  pub fn dir(mut self, path: impl AsRef<Path>, comp: Compression) -> Result<Self, Error> {
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

  pub fn set_filter(mut self, filter: HashSet<String>) -> Self {
    self.filter = filter;
    self
  }

  pub fn build(self) -> Result<TokenStream, Error> {
    let mut matches = TokenStream::new();
    for (name, (compression, include)) in &self.files {
      let include = include.display().to_string();
      if self.filter.contains(&include) {
        continue;
      }

      let comp = match compression {
        Compression::None => quote! {::tauri::api::assets::Compression::None},
        Compression::Gzip => quote! {::tauri::api::assets::Compression::Gzip},
      };

      matches.append_all(quote! {
        #name => (#comp, include_bytes!(#include)),
      })
    }

    Ok(quote! {
      ::tauri::api::assets::Assets {
        files: phf_map! {
          #matches
        }
      }
    })
  }
}
