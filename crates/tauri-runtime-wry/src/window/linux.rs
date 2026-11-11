// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use gtk::{prelude::*, subclass::prelude::ObjectSubclassIsExt};
use tao::dpi::LogicalSize;
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
use tao::platform::unix::WindowExtUnix;

impl super::WindowExt for tao::window::Window {
  fn set_enabled(&self, enabled: bool) {
    self.gtk_window().set_sensitive(enabled);
  }

  fn is_enabled(&self) -> bool {
    self.gtk_window().is_sensitive()
  }

  fn center(&self) {
    if let Some(monitor) = self.current_monitor() {
      let window_size = self.outer_size();
      let new_pos = super::calculate_window_center_position(window_size, monitor);
      self.set_outer_position(new_pos);
    }
  }
}

mod imp {
  use gtk::{glib, prelude::*, subclass::prelude::*};
  use std::cell::RefCell;
  use tao::dpi::LogicalSize;

  type OnRoom = Box<dyn Fn(LogicalSize<f32>)>;

  #[derive(Default)]
  pub struct WebviewContainer {
    pub(super) on_room: RefCell<Option<OnRoom>>,
  }

  #[glib::object_subclass]
  impl ObjectSubclass for WebviewContainer {
    const NAME: &'static str = "TauriWebviewContainer";
    type Type = super::WebviewContainer;
    type ParentType = gtk::Fixed;
  }

  impl ObjectImpl for WebviewContainer {
    fn constructed(&self) {
      self.parent_constructed();
      self.obj().set_has_window(true);
    }
  }

  impl WidgetImpl for WebviewContainer {
    fn preferred_width(&self) -> (i32, i32) {
      (0, 0)
    }

    fn preferred_height(&self) -> (i32, i32) {
      (0, 0)
    }

    // The parent allocates the webviews from their size request, so those that follow the room
    // get theirs first. They have their size in the same layout pass, as a packed webview has.
    // `on_room` ends in `gtk::Fixed::move_`, which queues a resize. GTK drops a resize that a widget
    // queues during its own allocation, so this does not start another layout pass.
    fn size_allocate(&self, allocation: &gtk::Allocation) {
      if let (Some(on_room), Some(room)) = (&*self.on_room.borrow(), super::room_of(allocation)) {
        on_room(room);
      }
      self.parent_size_allocate(allocation);
    }
  }

  impl ContainerImpl for WebviewContainer {}
  impl FixedImpl for WebviewContainer {}
}

gtk::glib::wrapper! {
  /// The container of the webviews of a window: a `GtkFixed`, where wry puts each webview at its
  /// bounds.
  ///
  /// A `GtkFixed` asks for the room of its children. That would grow the window and stop it from
  /// shrinking, so this one asks for none. It has its own window, which clips the webviews and the
  /// input that goes to them at its edges.
  pub struct WebviewContainer(ObjectSubclass<imp::WebviewContainer>)
    @extends gtk::Fixed, gtk::Container, gtk::Widget;
}

impl WebviewContainer {
  /// `on_room` runs with the room that GTK gives the container, each time it allocates it.
  pub fn new(on_room: impl Fn(LogicalSize<f32>) + 'static) -> Self {
    let container: Self = gtk::glib::Object::new();
    *container.imp().on_room.borrow_mut() = Some(Box::new(on_room));
    container
  }
}

/// Adds the container of the webviews to `window`, if every webview of it is a child: with the
/// `unstable` feature. `on_room` runs when GTK gives the container a new room.
pub fn add_webview_container(
  window: &tao::window::Window,
  on_room: impl Fn(LogicalSize<f32>) + 'static,
) -> Option<gtk::Fixed> {
  if !cfg!(feature = "unstable") {
    return None;
  }
  let container = WebviewContainer::new(on_room);
  window.default_vbox()?.pack_start(&container, true, true, 0);
  container.show();
  Some(container.upcast())
}

/// The room that an allocation of the container gives, in logical pixels. GTK allocates a widget
/// that it has not laid out yet with a size of 1x1.
fn room_of(allocation: &gtk::Allocation) -> Option<LogicalSize<f32>> {
  (allocation.width() > 1 && allocation.height() > 1)
    .then(|| LogicalSize::new(allocation.width() as f32, allocation.height() as f32))
}

/// The room that GTK gives the webviews of the container that holds `widget`, in logical pixels,
/// once it has allocated them. The inner size of the window is bigger: it counts the menu bar and,
/// on Wayland, the client-side decorations.
pub fn allocated_room(widget: &impl IsA<gtk::Widget>) -> Option<LogicalSize<f32>> {
  room_of(
    &widget
      .ancestor(WebviewContainer::static_type())?
      .allocation(),
  )
}

/// The room of the webviews of `window`, if `widget` is in their container: what GTK gave them.
/// Before GTK allocates the window, the inner size of the window less what the other children of
/// its vbox ask for, the menu bar.
pub fn room(
  window: &tao::window::Window,
  widget: &impl IsA<gtk::Widget>,
) -> Option<LogicalSize<f32>> {
  widget.ancestor(WebviewContainer::static_type())?;
  Some(allocated_room(widget).unwrap_or_else(|| {
    let size = window.inner_size().to_logical::<f32>(window.scale_factor());
    let others: i32 = window.default_vbox().map_or(0, |vbox| {
      vbox
        .children()
        .iter()
        .filter(|child| !child.is::<WebviewContainer>())
        .map(|child| child.preferred_height().0)
        .sum()
    });
    LogicalSize::new(size.width, (size.height - others as f32).max(1.))
  }))
}
