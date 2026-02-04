use gst::prelude::*;
use gstreamer as gst;
use gstreamer_check as gst_check;

fn init() {
  use std::sync::Once;
  static INIT: Once = Once::new();

  INIT.call_once(|| {
    gst::init().unwrap();
    gsttauriasset::plugin_register_static().unwrap();
  });
}

#[test]
fn test_uri_handler_interface() {
  init();
  let element = gst::ElementFactory::make("tauriasset")
    .build()
    .expect("Failed to create tauriasset element");

  let uri_handler = element
    .dynamic_cast::<gst::URIHandler>()
    .expect("Element should implement URIHandler");

  let protocols = uri_handler.protocols();
  assert!(!protocols.is_empty(), "Should have at least one protocol");
  assert!(
    protocols.contains(&"asset".into()),
    "Should support 'asset' protocol"
  );
}

#[test]
fn test_set_uri_basic() {
  init();
  let element = gst::ElementFactory::make("tauriasset")
    .build()
    .expect("Failed to create tauriasset element");

  let uri_handler = element
    .dynamic_cast::<gst::URIHandler>()
    .expect("Element should implement URIHandler");

  let uri = "asset:///tmp/test.mp3";
  let result = uri_handler.set_uri(uri);
  assert!(result.is_ok(), "Setting valid URI should succeed");
  assert_eq!(
    uri_handler.uri().unwrap(),
    uri,
    "URI should be set correctly"
  );
}

#[test]
fn test_set_uri_with_localhost() {
  init();
  let element = gst::ElementFactory::make("tauriasset")
    .build()
    .expect("Failed to create tauriasset element");

  let uri_handler = element
    .dynamic_cast::<gst::URIHandler>()
    .expect("Element should implement URIHandler");

  let result = uri_handler.set_uri("asset://localhost/tmp/test.mp3");
  assert!(result.is_ok(), "Setting URI with localhost should succeed");
  assert_eq!(
    uri_handler.uri().unwrap(),
    "asset:///tmp/test.mp3",
    "URI with localhost should be set correctly"
  );
}

#[test]
fn test_set_wrong_protocol_uri() {
  init();
  let element = gst::ElementFactory::make("tauriasset")
    .build()
    .expect("Failed to create tauriasset element");

  let uri_handler = element
    .dynamic_cast::<gst::URIHandler>()
    .expect("Element should implement URIHandler");

  let result = uri_handler.set_uri("invalid://test.mp3");
  assert_eq!(
    result.unwrap_err().kind::<gst::URIError>(),
    Some(gst::URIError::UnsupportedProtocol)
  );
}

#[test]
fn test_no_path() {
  init();
  let element = gst::ElementFactory::make("tauriasset")
    .build()
    .expect("Failed to create tauriasset element");

  let uri_handler = element
    .dynamic_cast::<gst::URIHandler>()
    .expect("Element should implement URIHandler");

  let result = uri_handler.set_uri("asset://");
  assert_eq!(
    result.unwrap_err().kind::<gst::URIError>(),
    Some(gst::URIError::BadUri)
  );
}

#[test]
fn test_set_uri_with_percent_encoding() {
  init();
  let element = gst::ElementFactory::make("tauriasset")
    .build()
    .expect("Failed to create tauriasset element");

  let uri_handler = element
    .dynamic_cast::<gst::URIHandler>()
    .expect("Element should implement URIHandler");

  let result = uri_handler.set_uri("asset:///tmp/test%20file.mp3");
  assert!(
    result.is_ok(),
    "Setting URI with percent-encoded characters should succeed"
  );

  assert_eq!(uri_handler.uri().unwrap(), "asset:///tmp/test file.mp3");
}

#[test]
fn test_read_file() {
  init();

  let audio_name = "test_mono_s16.flac";
  let expected_data = include_bytes!("test_mono_s16.flac");

  let mut path = std::env::current_dir().unwrap();
  path.push(format!("tests/{}", audio_name));
  let uri = format!("asset://{}", path.to_str().unwrap());

  let mut h = gst_check::Harness::new("tauriasset");

  let element = h.element().expect("Harness should have an element");
  let uri_handler = element
    .dynamic_cast_ref::<gst::URIHandler>()
    .expect("Element should implement URIHandler");

  uri_handler.set_uri(&uri).expect("Failed to set URI");

  h.play();

  let buffer = h.pull().expect("Failed to pull buffer");
  let map = buffer.map_readable().expect("Failed to map buffer");

  assert_eq!(&*map, expected_data);
}
