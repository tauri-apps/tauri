use web_view::WebView;

use tauri_api::dir;
use tauri_api::file;

use std::fs::File;
use std::io::Write;

pub fn list<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      dir::walk_dir(path.to_string())
        .map_err(|e| crate::ErrorKind::Command(e.to_string()).into())
        .and_then(|f| serde_json::to_string(&f).map_err(|err| err.into()))
    },
    callback,
    error,
  );
}

pub fn list_dirs<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      dir::list_dir_contents(&path)
        .map_err(|e| crate::ErrorKind::Command(e.to_string()).into())
        .and_then(|f| serde_json::to_string(&f).map_err(|err| err.into()))
    },
    callback,
    error,
  );
}

pub fn write_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  file: String,
  contents: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      File::create(file)
        .map_err(|e| crate::ErrorKind::Command(e.to_string()).into())
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

pub fn read_text_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      file::read_string(path)
        .map_err(|e| crate::ErrorKind::Command(e.to_string()).into())
        .and_then(|f| {
          serde_json::to_string(&f)
            .map_err(|err| err.into())
            .map(|s| s.to_string())
        })
    },
    callback,
    error,
  );
}

pub fn read_binary_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      file::read_binary(path)
        .map_err(|e| crate::ErrorKind::Command(e.to_string()).into())
        .and_then(|f| {
          serde_json::to_string(&f)
            .map_err(|err| err.into())
            .map(|s| s.to_string())
        })
    },
    callback,
    error,
  );
}

// test webview functionality.
#[cfg(test)]
mod test {
  use super::*;
  use web_view::*;

  // create a makeshift webview
  fn create_test_webview() -> crate::Result<WebView<'static, ()>> {
    // basic html set into webview
    let content = r#"<html><head></head><body></body></html>"#;

    Ok(
      // use webview builder to create simple webview
      WebViewBuilder::new()
        .title("test")
        .size(800, 800)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_wv, _arg| Ok(()))
        .content(Content::Html(content))
        .build()?,
    )
  }

  #[test]
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
    std::thread::sleep(std::time::Duration::from_millis(500));

    // read from the file.
    let data = read_to_string(path)?;

    // check that the file contents is equal to the expected contents.
    assert_eq!(data, contents);

    Ok(())
  }
}
