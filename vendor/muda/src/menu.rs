// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.inner
// SPDX-License-Identifier: MIT

use std::{cell::RefCell, rc::Rc};

use crate::{dpi::Position, util::AddOp, ContextMenu, IsMenuItem, MenuId, MenuItemKind};

/// A root menu that can be added to a Window on Windows and Linux
/// and used as the app global menu on macOS.
#[derive(Clone)]
pub struct Menu {
    id: Rc<MenuId>,
    inner: Rc<RefCell<crate::platform_impl::Menu>>,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    /// Creates a new menu.
    pub fn new() -> Self {
        let menu = crate::platform_impl::Menu::new(None);
        Self {
            id: Rc::new(menu.id().clone()),
            inner: Rc::new(RefCell::new(menu)),
        }
    }

    /// Creates a new menu with the specified id.
    pub fn with_id<I: Into<MenuId>>(id: I) -> Self {
        let id = id.into();
        Self {
            id: Rc::new(id.clone()),
            inner: Rc::new(RefCell::new(crate::platform_impl::Menu::new(Some(id)))),
        }
    }

    /// Creates a new menu with given `items`. It calls [`Menu::new`] and [`Menu::append_items`] internally.
    pub fn with_items(items: &[&dyn IsMenuItem]) -> crate::Result<Self> {
        let menu = Self::new();
        menu.append_items(items)?;
        Ok(menu)
    }

    /// Creates a new menu with the specified id and given `items`. It calls [`Menu::new`] and [`Menu::append_items`] internally.
    pub fn with_id_and_items<I: Into<MenuId>>(
        id: I,
        items: &[&dyn IsMenuItem],
    ) -> crate::Result<Self> {
        let menu = Self::with_id(id);
        menu.append_items(items)?;
        Ok(menu)
    }

    /// Returns a unique identifier associated with this menu.
    pub fn id(&self) -> &MenuId {
        &self.id
    }

    /// Add a menu item to the end of this menu.
    ///
    /// ## Platform-spcific:
    ///
    /// - **macOS:** Only [`Submenu`] can be added to the menu
    ///
    /// [`Submenu`]: crate::Submenu
    pub fn append(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner.borrow_mut().add_menu_item(item, AddOp::Append)
    }

    /// Add menu items to the end of this menu. It calls [`Menu::append`] in a loop internally.
    ///
    /// ## Platform-spcific:
    ///
    /// - **macOS:** Only [`Submenu`] can be added to the menu
    ///
    /// [`Submenu`]: crate::Submenu
    pub fn append_items(&self, items: &[&dyn IsMenuItem]) -> crate::Result<()> {
        for item in items {
            self.append(*item)?
        }

        Ok(())
    }

    /// Add a menu item to the beginning of this menu.
    ///
    /// ## Platform-spcific:
    ///
    /// - **macOS:** Only [`Submenu`] can be added to the menu
    ///
    /// [`Submenu`]: crate::Submenu
    pub fn prepend(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner
            .borrow_mut()
            .add_menu_item(item, AddOp::Insert(0))
    }

    /// Add menu items to the beginning of this menu. It calls [`Menu::insert_items`] with position of `0` internally.
    ///
    /// ## Platform-spcific:
    ///
    /// - **macOS:** Only [`Submenu`] can be added to the menu
    ///
    /// [`Submenu`]: crate::Submenu
    pub fn prepend_items(&self, items: &[&dyn IsMenuItem]) -> crate::Result<()> {
        self.insert_items(items, 0)
    }

    /// Insert a menu item at the specified `postion` in the menu.
    ///
    /// ## Platform-spcific:
    ///
    /// - **macOS:** Only [`Submenu`] can be added to the menu
    ///
    /// [`Submenu`]: crate::Submenu
    pub fn insert(&self, item: &dyn IsMenuItem, position: usize) -> crate::Result<()> {
        self.inner
            .borrow_mut()
            .add_menu_item(item, AddOp::Insert(position))
    }

    /// Insert menu items at the specified `postion` in the menu.
    ///
    /// ## Platform-spcific:
    ///
    /// - **macOS:** Only [`Submenu`] can be added to the menu
    ///
    /// [`Submenu`]: crate::Submenu
    pub fn insert_items(&self, items: &[&dyn IsMenuItem], position: usize) -> crate::Result<()> {
        for (i, item) in items.iter().enumerate() {
            self.insert(*item, position + i)?
        }

        Ok(())
    }

    /// Remove a menu item from this menu.
    pub fn remove(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner.borrow_mut().remove(item)
    }

    /// Remove the menu item at the specified position from this menu and returns it.
    pub fn remove_at(&self, position: usize) -> Option<MenuItemKind> {
        let mut items = self.items();
        if items.len() > position {
            let item = items.remove(position);
            let _ = self.remove(item.as_ref());
            Some(item)
        } else {
            None
        }
    }

