use std::{
  rc::Rc,
  sync::{
    atomic::{AtomicI32, Ordering},
    OnceLock,
  },
};

use gtk4::{
  glib::{self, subclass::Signal},
  prelude::*,
  subclass::prelude::*,
};

// Libadwaita support - conditional imports
#[cfg(feature = "libadwaita")]
use libadwaita::subclass::application_window::AdwApplicationWindowImpl;

#[derive(Debug, Default)]
// By implementing Default we don't have to provide a `new` fn in our
// ObjectSubclass impl.
pub struct ApplicationWindow {
  pub outer_size: Rc<(AtomicI32, AtomicI32)>,
  pub inner_size: Rc<(AtomicI32, AtomicI32)>,
}

#[glib::object_subclass]
impl ObjectSubclass for ApplicationWindow {
  const NAME: &'static str = "ExTaoWindow";
  type Type = super::ApplicationWindow;

  #[cfg(feature = "libadwaita")]
  type ParentType = libadwaita::ApplicationWindow;
  #[cfg(not(feature = "libadwaita"))]
  type ParentType = gtk4::ApplicationWindow;
}

impl ObjectImpl for ApplicationWindow {
  fn signals() -> &'static [Signal] {
    static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
    SIGNALS.get_or_init(|| {
      vec![Signal::builder("resized")
        .param_types([i32::static_type(), i32::static_type()])
        .build()]
    })
  }
}

impl WidgetImpl for ApplicationWindow {
  fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
    self.parent_size_allocate(width, height, baseline);
    let obj = self.obj();

    self.inner_size.0.store(width, Ordering::Release);
    self.inner_size.1.store(height, Ordering::Release);

    obj.emit_by_name::<()>("resized", &[&width, &height]);

    if obj.is_realized() {
      let surface = obj.surface().unwrap();
      self.outer_size.0.store(surface.width(), Ordering::Release);
      self.outer_size.1.store(surface.height(), Ordering::Release);
    }
  }
}
impl WindowImpl for ApplicationWindow {}
impl ApplicationWindowImpl for ApplicationWindow {}

#[cfg(feature = "libadwaita")]
impl AdwApplicationWindowImpl for ApplicationWindow {}
