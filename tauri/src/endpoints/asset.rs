use crate::{Context, ApplicationDispatcherExt};
use std::io::Read;
use tauri_api::assets::{AssetFetch, Assets};

#[allow(clippy::option_env_unwrap)]
pub async fn load<D: ApplicationDispatcherExt + 'static>(
  dispatcher: &mut D,
  asset: String,
  asset_type: String,
  callback: String,
  error: String,
  ctx: &Context,
) {
  let mut dispatcher_ = dispatcher.clone();
  let assets = ctx.assets;
  let public_path = ctx.config.tauri.embedded_server.public_path.clone();
  crate::execute_promise(
    dispatcher,
    async move {
      // strip "about:" uri scheme if it exists
      let asset = if asset.starts_with("about:") {
        &asset[6..]
      } else {
        &asset
      };

      // handle public path setting from tauri.conf > tauri > embeddedServer > publicPath
      let asset = if asset.starts_with(&public_path) {
        &asset[public_path.len() - 1..]
      } else {
        eprintln!(
          "found url not matching public path.\nasset url: {}\npublic path: {}",
          asset, public_path
        );
        asset
      }
      .to_string();

      // how should that condition be handled now?
      let asset_bytes = assets
        .get(&Assets::format_key(&asset), AssetFetch::Decompress)
        .ok_or_else(|| anyhow::anyhow!("Asset '{}' not found", asset))
        .and_then(|(read, _)| {
          read
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .map_err(Into::into)
        })?;

      if asset_type == "image" {
        let mime_type = if asset.ends_with("gif") {
          "gif"
        } else if asset.ends_with("apng") {
          "apng"
        } else if asset.ends_with("png") {
          "png"
        } else if asset.ends_with("avif") {
          "avif"
        } else if asset.ends_with("webp") {
          "webp"
        } else if asset.ends_with("svg") {
          "svg+xml"
        } else {
          "jpeg"
        };
        Ok(format!(
          r#""data:image/{};base64,{}""#,
          mime_type,
          base64::encode(&asset_bytes)
        ))
      } else {
        let asset_str =
          std::str::from_utf8(&asset_bytes).expect("failed to convert asset bytes to u8 slice");
        if asset_type == "stylesheet" {
          dispatcher_.eval(&format!(
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
            // Escape octal sequences, which aren't allowed in template literals
            css = asset_str.replace("\\", "\\\\").as_str()
          ));
        } else {
          dispatcher_.eval(asset_str);
        }
        Ok("Asset loaded successfully".to_string())
      }
    },
    callback,
    error,
  )
  .await;
}