    /// Returns a list of menu items that has been added to this menu.
    pub fn items(&self) -> Vec<MenuItemKind> {
        self.inner.borrow().items()
    }

    /// Adds this menu to a [`gtk::Window`]
    ///
    /// - `container`: this is an optional paramter to specify a container for the [`gtk::MenuBar`],
    ///   it is highly recommended to pass a container, otherwise the menubar will be added directly to the window,
    ///   which is usually not the desired behavior.
    ///   If using a [`gtk::Box`] as a container, it is added using [`Box::pack_start(menubar, false, false, 0)`](gtk::prelude::BoxExt::pack_start) then
    ///   reordered to be the first child of [`gtk::Box`] using [`Box::reorder_child(menubar, 0)`](gtk::prelude::BoxExt::reorder_child).
    ///
    /// ## Example:
    /// ```no_run
    /// let window = gtk::Window::builder().build();
    /// let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    /// let menu = muda::Menu::new();
    /// // -- snip, add your menu items --
    /// menu.init_for_gtk_window(&window, Some(&vbox));
    /// // then proceed to add your widgets to the `vbox`
    /// ```
    ///
    /// ## Panics:
    ///
    /// Panics if the gtk event loop hasn't been initialized on the thread.
    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        feature = "gtk"
    ))]
    pub fn init_for_gtk_window<W, C>(&self, window: &W, container: Option<&C>) -> crate::Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window>,
        W: gtk::prelude::IsA<gtk::Container>,
        C: gtk::prelude::IsA<gtk::Container>,
    {
        self.inner
            .borrow_mut()
            .init_for_gtk_window(window, container)
    }

    /// Adds this menu to a win32 window.
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    ///
    /// ##  Note about accelerators:
    ///
    /// For accelerators to work, the event loop needs to call
    /// [`TranslateAcceleratorW`](windows_sys::Win32::UI::WindowsAndMessaging::TranslateAcceleratorW)
    /// with the [`HACCEL`](windows_sys::Win32::UI::WindowsAndMessaging::HACCEL) returned from [`Menu::haccel`]
    ///
    /// #### Example:
    /// ```no_run
    /// # use muda::Menu;
    /// # use windows_sys::Win32::UI::WindowsAndMessaging::{MSG, GetMessageW, TranslateMessage, DispatchMessageW, TranslateAcceleratorW};
    /// let menu = Menu::new();
    /// unsafe {
    ///     let mut msg: MSG = std::mem::zeroed();
    ///     while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) == 1 {
    ///         let translated = TranslateAcceleratorW(msg.hwnd, menu.haccel() as _, &msg as *const _);
    ///         if translated != 1{
    ///             TranslateMessage(&msg);
    ///             DispatchMessageW(&msg);
    ///         }
    ///     }
    /// }
    /// ```
    #[cfg(target_os = "windows")]
    pub unsafe fn init_for_hwnd(&self, hwnd: isize) -> crate::Result<()> {
        self.inner.borrow_mut().init_for_hwnd(hwnd)
    }

    /// Adds this menu to a win32 window using the specified theme.
    ///
    /// See [Menu::init_for_hwnd] for more info.
    ///
    /// Note that the theme only affects the menu bar itself and not submenus or context menu.
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    #[cfg(target_os = "windows")]
    pub unsafe fn init_for_hwnd_with_theme(
        &self,
        hwnd: isize,
        theme: MenuTheme,
    ) -> crate::Result<()> {
        self.inner
            .borrow_mut()
            .init_for_hwnd_with_theme(hwnd, theme)
    }

    /// Set a theme for the menu bar on this window.
    ///
    /// Note that the theme only affects the menu bar itself and not submenus or context menu.
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    #[cfg(target_os = "windows")]
    pub unsafe fn set_theme_for_hwnd(&self, hwnd: isize, theme: MenuTheme) -> crate::Result<()> {
        self.inner.borrow().set_theme_for_hwnd(hwnd, theme)
    }

    /// Returns The [`HACCEL`](windows_sys::Win32::UI::WindowsAndMessaging::HACCEL) associated with this menu
    /// It can be used with [`TranslateAcceleratorW`](windows_sys::Win32::UI::WindowsAndMessaging::TranslateAcceleratorW)
    /// in the event loop to enable accelerators
    ///
    /// The returned [`HACCEL`](windows_sys::Win32::UI::WindowsAndMessaging::HACCEL) is valid as long as the [Menu] is.
    #[cfg(target_os = "windows")]
    pub fn haccel(&self) -> isize {
        self.inner.borrow_mut().haccel()
    }

    /// Removes this menu from a [`gtk::Window`]
    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        feature = "gtk"
    ))]
    pub fn remove_for_gtk_window<W>(&self, window: &W) -> crate::Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        self.inner.borrow_mut().remove_for_gtk_window(window)
    }

    /// Removes this menu from a win32 window
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    #[cfg(target_os = "windows")]
    pub unsafe fn remove_for_hwnd(&self, hwnd: isize) -> crate::Result<()> {
        self.inner.borrow_mut().remove_for_hwnd(hwnd)
    }

    /// Hides this menu from a [`gtk::Window`]
    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        feature = "gtk"
    ))]
    pub fn hide_for_gtk_window<W>(&self, window: &W) -> crate::Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        self.inner.borrow_mut().hide_for_gtk_window(window)
    }

    /// Hides this menu from a win32 window
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    #[cfg(target_os = "windows")]
    pub unsafe fn hide_for_hwnd(&self, hwnd: isize) -> crate::Result<()> {
        self.inner.borrow().hide_for_hwnd(hwnd)
    }

    /// Shows this menu on a [`gtk::Window`]
    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        feature = "gtk"
    ))]
    pub fn show_for_gtk_window<W>(&self, window: &W) -> crate::Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        self.inner.borrow_mut().show_for_gtk_window(window)
    }

    /// Shows this menu on a win32 window
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    #[cfg(target_os = "windows")]
    pub unsafe fn show_for_hwnd(&self, hwnd: isize) -> crate::Result<()> {
        self.inner.borrow().show_for_hwnd(hwnd)
    }

    /// Returns whether this menu visible on a [`gtk::Window`]
    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        feature = "gtk"
    ))]
    pub fn is_visible_on_gtk_window<W>(&self, window: &W) -> bool
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        self.inner.borrow().is_visible_on_gtk_window(window)
    }

    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        feature = "gtk"
    ))]
    /// Returns the [`gtk::MenuBar`] that is associated with this window if it exists.
    /// This is useful to get information about the menubar for example its height.
    pub fn gtk_menubar_for_gtk_window<W>(self, window: &W) -> Option<gtk::MenuBar>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        self.inner.borrow().gtk_menubar_for_gtk_window(window)
    }

    /// Returns whether this menu visible on a on a win32 window
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    #[cfg(target_os = "windows")]
    pub unsafe fn is_visible_on_hwnd(&self, hwnd: isize) -> bool {
        self.inner.borrow().is_visible_on_hwnd(hwnd)
    }

    /// Adds this menu to an NSApp.
    #[cfg(target_os = "macos")]
    pub fn init_for_nsapp(&self) {
        self.inner.borrow_mut().init_for_nsapp()
    }

    /// Removes this menu from an NSApp.
    #[cfg(target_os = "macos")]
    pub fn remove_for_nsapp(&self) {
        self.inner.borrow_mut().remove_for_nsapp()
    }
}

