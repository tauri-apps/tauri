use std::path::PathBuf;
use webview_rust_sys::Webview;

pub fn load(
  webview: &mut Webview,
  asset: String,
  asset_type: String,
  callback: String,
  error: String,
) {
  let handle = webview.handle();
  crate::execute_promise(
    webview,
    move || {
      let mut path = PathBuf::from(if asset.starts_with('/') {
        asset.replacen("/", "", 1)
      } else {
        asset.clone()
      });
      let mut read_asset;
      loop {
        read_asset = crate::assets::ASSETS.get(&format!(
          "{}/{}",
          option_env!("TAURI_DIST_DIR")
            .expect("tauri apps should be built with the TAURI_DIST_DIR environment variable"),
          path.to_string_lossy()
        ));
        if read_asset.is_err() {
          match path.iter().next() {
            Some(component) => {
              let first_component = component.to_str().expect("failed to read path component");
              path = PathBuf::from(path.to_string_lossy().replacen(
                format!("{}/", first_component).as_str(),
                "",
                1,
              ));
            }
            None => {
              return Err(anyhow::anyhow!("Asset '{}' not found", asset));
            }
          }
        } else {
          break;
        }
      }

      if asset_type == "image" {
        let ext = if asset.ends_with("gif") {
          "gif"
        } else if asset.ends_with("png") {
          "png"
        } else {
          "jpeg"
        };
        Ok(format!(
          r#""data:image/{};base64,{}""#,
          ext,
          base64::encode(&read_asset.expect("Failed to read asset type").into_owned())
        ))
      } else {
        handle
          .dispatch(move |_webview| {
            let asset_bytes = &read_asset.expect("Failed to read asset type").into_owned();
            let asset_str =
              &std::str::from_utf8(asset_bytes).expect("failed to convert asset bytes to u8 slice");
            if asset_type == "stylesheet" {
              _webview.inject_css(asset_str)
            } else {
              _webview.eval(asset_str)
            }
          })
          .map_err(|err| err.into())
          .map(|_| "Asset loaded successfully".to_string())
      }
    },
    callback,
    error,
  );
}
