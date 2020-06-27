use web_view::WebView;

use tauri_api::dir;
use tauri_api::file;
use tauri_api::path::resolve_path;

use std::fs;
use std::fs::File;
use std::io::Write;

use super::cmd::{DirOperationOptions, FileOperationOptions};

/// Reads a directory.
#[cfg(read_dir)]
pub fn read_dir<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  options: Option<DirOperationOptions>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      let (recursive, dir) = if let Some(options_value) = options {
        (options_value.recursive, options_value.dir)
      } else {
        (false, None)
      };
      dir::read_dir(resolve_path(path, dir)?, recursive)
        .and_then(|f| serde_json::to_string(&f).map_err(|err| err.into()))
    },
    callback,
    error,
  );
}

/// Copies a file.
#[cfg(copy_file)]
pub fn copy_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  source: String,
  destination: String,
  options: Option<FileOperationOptions>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      let (src, dest) = match options.and_then(|o| o.dir) {
        Some(dir) => (
          resolve_path(source, Some(dir.clone()))?,
          resolve_path(destination, Some(dir))?,
        ),
        None => (source, destination),
      };
      fs::copy(src, dest)
        .map_err(|e| e.into())
        .map(|_| "".to_string())
    },
    callback,
    error,
  );
}

/// Creates a directory.
#[cfg(create_dir)]
pub fn create_dir<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  options: Option<DirOperationOptions>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      let (recursive, dir) = if let Some(options_value) = options {
        (options_value.recursive, options_value.dir)
      } else {
        (false, None)
      };
      let resolved_path = resolve_path(path, dir)?;
      let response = if recursive {
        fs::create_dir_all(resolved_path)
      } else {
        fs::create_dir(resolved_path)
      };

      response.map_err(|e| e.into()).map(|_| "".to_string())
    },
    callback,
    error,
  );
}

/// Removes a directory.
#[cfg(remove_dir)]
pub fn remove_dir<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  options: Option<DirOperationOptions>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      let (recursive, dir) = if let Some(options_value) = options {
        (options_value.recursive, options_value.dir)
      } else {
        (false, None)
      };
      let resolved_path = resolve_path(path, dir)?;
      let response = if recursive {
        fs::remove_dir_all(resolved_path)
      } else {
        fs::remove_dir(resolved_path)
      };

      response.map_err(|e| e.into()).map(|_| "".to_string())
    },
    callback,
    error,
  );
}

/// Removes a file
#[cfg(remove_file)]
pub fn remove_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  options: Option<FileOperationOptions>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      let resolved_path = resolve_path(path, options.and_then(|o| o.dir))?;
      fs::remove_file(resolved_path)
        .map_err(|e| e.into())
        .map(|_| "".to_string())
    },
    callback,
    error,
  );
}

/// Renames a file.
#[cfg(rename_file)]
pub fn rename_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  old_path: String,
  new_path: String,
  options: Option<FileOperationOptions>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      let (old, new) = match options.and_then(|o| o.dir) {
        Some(dir) => (
          resolve_path(old_path, Some(dir.clone()))?,
          resolve_path(new_path, Some(dir))?,
        ),
        None => (old_path, new_path),
      };
      fs::rename(old, new)
        .map_err(|e| e.into())
        .map(|_| "".to_string())
    },
    callback,
    error,
  );
}

/// Writes a text file.
#[cfg(write_file)]
pub fn write_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  file: String,
  contents: String,
  options: Option<FileOperationOptions>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      File::create(resolve_path(file, options.and_then(|o| o.dir))?)
        .map_err(|e| e.into())
        .and_then(|mut f| {
          f.write_all(contents.as_bytes())
            .map_err(|err| err.into())
            .map(|_| "".to_string())
        })
    },
    callback,
    error,
  );
}

/// Writes a binary file.
#[cfg(write_binary_file)]
pub fn write_binary_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  file: String,
  contents: String,
  options: Option<FileOperationOptions>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      base64::decode(contents)
        .map_err(|e| e.into())
        .and_then(|c| {
          File::create(resolve_path(file, options.and_then(|o| o.dir))?)
            .map_err(|e| e.into())
            .and_then(|mut f| {
              f.write_all(&c)
                .map_err(|err| err.into())
                .map(|_| "".to_string())
            })
        })
    },
    callback,
    error,
  );
}

/// Reads a text file.
#[cfg(read_text_file)]
pub fn read_text_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  options: Option<FileOperationOptions>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      file::read_string(resolve_path(path, options.and_then(|o| o.dir))?)
        .and_then(|f| serde_json::to_string(&f).map_err(|err| err.into()))
    },
    callback,
    error,
  );
}

/// Reads a binary file.
#[cfg(read_binary_file)]
pub fn read_binary_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  options: Option<FileOperationOptions>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      file::read_binary(resolve_path(path, options.and_then(|o| o.dir))?)
        .and_then(|f| serde_json::to_string(&f).map_err(|err| err.into()))
    },
    callback,
    error,
  );
}

// test webview functionality.
#[cfg(test)]
mod test {
  // use super::*;
  // use web_view::*;

  // create a makeshift webview
  // fn create_test_webview() -> crate::Result<WebView<'static, ()>> {
  //   // basic html set into webview
  //   let content = r#"<html><head></head><body></body></html>"#;

  //   Ok(
  //     // use webview builder to create simple webview
  //     WebViewBuilder::new()
  //       .title("test")
  //       .size(800, 800)
  //       .resizable(true)
  //       .debug(true)
  //       .user_data(())
  //       .invoke_handler(|_wv, _arg| Ok(()))
  //       .content(Content::Html(content))
  //       .build()?,
  //   )
  // }

  /* #[test]
  #[cfg(not(any(target_os = "linux", target_os = "macos")))]
  // test the file_write functionality
  fn test_write_to_file() -> crate::Result<()> {
    // import read_to_string and write to be able to manipulate the file.
    use std::fs::{read_to_string, write};

    // create the webview
    let mut webview = create_test_webview()?;

    // setup the contents and the path.
    let contents = String::from(r#"Write to the Test file"#);
    let path = String::from("test/fixture/test.txt");

    // clear the file by writing nothing to it.
    write(&path, "")?;

    //call write file with the path and contents.
    write_file(
      &mut webview,
      path.clone(),
      contents.clone(),
      String::from(""),
      String::from(""),
    );

    // sleep the main thread to wait for the promise to execute.
    std::thread::sleep(std::time::Duration::from_millis(200));

    // read from the file.
    let data = read_to_string(path)?;

    // check that the file contents is equal to the expected contents.
    assert_eq!(data, contents);

    Ok(())
  } */
}