impl ContextMenu for Menu {
    #[cfg(target_os = "windows")]
    fn hpopupmenu(&self) -> isize {
        self.inner.borrow().hpopupmenu()
    }

    #[cfg(target_os = "windows")]
    unsafe fn show_context_menu_for_hwnd(&self, hwnd: isize, position: Option<Position>) -> bool {
        self.inner
            .borrow_mut()
            .show_context_menu_for_hwnd(hwnd, position)
    }

    #[cfg(target_os = "windows")]
    unsafe fn attach_menu_subclass_for_hwnd(&self, hwnd: isize) {
        self.inner.borrow().attach_menu_subclass_for_hwnd(hwnd)
    }

    #[cfg(target_os = "windows")]
    unsafe fn detach_menu_subclass_from_hwnd(&self, hwnd: isize) {
        self.inner.borrow().detach_menu_subclass_from_hwnd(hwnd)
    }

    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        feature = "gtk"
    ))]
    fn show_context_menu_for_gtk_window(
        &self,
        window: &gtk::Window,
        position: Option<Position>,
    ) -> bool {
        self.inner
            .borrow_mut()
            .show_context_menu_for_gtk_window(window, position)
    }

    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        feature = "gtk"
    ))]
    fn gtk_context_menu(&self) -> gtk::Menu {
        self.inner.borrow_mut().gtk_context_menu()
    }

    #[cfg(target_os = "macos")]
    unsafe fn show_context_menu_for_nsview(
        &self,
        view: *const std::ffi::c_void,
        position: Option<Position>,
    ) -> bool {
        self.inner
            .borrow_mut()
            .show_context_menu_for_nsview(view, position)
    }

    #[cfg(target_os = "macos")]
    fn ns_menu(&self) -> *mut std::ffi::c_void {
        self.inner.borrow().ns_menu()
    }

    fn as_menu(&self) -> Option<&Menu> {
        Some(self)
    }
}

/// The window menu bar theme
#[cfg(windows)]
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MenuTheme {
    Dark = 0,
    Light = 1,
    Auto = 2,
}
