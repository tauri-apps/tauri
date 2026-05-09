// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.inner
// SPDX-License-Identifier: MIT

use std::{cell::RefCell, mem, rc::Rc};

use crate::{
    dpi::Position, sealed::IsMenuItemBase, util::AddOp, ContextMenu, Icon, IsMenuItem, MenuId,
    MenuItemKind, NativeIcon,
};

/// A menu that can be added to a [`Menu`] or another [`Submenu`].
///
/// [`Menu`]: crate::Menu
#[derive(Clone)]
pub struct Submenu {
    pub(crate) id: Rc<MenuId>,
    pub(crate) inner: Rc<RefCell<crate::platform_impl::MenuChild>>,
}

impl IsMenuItemBase for Submenu {}
impl IsMenuItem for Submenu {
    fn kind(&self) -> MenuItemKind {
        MenuItemKind::Submenu(self.clone())
    }

    fn id(&self) -> &MenuId {
        self.id()
    }

    fn into_id(self) -> MenuId {
        self.into_id()
    }
}

impl Submenu {
    /// Create a new submenu.
    ///
    /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
    ///   for this submenu. To display a `&` without assigning a mnemenonic, use `&&`.
    pub fn new<S: AsRef<str>>(text: S, enabled: bool) -> Self {
        let submenu = crate::platform_impl::MenuChild::new_submenu(text.as_ref(), enabled, None);
        Self {
            id: Rc::new(submenu.id().clone()),
            inner: Rc::new(RefCell::new(submenu)),
        }
    }

    /// Create a new submenu with the specified id.
    ///
    /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
    ///   for this submenu. To display a `&` without assigning a mnemenonic, use `&&`.
    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(id: I, text: S, enabled: bool) -> Self {
        let id = id.into();

        Self {
            id: Rc::new(id.clone()),
            inner: Rc::new(RefCell::new(crate::platform_impl::MenuChild::new_submenu(
                text.as_ref(),
                enabled,
                Some(id),
            ))),
        }
    }

    /// Creates a new submenu with given `items`. It calls [`Submenu::new`] and [`Submenu::append_items`] internally.
    pub fn with_items<S: AsRef<str>>(
        text: S,
        enabled: bool,
        items: &[&dyn IsMenuItem],
    ) -> crate::Result<Self> {
        let menu = Self::new(text, enabled);
        menu.append_items(items)?;
        Ok(menu)
    }

    /// Creates a new submenu with the specified id and given `items`. It calls [`Submenu::new`] and [`Submenu::append_items`] internally.
    pub fn with_id_and_items<I: Into<MenuId>, S: AsRef<str>>(
        id: I,
        text: S,
        enabled: bool,
        items: &[&dyn IsMenuItem],
    ) -> crate::Result<Self> {
        let menu = Self::with_id(id, text, enabled);
        menu.append_items(items)?;
        Ok(menu)
    }

    /// Returns a unique identifier associated with this submenu.
    pub fn id(&self) -> &MenuId {
        &self.id
    }

    /// Add a menu item to the end of this menu.
    pub fn append(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner.borrow_mut().add_menu_item(item, AddOp::Append)
    }

    /// Add menu items to the end of this submenu. It calls [`Submenu::append`] in a loop.
    pub fn append_items(&self, items: &[&dyn IsMenuItem]) -> crate::Result<()> {
        for item in items {
            self.append(*item)?
        }

        Ok(())
    }

    /// Add a menu item to the beginning of this submenu.
    pub fn prepend(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner
            .borrow_mut()
            .add_menu_item(item, AddOp::Insert(0))
    }

    /// Add menu items to the beginning of this submenu.
    /// It calls [`Menu::prepend`](crate::Menu::prepend) on the first element and
    /// passes the rest to [`Menu::insert_items`](crate::Menu::insert_items) with position of `1`.
    pub fn prepend_items(&self, items: &[&dyn IsMenuItem]) -> crate::Result<()> {
        self.insert_items(items, 0)
    }

    /// Insert a menu item at the specified `postion` in the submenu.
    pub fn insert(&self, item: &dyn IsMenuItem, position: usize) -> crate::Result<()> {
        self.inner
            .borrow_mut()
            .add_menu_item(item, AddOp::Insert(position))
    }

