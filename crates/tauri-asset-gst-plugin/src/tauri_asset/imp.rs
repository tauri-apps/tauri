use std::sync::{LazyLock, OnceLock};

use gst::glib::subclass::types::ObjectSubclassExt;
use gst::prelude::{ElementExt, GstBinExt, PadExt};
use gst::subclass::prelude::{BinImpl, ElementImpl, ObjectImpl, URIHandlerImpl};
use gst::subclass::prelude::{GstObjectImpl, ObjectSubclass};
use gst::{glib, GhostPad};
use gstreamer::glib::object::ObjectExt;
use gstreamer::glib::subclass::object::ObjectImplExt;
use gstreamer_base::gst;

// GST_DEBUG=tauri_asset:5 ...
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
impl ObjectImpl for TauriAsset {
  fn constructed(&self) {
    self.parent_constructed();

    let element = self.obj();
    let filesrc = gst::ElementFactory::make("filesrc")
      .build()
      .unwrap_or_else(|err| {
        gst::error!(CAT, imp = self, "Failed to create filesrc element: {err}");
        panic!("Creating filesrc element failed");
      });

    element
      .add(&filesrc)
      .unwrap_or_else(|err| {
        gst::error!(CAT, imp = self, "Failed to add filesrc to bin: {err}");
        panic!("Adding filesrc to bin failed");
      });

    let srcpad = filesrc
      .static_pad("src")
      .unwrap_or_else(|| {
        gst::error!(CAT, imp = self, "Failed to get src pad from filesrc");
        panic!("Getting src pad failed");
      });

    let ghostpad = GhostPad::with_target(&srcpad).unwrap_or_else(|err| {
      gst::error!(CAT, imp = self, "Failed to create ghost pad: {err}");
      panic!("Creating ghost pad failed");
    });

    element
      .add_pad(&ghostpad)
      .unwrap_or_else(|err| {
        gst::error!(CAT, imp = self, "Failed to add ghost pad: {err}");
        panic!("Adding ghost pad failed");
      });

    ghostpad.set_active(true).unwrap_or_else(|err| {
        gst::error!(CAT, imp = self, "Failed to activate ghost pad: {err}");
        panic!("Ghost pad activation failed");
    });

    self.filesrc.set(filesrc).unwrap_or_else(|_| {
        gst::error!(CAT, imp = self, "Failed to set filesrc OnceLock");
        panic!("Setting filesrc OnceLock failed");
    });

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
    None
  }

  fn set_uri(&self, uri: &str) -> Result<(), glib::Error> {
    // uri is like: asset://path/to/asset or asset://localhost/path/to/asset
    let sep = format!("{}://", ASSET_URI_SCHEME);
    let mut split = uri.split(sep.as_str());
    let location = split.nth(1).ok_or_else(|| {
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
      .ok_or_else(|| {
        let msg = "filesrc element is not initialized";
        gst::error!(CAT, imp = self, "{msg}");
        glib::Error::new(gst::URIError::BadUri, &msg)
      })
      .map(|filesrc| {
        filesrc.set_property("location", &location);
      })
  }
}
