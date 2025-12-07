use gst::glib::subclass::types::ObjectSubclassExt;
use gst::prelude::{ElementExt, GhostPadExt, GstBinExt, PadExt};
use gst::subclass::prelude::{BinImpl, ElementImpl, ObjectImpl, URIHandlerImpl};
use gst::subclass::prelude::{GstObjectImpl, ObjectSubclass};
use gst::{glib, GhostPad, PadDirection};
use gstreamer_base::gst;

const ASSET_URI_SCHEME: &str = "asset";

#[derive(Default)]
pub struct TauriAsset {
  // uri: Option<String>,
}

impl TauriAsset {
  pub fn new() -> Self {
    Self::default()
  }
}

#[glib::object_subclass]
impl ObjectSubclass for TauriAsset {
  const NAME: &'static str = "GstTauriAsset";
  type Type = super::TauriAsset;
  type ParentType = gst::Bin;
  type Interfaces = (gst::URIHandler,);
}

impl GstObjectImpl for TauriAsset {}
impl ObjectImpl for TauriAsset {}
impl BinImpl for TauriAsset {}
impl ElementImpl for TauriAsset {}

impl URIHandlerImpl for TauriAsset {
  const URI_TYPE: gst::URIType = gst::URIType::Src;

  fn protocols() -> &'static [&'static str] {
    &[ASSET_URI_SCHEME]
  }

  fn uri(&self) -> Option<String> {
    None
  }

  fn set_uri(&self, uri: &str) -> Result<(), glib::Error> {
    // uri is like: asset://path/to/asset or asset://localhost/path/to/asset
    let sep = format!("{}://", ASSET_URI_SCHEME);
    let mut split = uri.split(sep.as_str());
    let location = split
      .nth(1)
      .ok_or_else(|| glib::Error::new(gst::URIError::BadUri, "Could not get location from URI"))?;

    // directly having full path after asset:// or having localhost
    let location = location.strip_prefix("localhost").unwrap_or(location);

    // Uri could be percent-encoded
    let location = percent_encoding::percent_decode_str(location)
      .decode_utf8()
      .map_err(|_| {
        glib::Error::new(
          gst::URIError::BadUri,
          "Could not decode percent-encoded URI",
        )
      })?
      .to_string();

    // now location is like: /path/to/asset
    let internal_src = gst::ElementFactory::make("filesrc")
      .name("filesrc")
      .property("location", location)
      .build()
      .ok();

    let element = self.obj();
    element
      .add(internal_src.as_ref().unwrap())
      .expect("Failed to add internal source");

    let srcpad = internal_src
      .as_ref()
      .and_then(|src| src.static_pad("src"))
      .ok_or_else(|| {
        glib::Error::new(
          gst::URIError::BadUri,
          "Could not get src pad from internal source",
        )
      })?;

    let ghostpad = GhostPad::new(PadDirection::Src);
    ghostpad
      .set_target(Some(&srcpad))
      .ok()
      .ok_or_else(|| glib::Error::new(gst::URIError::BadUri, "Could not create ghost pad"))?;

    ghostpad
      .set_active(true)
      .ok()
      .ok_or_else(|| glib::Error::new(gst::URIError::BadUri, "Could not activate ghost pad"))?;

    let element = self.obj();
    element.add_pad(&ghostpad).expect("Failed to add ghost pad");
    Ok(())
  }
}
