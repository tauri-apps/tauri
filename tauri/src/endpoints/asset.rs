use std::io::Read;
use tauri_api::assets::{AssetFetch, Assets};
use webview_official::Webview;

pub fn load(
  webview: &mut Webview<'_>,
  asset: String,
  asset_type: String,
  callback: String,
  error: String,
  assets: &'static tauri_api::assets::Assets,
) {
  let mut webview_mut = webview.as_mut();
  crate::execute_promise(
    webview,
    move || {
      // strip "about:" uri scheme if it exists
      let asset = if asset.starts_with("about:") {
        &asset[6..]
      } else {
        &asset
      };

      // TODO: previously would strip away parents to try and handle webpack public path
      // how should that condition be handled now?
      let asset_bytes = assets
        .get(&Assets::format_key(asset), AssetFetch::Decompress)
        .ok_or_else(|| anyhow::anyhow!("Asset '{}' not found", asset))
        .and_then(|(read, _)| {
          read
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .map_err(Into::into)
        })?;

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
          base64::encode(&asset_bytes)
        ))
      } else {
        webview_mut.dispatch(move |webview_ref| {
          let asset_str =
            std::str::from_utf8(&asset_bytes).expect("failed to convert asset bytes to u8 slice");
          if asset_type == "stylesheet" {
            webview_ref.eval(&format!(
              r#"
                (function (content) {{
                  var css = document.createElement('style')
                  css.type = 'text/css'
                  if (css.styleSheet)
                      css.styleSheet.cssText = content
                  else
                      css.appendChild(document.createTextNode(content))
                  document.getElementsByTagName("head")[0].appendChild(css);
                }})(`{css}`)
              "#,
              css = asset_str
            ));
          } else {
            webview_ref.eval(asset_str);
          }
        })?;
        Ok("Asset loaded successfully".to_string())
      }
    },
    callback,
    error,
  );
}
