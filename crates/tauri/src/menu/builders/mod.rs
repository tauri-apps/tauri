// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(desktop)]

//! A module containing menu builder types

type ItemEventCallback<T> = Box<dyn Fn(&T, crate::menu::MenuEvent) + Send + Sync + 'static>;

/// The default handler state of a menu item builder before a handler is set.
#[doc(hidden)]
pub struct NoHandler;

/// Converts a menu item builder's inferred handler into a callback for its item type.
#[doc(hidden)]
pub trait ItemEventHandler<T>: sealed::Sealed<T> + Send + Sync + 'static {
  fn into_handler(self) -> Option<ItemEventCallback<T>>;
}

impl<T> ItemEventHandler<T> for NoHandler {
  fn into_handler(self) -> Option<ItemEventCallback<T>> {
    None
  }
}

impl<T, F> ItemEventHandler<T> for F
where
  F: Fn(&T, crate::menu::MenuEvent) + Send + Sync + 'static,
{
  fn into_handler(self) -> Option<ItemEventCallback<T>> {
    Some(Box::new(self))
  }
}

mod sealed {
  pub trait Sealed<T> {}
  impl<T> Sealed<T> for super::NoHandler {}
  impl<T, F> Sealed<T> for F where F: Fn(&T, crate::menu::MenuEvent) + Send + Sync + 'static {}
}

mod menu;
pub use menu::MenuBuilder;
pub use menu::SubmenuBuilder;
mod normal;
pub use normal::MenuItemBuilder;
mod check;
pub use check::CheckMenuItemBuilder;
mod icon;
pub use icon::IconMenuItemBuilder;

// GTK menu items require initialization on GTK's event-loop thread.
#[cfg(all(test, not(gtk)))]
mod tests {
  use std::sync::Arc;

  use super::{CheckMenuItemBuilder, IconMenuItemBuilder, MenuItemBuilder};
  use crate::{menu::MenuEvent, sealed::ManagerBase, test::mock_app};

  #[test]
  fn handlers_do_not_keep_menu_items_alive() {
    let app = mock_app();
    let menu_manager = &app.manager().menu;

    macro_rules! assert_no_leak {
      ($builder:ident) => {{
        let item = $builder::new("item")
          .handler(|item, event| assert_eq!(item.id(), event.id()))
          .build(&app)
          .unwrap();
        let weak_item = Arc::downgrade(&item.0);
        let id = item.id().clone();

        assert!(
          menu_manager
            .item_event_listeners
            .lock()
            .unwrap()
            .contains_key(&id)
        );
        menu_manager.dispatch_menu_item_event(MenuEvent { id: id.clone() });

        drop(item);

        assert!(weak_item.upgrade().is_none(), "handler retained the item");
        assert!(
          !menu_manager
            .item_event_listeners
            .lock()
            .unwrap()
            .contains_key(&id),
          "handler registration was not removed"
        );
      }};
    }

    assert_no_leak!(MenuItemBuilder);
    assert_no_leak!(CheckMenuItemBuilder);
    assert_no_leak!(IconMenuItemBuilder);
  }
}
