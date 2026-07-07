// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  cell::{Cell, UnsafeCell},
  path::PathBuf,
  rc::Rc,
};

use gtk::{
  gdk::{DragAction, FileList},
  gio::Cancellable,
  prelude::*,
  DropTargetAsync,
};
use webkit::WebView;

use crate::DragDropEvent;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum DragControllerState {
  Entered,
  Leaving,
  Left,
}

struct DragDropController {
  paths: UnsafeCell<Option<Vec<PathBuf>>>,
  state: Cell<DragControllerState>,
  position: Cell<(i32, i32)>,
  handler: Box<dyn Fn(DragDropEvent) -> bool>,
}

impl DragDropController {
  fn new(handler: Box<dyn Fn(DragDropEvent) -> bool>) -> Self {
    Self {
      handler,
      paths: UnsafeCell::new(None),
      state: Cell::new(DragControllerState::Left),
      position: Cell::new((0, 0)),
    }
  }

  fn store_paths(&self, paths: Vec<PathBuf>) {
    unsafe { *self.paths.get() = Some(paths) };
  }

  fn take_paths(&self) -> Option<Vec<PathBuf>> {
    unsafe { &mut *self.paths.get() }.take()
  }

  fn store_position(&self, position: (i32, i32)) {
    self.position.replace(position);
  }

  fn enter(&self) {
    self.state.set(DragControllerState::Entered);
  }

  fn leaving(&self) {
    self.state.set(DragControllerState::Leaving);
  }

  fn leave(&self) {
    self.state.set(DragControllerState::Left);
  }

  fn state(&self) -> DragControllerState {
    self.state.get()
  }

  fn call(&self, event: DragDropEvent) -> bool {
    (self.handler)(event)
  }
}

pub(crate) fn connect_drag_event(webview: &WebView, handler: Box<dyn Fn(DragDropEvent) -> bool>) {
  let controller = Rc::new(DragDropController::new(handler));
  let drop_target = DropTargetAsync::new(None, DragAction::all());

  {
    let controller: Rc<DragDropController> = controller.clone();
    drop_target.connect_accept(move |_, drop| {
      let controller = controller.clone();
      drop.read_value_async(
        FileList::static_type(),
        gtk::glib::Priority::DEFAULT,
        Cancellable::NONE,
        move |result| {
          if let Ok(value) = result {
            if let Ok(files) = value.get::<FileList>() {
              let paths = files
                .files()
                .iter()
                .filter_map(|gfile| gfile.path())
                .collect::<Vec<_>>();

              controller.enter();
              controller.call(DragDropEvent::Enter {
                paths: paths.clone(),
                position: controller.position.get(),
              });
              controller.store_paths(paths);
            }
          }
        },
      );
      true
    });
  }

  {
    let controller = controller.clone();
    drop_target.connect_drag_motion(move |_, _, x, y| {
      if controller.state() == DragControllerState::Entered {
        controller.call(DragDropEvent::Over {
          position: (x.round() as _, y.round() as _),
        });
      } else {
        controller.store_position((x.round() as _, y.round() as _));
      }
      DragAction::COPY
    });
  }

  {
    let controller = controller.clone();
    drop_target.connect_drop(move |_, drop, x, y| {
      if controller.state() == DragControllerState::Entered {
        if let Some(paths) = controller.take_paths() {
          drop.finish(DragAction::COPY);
          controller.leave();
          return controller.call(DragDropEvent::Drop {
            paths,
            position: (x.round() as _, y.round() as _),
          });
        }
      }

      false
    });
  }

  drop_target.connect_drag_leave(move |_, _| {
    if controller.state() != DragControllerState::Left {
      controller.leaving();
      let controller = controller.clone();
      gtk::glib::idle_add_local_once(move || {
        if controller.state() == DragControllerState::Leaving {
          controller.leave();
          controller.call(DragDropEvent::Leave);
        }
      });
    }
  });

  webview.add_controller(drop_target);
}
