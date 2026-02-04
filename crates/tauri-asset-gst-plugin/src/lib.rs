use gst::glib;
use gstreamer as gst;

mod tauri_asset;

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
  tauri_asset::register(plugin)
}

gst::plugin_define!(
  tauriasset,
  env!("CARGO_PKG_DESCRIPTION"),
  plugin_init,
  concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
  "MIT/X11",
  env!("CARGO_PKG_NAME"),
  env!("CARGO_PKG_NAME"),
  env!("CARGO_PKG_REPOSITORY"),
  env!("BUILD_REL_DATE")
);
