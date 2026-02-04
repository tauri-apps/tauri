use std::sync::{LazyLock, OnceLock};

use gst::glib::subclass::types::ObjectSubclassExt;
use gst::prelude::{ElementExt, GstBinExt, PadExt};
use gst::subclass::prelude::{BinImpl, ElementImpl, ObjectImpl, URIHandlerImpl};
use gst::subclass::prelude::{GstObjectImpl, ObjectSubclass};
use gst::{glib, GhostPad};
use gstreamer as gst;
use gstreamer::glib::object::ObjectExt;
use gstreamer::glib::subclass::object::ObjectImplExt;

// for debugging
// GST_DEBUG=tauri_asset:5 <your-binary-using-the-plugin>
static CAT: LazyLock<gst::DebugCategory> = LazyLock::new(|| {
  gst::DebugCategory::new(
    "tauri_asset",
    gst::DebugColorFlags::empty(),
    Some("Tauri Asset Element"),
  )
});

const ASSET_URI_SCHEME: &str = "asset";

#[derive(Default)]
pub struct TauriAsset {
  pub filesrc: OnceLock<gst::Element>,
}

#[glib::object_subclass]
impl ObjectSubclass for TauriAsset {
  const NAME: &'static str = "GstTauriAsset";
  type Type = super::TauriAsset;
  type ParentType = gst::Bin;
  type Interfaces = (gst::URIHandler,);
}

impl GstObjectImpl for TauriAsset {}
impl ObjectImpl for TauriAsset {
  fn constructed(&self) {
    self.parent_constructed();

    let element = self.obj();
    let filesrc = match gst::ElementFactory::make("filesrc").build() {
      Ok(src) => src,
      Err(err) => {
        gst::element_error!(
          element,
          gst::LibraryError::Init,
          ("Failed to create filesrc: {}", err)
        );
        return;
      }
    };

    match element.add(&filesrc) {
      Ok(_) => (),
      Err(err) => {
        gst::element_error!(
          element,
          gst::LibraryError::Init,
          ("Failed to add filesrc to bin: {}", err)
        );
        return;
      }
    };

    let srcpad = match filesrc.static_pad("src") {
      Some(pad) => pad,
      None => {
        gst::element_error!(
          element,
          gst::LibraryError::Init,
          ("Failed to get src pad from filesrc")
        );
        return;
      }
    };

    let ghostpad = match GhostPad::with_target(&srcpad) {
      Ok(pad) => pad,
      Err(err) => {
        gst::element_error!(
          element,
          gst::LibraryError::Init,
          ("Failed to create ghost pad from filesrc src pad: {}", err)
        );
        return;
      }
    };

    match element.add_pad(&ghostpad) {
      Ok(_) => (),
      Err(err) => {
        gst::element_error!(
          element,
          gst::LibraryError::Init,
          ("Failed to add ghost pad to bin: {}", err)
        );
        return;
      }
    };

    match ghostpad.set_active(true) {
      Ok(_) => (),
      Err(err) => {
        gst::element_error!(
          element,
          gst::LibraryError::Init,
          ("Failed to activate ghost pad: {}", err)
        );
        return;
      }
    };

    match self.filesrc.set(filesrc) {
      Ok(_) => (),
      Err(_) => {
        gst::element_error!(
          element,
          gst::LibraryError::Init,
          ("Failed to store filesrc element")
        );
        return;
      }
    }

    gst::debug!(CAT, imp = self, "TauriAsset constructed");
  }
}
impl BinImpl for TauriAsset {}
impl ElementImpl for TauriAsset {}

impl URIHandlerImpl for TauriAsset {
  const URI_TYPE: gst::URIType = gst::URIType::Src;

  fn protocols() -> &'static [&'static str] {
    &[ASSET_URI_SCHEME]
  }

  fn uri(&self) -> Option<String> {
    self.filesrc.get().and_then(|src| {
      src
        .property::<Option<String>>("location")
        .filter(|p| !p.is_empty())
        .map(|p| format!("{}://{}", ASSET_URI_SCHEME, p))
    })
  }

  fn set_uri(&self, uri: &str) -> Result<(), glib::Error> {
    // uri is like: asset://path/to/asset or asset://localhost/path/to/asset
    let sep = format!("{}://", ASSET_URI_SCHEME);
    let mut split = uri.split(sep.as_str());
    let location = split.nth(1).filter(|l| !l.is_empty()).ok_or_else(|| {
      let msg = format!("URI does not contain location: {}", uri);
      gst::error!(CAT, imp = self, "{msg}");
      glib::Error::new(gst::URIError::BadUri, &msg)
    })?;

    // directly having full path after asset:// or having localhost
    let location = location.strip_prefix("localhost").unwrap_or(location);

    // Uri could be percent-encoded
    let location = percent_encoding::percent_decode_str(location)
      .decode_utf8()
      .map_err(|_| {
        let msg = format!("Failed to decode percent-encoded URI: {}", uri);
        gst::error!(CAT, imp = self, "{msg}");
        glib::Error::new(gst::URIError::BadUri, &msg)
      })?
      .to_string();

    gst::debug!(CAT, imp = self, "URI from \"{}\" to \"{}\"", uri, &location);

    self
      .filesrc
      .get()
      .map(|src| src.set_property("location", &location))
      .ok_or_else(|| {
        let msg = "filesrc element is not initialized";
        gst::error!(CAT, imp = self, "{msg}");
        glib::Error::new(gst::URIError::BadUri, msg)
      })
  }
}
