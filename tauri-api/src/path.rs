use std::path::PathBuf;
use serde_repr::{Serialize_repr, Deserialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Clone)]
#[repr(u16)]
pub enum BaseDirectory {
  Audio = 1,
  Cache,
  Config,
  Data,
  LocalData,
  Desktop,
  Document,
  Download,
  Executable,
  Font,
  Home,
  Picture,
  Public,
  Runtime,
  Template,
  Video,
  Resource,
  App,
}

pub fn resolve_path(path: String, dir: Option<BaseDirectory>) -> crate::Result<String> {
  if let Some(base_dir) = dir {
    let base_dir_path = match base_dir {
      BaseDirectory::Audio => audio_dir(),
      BaseDirectory::Cache => cache_dir(),
      BaseDirectory::Config => config_dir(),
      BaseDirectory::Data => data_dir(),
      BaseDirectory::LocalData => local_data_dir(),
      BaseDirectory::Desktop => desktop_dir(),
      BaseDirectory::Document => document_dir(),
      BaseDirectory::Download => download_dir(),
      BaseDirectory::Executable => executable_dir(),
      BaseDirectory::Font => font_dir(),
      BaseDirectory::Home => home_dir(),
      BaseDirectory::Picture => picture_dir(),
      BaseDirectory::Public => public_dir(),
      BaseDirectory::Runtime => runtime_dir(),
      BaseDirectory::Template => template_dir(),
      BaseDirectory::Video => video_dir(),
      BaseDirectory::Resource => resource_dir(),
      BaseDirectory::App => app_dir(),
    };
    if let Some(mut base_dir_path_value) = base_dir_path {
      base_dir_path_value.push(path);
      Ok(base_dir_path_value.to_string_lossy().to_string())
    } else {
      Err(crate::Error::from(crate::ErrorKind::Path("unable to determine base dir path".to_string())))
    }
  } else {
    Ok(path)
  }
}

// Returns the path to the user's audio directory.
pub fn audio_dir() -> Option<PathBuf> {
  dirs::audio_dir()
}

// Returns the path to the user's cache directory.
pub fn cache_dir() -> Option<PathBuf> {
  dirs::cache_dir()
}

// Returns the path to the user's config directory.
pub fn config_dir() -> Option<PathBuf> {
  dirs::config_dir()
}

// Returns the path to the user's data directory.
pub fn data_dir() -> Option<PathBuf> {
  dirs::data_dir()
}

// Returns the path to the user's local data directory.
pub fn local_data_dir() -> Option<PathBuf> {
  dirs::data_local_dir()
}

// Returns the path to the user's desktop directory.
pub fn desktop_dir() -> Option<PathBuf> {
  dirs::desktop_dir()
}

// Returns the path to the user's document directory.
pub fn document_dir() -> Option<PathBuf> {
  dirs::document_dir()
}

// Returns the path to the user's download directory.
pub fn download_dir() -> Option<PathBuf> {
  dirs::download_dir()
}

// Returns the path to the user's executable directory.
pub fn executable_dir() -> Option<PathBuf> {
  dirs::executable_dir()
}

// Returns the path to the user's font directory.
pub fn font_dir() -> Option<PathBuf> {
  dirs::font_dir()
}

// Returns the path to the user's home directory.
pub fn home_dir() -> Option<PathBuf> {
  dirs::home_dir()
}

// Returns the path to the user's picture directory.
pub fn picture_dir() -> Option<PathBuf> {
  dirs::picture_dir()
}

// Returns the path to the user's public directory.
pub fn public_dir() -> Option<PathBuf> {
  dirs::public_dir()
}

// Returns the path to the user's runtime directory.
pub fn runtime_dir() -> Option<PathBuf> {
  dirs::runtime_dir()
}

// Returns the path to the user's template directory.
pub fn template_dir() -> Option<PathBuf> {
  dirs::template_dir()
}

// Returns the path to the user's video dir
pub fn video_dir() -> Option<PathBuf> {
  dirs::video_dir()
}

pub fn resource_dir() -> Option<PathBuf> {
  crate::platform::resource_dir().ok()
}

fn app_name() -> crate::Result<String> {
  let exe = std::env::current_exe()?;
  let exe_dir = exe.parent().expect("failed to get current exe parent");
  let app_name = exe_dir
    .file_name().expect("failed to get exe_dir filename")
    .to_string_lossy();

  Ok(app_name.to_string())
}

pub fn app_dir() -> Option<PathBuf> {
  dirs::config_dir()
    .and_then(|mut dir| {
      if let Ok(app_name) = app_name() {
        dir.push(app_name);
        Some(dir)
      } else {
        None
      }
    })
}