    /// Insert menu items at the specified `postion` in the submenu.
    pub fn insert_items(&self, items: &[&dyn IsMenuItem], position: usize) -> crate::Result<()> {
        for (i, item) in items.iter().enumerate() {
            self.insert(*item, position + i)?
        }

        Ok(())
    }

    /// Remove a menu item from this submenu.
    pub fn remove(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner.borrow_mut().remove(item)
    }

    /// Remove the menu item at the specified position from this submenu and returns it.
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

    /// Returns a list of menu items that has been added to this submenu.
    pub fn items(&self) -> Vec<MenuItemKind> {
        self.inner.borrow().items()
    }

    /// Get the text for this submenu.
    pub fn text(&self) -> String {
        self.inner.borrow().text()
    }

    /// Set the text for this submenu. `text` could optionally contain
    /// an `&` before a character to assign this character as the mnemonic
    /// for this submenu. To display a `&` without assigning a mnemenonic, use `&&`.
    pub fn set_text<S: AsRef<str>>(&self, text: S) {
        self.inner.borrow_mut().set_text(text.as_ref())
    }

    /// Get whether this submenu is enabled or not.
    pub fn is_enabled(&self) -> bool {
        self.inner.borrow().is_enabled()
    }

    /// Enable or disable this submenu.
    pub fn set_enabled(&self, enabled: bool) {
        self.inner.borrow_mut().set_enabled(enabled)
    }

    /// Set this submenu as the Window menu for the application on macOS.
    /// This will cause macOS to automatically add window-switching items and
    /// certain other items to the menu.
    ///
    /// Must be called after adding this submenu to [`Menu`](crate::Menu)
    /// and after calling [`Menu::init_for_nsapp`](crate::Menu::init_for_nsapp) on that menu.
    ///
    ///
    /// # Note
    ///
    /// Because a [`Submenu`] can be added multiple times to the same [`Menu`](crate::Menu)
    /// this method will set the first instance of this submenu as the Window menu for the application.
    ///
    /// It is not recommended to add the same submenu multiple times to the same menu, but if you do, be aware of this behavior.
    #[cfg(target_os = "macos")]
    pub fn set_as_windows_menu_for_nsapp(&self) {
        self.inner.borrow_mut().set_as_windows_menu_for_nsapp()
    }

    /// Set this submenu as the Help menu for the application on macOS.
    /// This will cause macOS to automatically add a search box to the menu.
    ///
    /// Must be called after adding this submenu to [`Menu`](crate::Menu)
    /// and after calling [`Menu::init_for_nsapp`](crate::Menu::init_for_nsapp) on that menu.
    ///
    /// If no menu is set as the Help menu, macOS will automatically use any menu
    /// which has a title matching the localized word "Help".
    ///
    /// # Note
    ///
    /// Because a [`Submenu`] can be added multiple times to the same [`Menu`](crate::Menu)
    /// this method will set the first instance of this submenu as the Help menu for the application.
    ///
    /// It is not recommended to add the same submenu multiple times to the same menu, but if you do, be aware of this behavior.
    #[cfg(target_os = "macos")]
    pub fn set_as_help_menu_for_nsapp(&self) {
        self.inner.borrow_mut().set_as_help_menu_for_nsapp()
    }

    /// Convert this submenu into its menu ID.
    pub fn into_id(mut self) -> MenuId {
        // Note: `Rc::into_inner` is available from Rust 1.70
        if let Some(id) = Rc::get_mut(&mut self.id) {
            mem::take(id)
        } else {
            self.id().clone()
        }
    }

    /// Change this menu item icon or remove it.
    pub fn set_icon(&self, icon: Option<Icon>) {
        self.inner.borrow_mut().set_icon(icon)
    }

    /// Change this menu item icon to a native image or remove it.
    ///
    /// ## Platform-specific:
    ///
    /// - **Windows / Linux**: Unsupported.
    pub fn set_native_icon(&self, _icon: Option<NativeIcon>) {
        #[cfg(target_os = "macos")]
        self.inner.borrow_mut().set_native_icon(_icon)
    }
}

impl ContextMenu for Submenu {
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
        w: &gtk::Window,
        position: Option<Position>,
    ) -> bool {
        self.inner
            .borrow_mut()
            .show_context_menu_for_gtk_window(w, position)
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

    fn as_submenu(&self) -> Option<&Submenu> {
        Some(self)
    }
}
