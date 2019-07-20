use ffi::{self, DialogFlags, DialogType};
use std::{ffi::CString, path::PathBuf};
use super::{read_str, WVResult, WebView};

const STR_BUF_SIZE: usize = 4096;

/// A builder for opening a new dialog window.
#[derive(Debug)]
pub struct DialogBuilder<'a: 'b, 'b, T: 'a> {
  webview: &'b mut WebView<'a, T>,
}

impl<'a: 'b, 'b, T: 'a> DialogBuilder<'a, 'b, T> {
  /// Creates a new dialog builder for a WebView.
  pub fn new(webview: &'b mut WebView<'a, T>) -> DialogBuilder<'a, 'b, T> {
    DialogBuilder { webview }
  }

  fn dialog(
    &mut self,
    title: String,
    arg: String,
    dialog_type: DialogType,
    dialog_flags: DialogFlags,
  ) -> WVResult<String> {
    let mut s = [0u8; STR_BUF_SIZE];

    let title_cstr = CString::new(title)?;
    let arg_cstr = CString::new(arg)?;

    unsafe {
      ffi::webview_dialog(
        self.webview.inner,
        dialog_type,
        dialog_flags,
        title_cstr.as_ptr(),
        arg_cstr.as_ptr(),
        s.as_mut_ptr() as _,
        s.len(),
      );
    }

    Ok(read_str(&s))
  }

  /// Opens a new open file dialog and returns the chosen file path.
  pub fn open_file<S, P>(&mut self, title: S, default_file: P) -> WVResult<Option<PathBuf>>
  where
    S: Into<String>,
    P: Into<PathBuf>,
  {
    self
      .dialog(
        title.into(),
        default_file.into().to_string_lossy().into_owned(),
        DialogType::Open,
        DialogFlags::FILE,
      )
      .map(|path| {
        if path.is_empty() {
          None
        } else {
          Some(PathBuf::from(path))
        }
      })
  }

  /// Opens a new choose directory dialog as returns the chosen directory path.
  pub fn choose_directory<S, P>(
    &mut self,
    title: S,
    default_directory: P,
  ) -> WVResult<Option<PathBuf>>
  where
    S: Into<String>,
    P: Into<PathBuf>,
  {
    self
      .dialog(
        title.into(),
        default_directory.into().to_string_lossy().into_owned(),
        DialogType::Open,
        DialogFlags::DIRECTORY,
      )
      .map(|path| {
        if path.is_empty() {
          None
        } else {
          Some(PathBuf::from(path))
        }
      })
  }

  /// Opens an info alert dialog.
  pub fn info<TS, MS>(&mut self, title: TS, message: MS) -> WVResult
  where
    TS: Into<String>,
    MS: Into<String>,
  {
    self
      .dialog(
        title.into(),
        message.into(),
        DialogType::Alert,
        DialogFlags::INFO,
      )
      .map(|_| ())
  }

  /// Opens a warning alert dialog.
  pub fn warning<TS, MS>(&mut self, title: TS, message: MS) -> WVResult
  where
    TS: Into<String>,
    MS: Into<String>,
  {
    self
      .dialog(
        title.into(),
        message.into(),
        DialogType::Alert,
        DialogFlags::WARNING,
      )
      .map(|_| ())
  }

  /// Opens an error alert dialog.
  pub fn error<TS, MS>(&mut self, title: TS, message: MS) -> WVResult
  where
    TS: Into<String>,
    MS: Into<String>,
  {
    self
      .dialog(
        title.into(),
        message.into(),
        DialogType::Alert,
        DialogFlags::ERROR,
      )
      .map(|_| ())
  }
}
