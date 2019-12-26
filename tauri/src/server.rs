use tiny_http::{Header, Response};

pub fn asset_response(path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
  let asset_path = &format!("{}{}", env!("TAURI_DIST_DIR"), path);
  let asset = crate::assets::ASSETS
    .get(asset_path)
    .expect(&format!("Could not read asset {}", asset_path))
    .into_owned();
  let mut response = Response::from_data(asset);
  let header;

  if path.ends_with(".svg") {
    header = Header::from_bytes(&b"Content-Type"[..], &b"image/svg+xml"[..])
      .expect("Could not add svg+xml header");
  } else if path.ends_with(".css") {
    header =
      Header::from_bytes(&b"Content-Type"[..], &b"text/css"[..]).expect("Could not add css header");
  } else if path.ends_with(".html") {
    header = Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..])
      .expect("Could not add html header");
  } else if path.ends_with(".js") {
    header = Header::from_bytes(&b"Content-Type"[..], &b"text/javascript"[..])
      .expect("Could not add Javascript header");
  } else {
    header = Header::from_bytes(&b"Content-Type"[..], &b"application/octet-stream"[..])
      .expect("Could not add octet-stream header");
  }

  response.add_header(header);

  response
}
