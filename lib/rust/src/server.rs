use tiny_http::{Header, Response};

include!(concat!(env!("OUT_DIR"), "/data.rs"));

pub fn asset_response(path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
  let asset = ASSETS
    .get(&format!("{}{}", env!("TAURI_DIST_DIR"), path))
    .unwrap()
    .into_owned();
  let mut response = Response::from_data(asset);
  let mut header = ();

  if path.ends_with(".svg") {
    header = Header::from_bytes(&b"Content-Type"[..], &b"image/svg+xml"[..]).unwrap();
  } else if path.ends_with(".css") {
    header = Header::from_bytes(&b"Content-Type"[..], &b"text/css"[..]).unwrap();
  } else if path.ends_with(".html") {
    header = Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap();
  } else {
    header = Header::from_bytes(&b"Content-Type"[..], &b"application/octet-stream"[..]).unwrap();
  }

  response.add_header(header);

  response
}
