use std::fmt::Debug;

use tracing::{
  field::{Field, Visit},
  span::{Attributes, Record},
  Event, Id, Level, Metadata, Subscriber,
};

pub struct InvokeKeyErrorSubscriber;

impl Subscriber for InvokeKeyErrorSubscriber {
  fn enabled(&self, metadata: &Metadata<'_>) -> bool {
    metadata.is_event() && *metadata.level() == Level::ERROR
  }

  fn new_span(&self, _: &Attributes<'_>) -> Id {
    // shouldn't be called because we only enable events
    unimplemented!()
  }

  fn record(&self, _: &Id, _: &Record<'_>) {}

  fn record_follows_from(&self, _: &Id, _: &Id) {}

  fn event(&self, event: &Event<'_>) {
    event.record(&mut InvokeKeyExitVisit)
  }

  fn enter(&self, _: &Id) {}

  fn exit(&self, _: &Id) {}
}

struct InvokeKeyExitVisit;

impl Visit for InvokeKeyExitVisit {
  fn record_str(&mut self, field: &Field, value: &str) {
    if field.name() == "error" && value == "received ipc message without a __TAURI_INVOKE_KEY__" {
      std::process::exit(0)
    }
  }

  fn record_debug(&mut self, _: &Field, _: &dyn Debug) {}
}
