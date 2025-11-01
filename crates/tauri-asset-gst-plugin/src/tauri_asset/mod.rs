use gst::glib;
use gst::prelude::*;
use gstreamer_base::gst;

mod imp;

glib::wrapper! {
    pub struct TauriAsset(ObjectSubclass<imp::TauriAsset>)
    @extends gst::Bin, gst::Element, gst::Object,
    @implements gst::URIHandler;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
  gst::Element::register(
    Some(plugin),
    "tauriasset",
    gst::Rank::PRIMARY,
    TauriAsset::static_type(),
  )
}
