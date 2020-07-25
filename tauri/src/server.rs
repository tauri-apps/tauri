use std::io::Read;
use tauri_api::assets::{AssetFetch, Assets};
use tiny_http::{Response, StatusCode};

/// Returns the HTTP response of the given asset path.
pub fn asset_response(path: &str, assets: &'static Assets) -> Response<impl Read> {
  let (asset, _) = assets
    .get(path, AssetFetch::Compress)
    .unwrap_or_else(|| panic!("Could not read asset {}", path));

  let mut headers = Vec::new();

  // Content-Encoding
  const BROTLI_HEADER: &str = "Content-Encoding: br";
  let content_encoding = BROTLI_HEADER
    .parse()
    .unwrap_or_else(|_| panic!("Could not add {} header", BROTLI_HEADER));
  headers.push(content_encoding);

  // Content-Type
  let mime = if path.ends_with(".svg") {
    "Content-Type: image/svg+xml"
  } else if path.ends_with(".css") {
    "Content-Type: text/css"
  } else if path.ends_with(".html") {
    "Content-Type: text/html"
  } else if path.ends_with(".js") {
    "Content-Type: text/javascript"
  } else {
    "Content-Type: application/octet-stream"
  };

  let content_type = mime
    .parse()
    .unwrap_or_else(|_| panic!("Could not add {} header", mime));
  headers.push(content_type);

  Response::new(StatusCode(200), headers, asset, None, None)
}
