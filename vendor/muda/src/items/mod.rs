// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod check;
mod icon;
mod normal;
mod predefined;
mod submenu;

pub use check::*;
pub use icon::*;
pub use normal::*;
pub use predefined::*;
pub use submenu::*;

#[cfg(test)]
mod test {
    use crate::{CheckMenuItem, IconMenuItem, MenuId, MenuItem, PredefinedMenuItem, Submenu};

    #[test]
    #[cfg_attr(
        all(
            miri,
            not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))
        ),
        ignore
    )]
    fn it_returns_same_id() {
        let id = MenuId::new("1");
        assert_eq!(id, MenuItem::with_id(id.clone(), "", true, None).id());
        assert_eq!(id, Submenu::with_id(id.clone(), "", true).id());
        assert_eq!(
            id,
            CheckMenuItem::with_id(id.clone(), "", true, true, None).id()
        );
        assert_eq!(
            id,
            IconMenuItem::with_id(id.clone(), "", true, None, None).id()
        );
    }

    #[test]
    #[cfg_attr(
        all(
            miri,
            not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))
        ),
        ignore
    )]
    fn test_convert_from_id_and_into_id() {
        let id = "TEST ID";
        let expected = MenuId(id.to_string());

        let item = CheckMenuItem::with_id(id, "test", true, true, None);
        assert_eq!(item.id(), &expected);
        assert_eq!(item.into_id(), expected);

        let item = IconMenuItem::with_id(id, "test", true, None, None);
        assert_eq!(item.id(), &expected);
        assert_eq!(item.into_id(), expected);

        let item = MenuItem::with_id(id, "test", true, None);
        assert_eq!(item.id(), &expected);
        assert_eq!(item.into_id(), expected);

        let item = Submenu::with_id(id, "test", true);
        assert_eq!(item.id(), &expected);
        assert_eq!(item.into_id(), expected);

        let item = PredefinedMenuItem::separator();
        assert_eq!(item.id().clone(), item.into_id());
    }
}
