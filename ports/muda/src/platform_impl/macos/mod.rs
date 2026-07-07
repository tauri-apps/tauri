// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod accelerator;
mod icon;
mod util;

pub(crate) use icon::PlatformIcon;

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ffi::c_void,
    ptr,
    rc::Rc,
};

use objc2::{
    define_class, msg_send,
    rc::Retained,
    runtime::{AnyObject, NSObjectProtocol, ProtocolObject, Sel},
    sel, DeclaredClass, MainThreadOnly, Message,
};
use objc2_app_kit::{
    NSAboutPanelOptionApplicationIcon, NSAboutPanelOptionApplicationName,
    NSAboutPanelOptionApplicationVersion, NSAboutPanelOptionCredits, NSAboutPanelOptionVersion,
    NSApplication, NSControlStateValueOff, NSControlStateValueOn, NSEvent, NSEventModifierFlags,
    NSImage, NSImageName, NSMenu, NSMenuDelegate, NSMenuItem, NSRunningApplication, NSView,
};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSAttributedString, NSDictionary, NSInteger, NSObject, NSPoint,
    NSSize, NSString,
};

use self::util::strip_mnemonic;
use crate::{
    accelerator::KeyAccelerator,
    dpi::{LogicalPosition, Position},
    icon::{Icon, NativeIcon},
    items::*,
    util::{AddOp, Counter},
    IsMenuItem, MenuEvent, MenuId, MenuItemKind, MenuItemType,
};

static COUNTER: Counter = Counter::new();

/// https://developer.apple.com/documentation/appkit/nsapplication/1428479-orderfrontstandardaboutpanelwith#discussion
#[allow(non_upper_case_globals)]
const NSAboutPanelOptionCopyright: &str = "Copyright";

define_class!(
    /// A delegate for NSMenu that stores the menu id as an instance variable,
    /// so that we can identify it later. Like when calling `set_as_windows_menu_for_nsapp`.
    #[unsafe(super(NSObject))]
    #[name = "MudaMenuDelegate"]
    #[thread_kind = MainThreadOnly]
    #[ivars = u32]
    struct MudaMenuDelegate;

    unsafe impl NSObjectProtocol for MudaMenuDelegate {}
    unsafe impl NSMenuDelegate for MudaMenuDelegate {}
);

impl MudaMenuDelegate {
    fn new(mtm: MainThreadMarker, menu_id: u32) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(menu_id);
        unsafe { msg_send![super(this), init] }
    }

    fn menu_id(&self) -> u32 {
        *self.ivars()
    }
}

#[derive(Clone)]
struct NsMenuRef(
    u32,
    Retained<NSMenu>,
    /// Prevent deallocation — NSMenu's delegate is a weak reference.
    #[allow(dead_code)]
    Retained<MudaMenuDelegate>,
);

impl NsMenuRef {
    fn new(mtm: MainThreadMarker, id: u32, ns_menu: Retained<NSMenu>) -> Self {
        let delegate = MudaMenuDelegate::new(mtm, id);
        unsafe {
            ns_menu.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        }
        Self(id, ns_menu, delegate)
    }
}

impl std::fmt::Debug for NsMenuRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NsMenuRef")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

impl Drop for NsMenuRef {
    fn drop(&mut self) {
        unsafe { self.1.cancelTrackingWithoutAnimation() };
    }
}

#[derive(Debug)]
pub struct Menu {
    id: MenuId,
    ns_menu: NsMenuRef,
    children: Vec<Rc<RefCell<MenuChild>>>,
}

impl Drop for Menu {
    fn drop(&mut self) {
        for child in &self.children {
            let mut child_ = child.borrow_mut();
            child_.ns_menu_items.remove(&self.ns_menu.0);
            if child_.item_type == MenuItemType::Submenu {
                child_.ns_menus.as_mut().unwrap().remove(&self.ns_menu.0);
            }
        }
    }
}

impl Menu {
    pub fn new(id: Option<MenuId>) -> Self {
        let mtm =
            MainThreadMarker::new().expect("`muda::Menu` can only be created on the main thread");
        let ns_menu = NSMenu::new(mtm);
        unsafe { ns_menu.setAutoenablesItems(false) };
        Self {
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            ns_menu: NsMenuRef::new(mtm, COUNTER.next(), ns_menu),
            children: Vec::new(),
        }
    }

    pub fn id(&self) -> &MenuId {
        &self.id
    }

    pub fn add_menu_item(&mut self, item: &dyn crate::IsMenuItem, op: AddOp) -> crate::Result<()> {
        let ns_menu_item = item.make_ns_item_for_menu(self.ns_menu.0)?;
        let child = item.child();

        unsafe {
            match op {
                AddOp::Append => {
                    self.ns_menu.1.addItem(&ns_menu_item);
                    self.children.push(child);
                }
                AddOp::Insert(position) => {
                    self.ns_menu
                        .1
                        .insertItem_atIndex(&ns_menu_item, position as NSInteger);
                    self.children.insert(position, child);
                }
            }
        }

        Ok(())
    }

    pub fn remove(&mut self, item: &dyn crate::IsMenuItem) -> crate::Result<()> {
        // get child
        let child = {
            let index = self
                .children
                .iter()
                .position(|e| e.borrow().id == item.id())
                .ok_or(crate::Error::NotAChildOfThisMenu)?;
            self.children.remove(index)
        };

        let mut child_ = child.borrow_mut();

        if child_.item_type == MenuItemType::Submenu {
            let menu_id = &self.ns_menu.0;
            let menus = child_.ns_menus.as_ref().unwrap().get(menu_id).cloned();
            if let Some(menus) = menus {
                for menu in menus {
                    for item in child_.items() {
                        child_.remove_inner(item.as_ref(), false, Some(menu.0))?;
                    }
                }
            }
            child_.ns_menus.as_mut().unwrap().remove(menu_id);
        }

        // remove each NSMenuItem from the NSMenu
        if let Some(ns_menu_items) = child_.ns_menu_items.remove(&self.ns_menu.0) {
            for item in ns_menu_items {
                unsafe { self.ns_menu.1.removeItem(&item) };
            }
        }

        Ok(())
    }

    pub fn items(&self) -> Vec<MenuItemKind> {
        self.children
            .iter()
            .map(|c| c.borrow().kind(c.clone()))
            .collect()
    }

    pub fn init_for_nsapp(&self) {
        let mtm = MainThreadMarker::from(&*self.ns_menu.1);
        let app = NSApplication::sharedApplication(mtm);
        app.setMainMenu(Some(&self.ns_menu.1));
    }

    pub fn remove_for_nsapp(&self) {
        let mtm = MainThreadMarker::from(&*self.ns_menu.1);
        let app = NSApplication::sharedApplication(mtm);
        app.setMainMenu(None);
    }

    pub unsafe fn show_context_menu_for_nsview(
        &self,
        view: *const c_void,
        position: Option<Position>,
    ) -> bool {
        // SAFETY: Upheld by caller
        show_context_menu(&self.ns_menu.1, view, position)
    }

    pub fn ns_menu(&self) -> *mut std::ffi::c_void {
        Retained::as_ptr(&self.ns_menu.1) as _
    }
}

/// A generic child in a menu
#[derive(Debug, Default)]
pub struct MenuChild {
    // shared fields between submenus and menu items
    item_type: MenuItemType,
    id: MenuId,
    text: String,
    enabled: bool,

    ns_menu_items: HashMap<u32, Vec<Retained<NSMenuItem>>>,

    // menu item fields
    key_accelerator: Option<KeyAccelerator>,

    // predefined menu item fields
    predefined_item_type: Option<PredefinedMenuItemType>,

    // check menu item fields
    checked: Cell<bool>,

    // icon menu item fields
    icon: Option<Icon>,
    native_icon: Option<NativeIcon>,

    // submenu fields
    pub children: Option<Vec<Rc<RefCell<MenuChild>>>>,
    ns_menus: Option<HashMap<u32, Vec<NsMenuRef>>>,
    ns_menu: Option<NsMenuRef>,
}

impl Drop for MenuChild {
    fn drop(&mut self) {
        fn drop_children(id: u32, children: &Vec<Rc<RefCell<MenuChild>>>) {
            for child in children {
                let mut child_ = child.borrow_mut();
                child_.ns_menu_items.remove(&id);

                if child_.item_type == MenuItemType::Submenu {
                    if let Some(menus) = child_.ns_menus.as_mut().unwrap().remove(&id) {
                        for menu in menus {
                            drop_children(menu.0, child_.children.as_ref().unwrap());
                        }
                    }
                }
            }
        }

        if self.item_type == MenuItemType::Submenu {
            for menus in self.ns_menus.as_ref().unwrap().values() {
                for menu in menus {
                    drop_children(menu.0, self.children.as_ref().unwrap())
                }
            }

            if let Some(menu) = &self.ns_menu {
                drop_children(menu.0, self.children.as_ref().unwrap());
            }
        }
    }
}

/// Constructors
impl MenuChild {
    pub fn new(
        text: &str,
        enabled: bool,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        Self {
            item_type: MenuItemType::MenuItem,
            text: strip_mnemonic(text),
            enabled,
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            key_accelerator,
            checked: Cell::new(false),
            children: None,
            icon: None,
            native_icon: None,
            ns_menu: None,
            ns_menu_items: HashMap::new(),
            ns_menus: None,
            predefined_item_type: None,
        }
    }

    pub fn new_submenu(text: &str, enabled: bool, id: Option<MenuId>) -> Self {
        let mtm = if cfg!(test) {
            unsafe { MainThreadMarker::new_unchecked() }
        } else {
            MainThreadMarker::new()
                .expect("`muda::MenuChild` can only be created on the main thread")
        };
        Self {
            item_type: MenuItemType::Submenu,
            text: strip_mnemonic(text),
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            enabled,
            children: Some(Vec::new()),
            ns_menu: Some({
                let menu = NSMenu::new(mtm);
                unsafe { menu.setAutoenablesItems(false) };
                NsMenuRef::new(mtm, COUNTER.next(), menu)
            }),
            key_accelerator: None,
            checked: Cell::new(false),
            icon: None,
            native_icon: None,
            ns_menu_items: HashMap::new(),
            ns_menus: Some(HashMap::new()),
            predefined_item_type: None,
        }
    }

    pub(crate) fn new_predefined(item_type: PredefinedMenuItemType, text: Option<String>) -> Self {
        let text = strip_mnemonic(text.unwrap_or_else(|| {
            // Gets the app's name from `NSRunningApplication::localizedName`.
            let app_name = || unsafe {
                let app = NSRunningApplication::currentApplication();
                app.localizedName().unwrap_or_default()
            };

            match item_type {
                PredefinedMenuItemType::About(_) => {
                    format!("About {}", app_name()).trim().to_string()
                }
                PredefinedMenuItemType::Hide => format!("Hide {}", app_name()).trim().to_string(),
                PredefinedMenuItemType::Quit => format!("Quit {}", app_name()).trim().to_string(),
                _ => item_type.text().to_string(),
            }
        }));

        Self {
            item_type: MenuItemType::Predefined,
            text,
            enabled: true,
            id: MenuId(COUNTER.next().to_string()),
            key_accelerator: item_type.accelerator().map(KeyAccelerator::from),
            predefined_item_type: Some(item_type),
            checked: Cell::new(false),
            children: None,
            icon: None,
            native_icon: None,
            ns_menu: None,
            ns_menu_items: HashMap::new(),
            ns_menus: None,
        }
    }

    pub fn new_check(
        text: &str,
        enabled: bool,
        checked: bool,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        Self {
            item_type: MenuItemType::Check,
            text: text.to_string(),
            enabled,
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            key_accelerator,
            checked: Cell::new(checked),
            children: None,
            icon: None,
            native_icon: None,
            ns_menu: None,
            ns_menu_items: HashMap::new(),
            ns_menus: None,
            predefined_item_type: None,
        }
    }

    pub fn new_icon(
        text: &str,
        enabled: bool,
        icon: Option<Icon>,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        Self {
            item_type: MenuItemType::Icon,
            text: text.to_string(),
            enabled,
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            icon,
            key_accelerator,
            checked: Cell::new(false),
            children: None,
            native_icon: None,
            ns_menu: None,
            ns_menu_items: HashMap::new(),
            ns_menus: None,
            predefined_item_type: None,
        }
    }

    pub fn new_native_icon(
        text: &str,
        enabled: bool,
        native_icon: Option<NativeIcon>,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        Self {
            item_type: MenuItemType::Icon,
            text: text.to_string(),
            enabled,
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            native_icon,
            key_accelerator,
            checked: Cell::new(false),
            children: None,
            icon: None,
            ns_menu: None,
            ns_menu_items: HashMap::new(),
            ns_menus: None,
            predefined_item_type: None,
        }
    }
}

/// Shared methods
impl MenuChild {
    pub(crate) fn item_type(&self) -> MenuItemType {
        self.item_type
    }

    pub fn id(&self) -> &MenuId {
        &self.id
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = strip_mnemonic(text);
        unsafe {
            let title = NSString::from_str(&self.text);
            for ns_items in self.ns_menu_items.values() {
                for ns_item in ns_items {
                    ns_item.setTitle(&title);
                    if let Some(submenu) = ns_item.submenu() {
                        submenu.setTitle(&title);
                    }
                }
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        for ns_items in self.ns_menu_items.values() {
            for ns_item in ns_items {
                unsafe { ns_item.setEnabled(enabled) };
            }
        }
    }

    pub fn set_key_accelerator(
        &mut self,
        key_accelerator: Option<KeyAccelerator>,
    ) -> crate::Result<()> {
        let key_equivalent = key_accelerator
            .as_ref()
            .map(KeyAccelerator::key_equivalent)
            .transpose()?;

        if let Some(key_equivalent) = key_equivalent {
            let key_equivalent = NSString::from_str(key_equivalent.as_str());

            let modifier_mask = key_accelerator
                .as_ref()
                .map(KeyAccelerator::modifier_mask)
                .unwrap_or_else(NSEventModifierFlags::empty);

            for ns_items in self.ns_menu_items.values() {
                for ns_item in ns_items {
                    unsafe {
                        ns_item.setKeyEquivalent(&key_equivalent);
                        ns_item.setKeyEquivalentModifierMask(modifier_mask);
                    }
                }
            }
        }

        self.key_accelerator = key_accelerator;

        Ok(())
    }
}

/// CheckMenuItem methods
impl MenuChild {
    pub fn is_checked(&self) -> bool {
        self.checked.get()
    }

    pub fn set_checked(&self, checked: bool) {
        self.checked.set(checked);
        let state = if checked {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        };
        for ns_items in self.ns_menu_items.values() {
            for ns_item in ns_items {
                unsafe {
                    ns_item.setState(state);
                }
            }
        }
    }
}

/// IconMenuItem methods
impl MenuChild {
    pub fn set_icon(&mut self, icon: Option<Icon>) {
        self.icon.clone_from(&icon);
        self.native_icon = None;
        for ns_items in self.ns_menu_items.values() {
            for ns_item in ns_items {
                menuitem_set_icon(ns_item, icon.as_ref());
            }
        }
    }

    pub fn set_native_icon(&mut self, icon: Option<NativeIcon>) {
        self.native_icon = icon;
        self.icon = None;
        for ns_items in self.ns_menu_items.values() {
            for ns_item in ns_items {
                menuitem_set_native_icon(ns_item, icon);
            }
        }
    }
}

/// Submenu methods
impl MenuChild {
    pub fn add_menu_item(&mut self, item: &dyn crate::IsMenuItem, op: AddOp) -> crate::Result<()> {
        let child = item.child();

        unsafe {
            match op {
                AddOp::Append => {
                    for menus in self.ns_menus.as_ref().unwrap().values() {
                        for ns_menu in menus {
                            let ns_menu_item = item.make_ns_item_for_menu(ns_menu.0)?;
                            ns_menu.1.addItem(&ns_menu_item);
                        }
                    }

                    let ns_menu_item =
                        item.make_ns_item_for_menu(self.ns_menu.as_ref().unwrap().0)?;
                    self.ns_menu.as_ref().unwrap().1.addItem(&ns_menu_item);

                    self.children.as_mut().unwrap().push(child);
                }
                AddOp::Insert(position) => {
                    for menus in self.ns_menus.as_ref().unwrap().values() {
                        for ns_menu in menus {
                            let ns_menu_item = item.make_ns_item_for_menu(ns_menu.0)?;
                            ns_menu
                                .1
                                .insertItem_atIndex(&ns_menu_item, position as NSInteger);
                        }
                    }

                    let ns_menu_item =
                        item.make_ns_item_for_menu(self.ns_menu.as_ref().unwrap().0)?;
                    self.ns_menu
                        .as_ref()
                        .unwrap()
                        .1
                        .insertItem_atIndex(&ns_menu_item, position as NSInteger);

                    self.children.as_mut().unwrap().insert(position, child);
                }
            }
        }

        Ok(())
    }

    pub fn remove(&mut self, item: &dyn crate::IsMenuItem) -> crate::Result<()> {
        self.remove_inner(item, true, None)
    }
    pub fn remove_inner(
        &mut self,
        item: &dyn crate::IsMenuItem,
        remove_from_cache: bool,
        id: Option<u32>,
    ) -> crate::Result<()> {
        // get child
        let child = {
            let index = self
                .children
                .as_ref()
                .unwrap()
                .iter()
                .position(|e| e.borrow().id == item.id())
                .ok_or(crate::Error::NotAChildOfThisMenu)?;
            if remove_from_cache {
                self.children.as_mut().unwrap().remove(index)
            } else {
                self.children.as_ref().unwrap().get(index).cloned().unwrap()
            }
        };

        for menus in self.ns_menus.as_ref().unwrap().values() {
            for menu in menus {
                // check if we are removing this item from all ns_menus
                //      which is usually when this is the item the user is actually removing
                // or if we are removing from a specific menu (id)
                //      which is when the actual item being removed is a submenu
                //      and we are iterating through its children and removing
                //      each child ns menu item that are related to this submenu.
                if id.map(|i| i == menu.0).unwrap_or(true) {
                    let mut child_ = child.borrow_mut();

                    if child_.item_type == MenuItemType::Submenu {
                        let menus = child_.ns_menus.as_ref().unwrap().get(&menu.0).cloned();
                        if let Some(menus) = menus {
                            for menu in menus {
                                // iterate through children and only remove the ns menu items
                                // related to this submenu
                                for item in child_.items() {
                                    child_.remove_inner(item.as_ref(), false, Some(menu.0))?;
                                }
                            }
                        }
                        child_.ns_menus.as_mut().unwrap().remove(&menu.0);
                    }

                    if let Some(items) = child_.ns_menu_items.remove(&menu.0) {
                        for item in items {
                            unsafe { menu.1.removeItem(&item) };
                        }
                    }
                }
            }
        }

        if remove_from_cache {
            if let Some(ns_menu_items) = child
                .borrow_mut()
                .ns_menu_items
                .remove(&self.ns_menu.as_ref().unwrap().0)
            {
                for item in ns_menu_items {
                    unsafe { self.ns_menu.as_ref().unwrap().1.removeItem(&item) };
                }
            }
        }

        Ok(())
    }

    pub fn items(&self) -> Vec<MenuItemKind> {
        self.children
            .as_ref()
            .unwrap()
            .iter()
            .map(|c| c.borrow().kind(c.clone()))
            .collect()
    }

    pub unsafe fn show_context_menu_for_nsview(
        &self,
        view: *const c_void,
        position: Option<Position>,
    ) -> bool {
        show_context_menu(&self.ns_menu.as_ref().unwrap().1, view, position)
    }

    pub fn set_as_windows_menu_for_nsapp(&self) {
        let Some(menu) = self.resolve_ns_menu_for_nsapp() else {
            return;
        };

        let mtm = MainThreadMarker::from(&*menu);
        let app = NSApplication::sharedApplication(mtm);
        unsafe { app.setWindowsMenu(Some(&menu)) }
    }

    pub fn set_as_help_menu_for_nsapp(&self) {
        let Some(menu) = self.resolve_ns_menu_for_nsapp() else {
            return;
        };

        let mtm = MainThreadMarker::from(&*menu);
        let app = NSApplication::sharedApplication(mtm);
        unsafe { app.setHelpMenu(Some(&menu)) }
    }

    /// Finds the NSMenu instance for this submenu that is attached to the
    /// current NSApp main menu, by reading the menu id stored in the
    /// main menu's delegate.
    fn resolve_ns_menu_for_nsapp(&self) -> Option<Retained<NSMenu>> {
        let ns_menu = &self.ns_menu.as_ref().unwrap().1;
        let mtm = MainThreadMarker::from(&**ns_menu);
        let app = NSApplication::sharedApplication(mtm);
        let main_menu = unsafe { app.mainMenu()? };
        let delegate = unsafe { main_menu.delegate()? };

        // Downcast the delegate to our MudaMenuDelegate to get the menu id
        let delegate_obj: &AnyObject = ProtocolObject::as_ref(&*delegate);
        let muda_delegate: &MudaMenuDelegate = delegate_obj.downcast_ref()?;
        let parent_id = muda_delegate.menu_id();

        // Look up the NSMenu in ns_menus for this parent id
        self.ns_menus
            .as_ref()
            .unwrap()
            .get(&parent_id)
            // A submenu can be added multiple times to the same parent menu
            // lets just take the first one we find
            .and_then(|menus| menus.first())
            .map(|menu_ref| menu_ref.1.clone())
    }

    pub fn ns_menu(&self) -> *mut std::ffi::c_void {
        Retained::as_ptr(&self.ns_menu.as_ref().unwrap().1) as *mut _
    }
}

/// NSMenuItem item creation methods
impl MenuChild {
    pub fn create_ns_item_for_submenu(
        &mut self,
        menu_id: u32,
    ) -> crate::Result<Retained<NSMenuItem>> {
        let mtm = MainThreadMarker::new().expect("can only create menu item on the main thread");
        let ns_menu_item;
        let ns_submenu;

        let title = NSString::from_str(&self.text);
        unsafe {
            ns_menu_item = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &title,
                None,
                &NSString::new(),
            );
            ns_submenu = NSMenu::new(mtm);
            ns_submenu.setTitle(&title);

            ns_menu_item.setSubmenu(Some(&ns_submenu));
            ns_submenu.setAutoenablesItems(false);

            ns_menu_item.setEnabled(self.enabled);

            if let Some(native_icon) = self.native_icon {
                menuitem_set_native_icon(&ns_menu_item, Some(native_icon));
            }

            if let Some(icon) = self.icon.as_ref() {
                menuitem_set_icon(&ns_menu_item, Some(icon));
            }
        }

        let id = COUNTER.next();

        for item in self.children.as_ref().unwrap() {
            let ns_item = item.borrow_mut().make_ns_item_for_menu(id)?;
            ns_submenu.addItem(&ns_item);
        }

        self.ns_menus
            .as_mut()
            .unwrap()
            .entry(menu_id)
            .or_default()
            .push(NsMenuRef::new(mtm, id, ns_submenu));

        self.ns_menu_items
            .entry(menu_id)
            .or_default()
            .push(ns_menu_item.retain());

        Ok(ns_menu_item)
    }

    pub fn create_ns_item_for_menu_item(
        &mut self,
        menu_id: u32,
    ) -> crate::Result<Retained<NSMenuItem>> {
        let mtm = MainThreadMarker::new().expect("can only create menu item on the main thread");
        let ns_menu_item = MenuItem::create(
            mtm,
            &self.text,
            Some(sel!(fireMenuItemAction:)),
            &self.key_accelerator,
        )?;

        unsafe {
            ns_menu_item.setTarget(Some(&ns_menu_item));

            // Store a raw pointer to the `MenuChild` as an instance variable on the native menu item
            ns_menu_item.ivars().set(&*self);

            ns_menu_item.setEnabled(self.enabled);
        }

        self.ns_menu_items
            .entry(menu_id)
            .or_default()
            .push(Retained::into_super(ns_menu_item.retain()));

        Ok(Retained::into_super(ns_menu_item))
    }

    pub fn create_ns_item_for_predefined_menu_item(
        &mut self,
        menu_id: u32,
    ) -> crate::Result<Retained<NSMenuItem>> {
        let mtm = MainThreadMarker::new().expect("can only create menu item on the main thread");
        let item_type = self.predefined_item_type.as_ref().unwrap();
        let ns_menu_item = match item_type {
            PredefinedMenuItemType::Separator => NSMenuItem::separatorItem(mtm),
            _ => {
                let ns_menu_item =
                    MenuItem::create(mtm, &self.text, item_type.selector(), &self.key_accelerator)?;

                if let PredefinedMenuItemType::About(_) = item_type {
                    unsafe {
                        ns_menu_item.setTarget(Some(&ns_menu_item));

                        // Store a raw pointer to the `MenuChild` as an instance variable on the native menu item
                        ns_menu_item.ivars().set(&*self);
                    }
                }

                Retained::into_super(ns_menu_item)
            }
        };

        unsafe {
            ns_menu_item.setEnabled(self.enabled);

            if let PredefinedMenuItemType::Services = item_type {
                // we have to assign an empty menu as the app's services menu, and macOS will populate it
                let services_menu = NSMenu::new(mtm);
                NSApplication::sharedApplication(mtm).setServicesMenu(Some(&services_menu));
                ns_menu_item.setSubmenu(Some(&services_menu));
            }
        }

        self.ns_menu_items
            .entry(menu_id)
            .or_default()
            .push(ns_menu_item.retain());

        Ok(ns_menu_item)
    }

    pub fn create_ns_item_for_check_menu_item(
        &mut self,
        menu_id: u32,
    ) -> crate::Result<Retained<NSMenuItem>> {
        let mtm = MainThreadMarker::new().expect("can only create menu item on the main thread");
        let ns_menu_item = MenuItem::create(
            mtm,
            &self.text,
            Some(sel!(fireMenuItemAction:)),
            &self.key_accelerator,
        )?;

        unsafe {
            ns_menu_item.setTarget(Some(&ns_menu_item));

            // Store a raw pointer to the `MenuChild` as an instance variable on the native menu item
            ns_menu_item.ivars().set(&*self);

            ns_menu_item.setEnabled(self.enabled);
            if self.checked.get() {
                ns_menu_item.setState(NSControlStateValueOn);
            }
        }

        self.ns_menu_items
            .entry(menu_id)
            .or_default()
            .push(Retained::into_super(ns_menu_item.retain()));

        Ok(Retained::into_super(ns_menu_item))
    }

    pub fn create_ns_item_for_icon_menu_item(
        &mut self,
        menu_id: u32,
    ) -> crate::Result<Retained<NSMenuItem>> {
        let mtm = MainThreadMarker::new().expect("can only create menu item on the main thread");
        let ns_menu_item = MenuItem::create(
            mtm,
            &self.text,
            Some(sel!(fireMenuItemAction:)),
            &self.key_accelerator,
        )?;

        unsafe {
            ns_menu_item.setTarget(Some(&ns_menu_item));

            // Store a raw pointer to the `MenuChild` as an instance variable on the native menu item
            ns_menu_item.ivars().set(&*self);

            ns_menu_item.setEnabled(self.enabled);

            if self.icon.is_some() {
                menuitem_set_icon(&ns_menu_item, self.icon.as_ref());
            } else if self.native_icon.is_some() {
                menuitem_set_native_icon(&ns_menu_item, self.native_icon);
            }
        }

        self.ns_menu_items
            .entry(menu_id)
            .or_default()
            .push(Retained::into_super(ns_menu_item.retain()));

        Ok(Retained::into_super(ns_menu_item))
    }

    fn make_ns_item_for_menu(&mut self, menu_id: u32) -> crate::Result<Retained<NSMenuItem>> {
        match self.item_type {
            MenuItemType::Submenu => self.create_ns_item_for_submenu(menu_id),
            MenuItemType::MenuItem => self.create_ns_item_for_menu_item(menu_id),
            MenuItemType::Predefined => self.create_ns_item_for_predefined_menu_item(menu_id),
            MenuItemType::Check => self.create_ns_item_for_check_menu_item(menu_id),
            MenuItemType::Icon => self.create_ns_item_for_icon_menu_item(menu_id),
        }
    }
}

impl PredefinedMenuItemType {
    pub(crate) fn selector(&self) -> Option<Sel> {
        match self {
            PredefinedMenuItemType::Separator => None,
            PredefinedMenuItemType::Copy => Some(sel!(copy:)),
            PredefinedMenuItemType::Cut => Some(sel!(cut:)),
            PredefinedMenuItemType::Paste => Some(sel!(paste:)),
            PredefinedMenuItemType::SelectAll => Some(sel!(selectAll:)),
            PredefinedMenuItemType::Undo => Some(sel!(undo:)),
            PredefinedMenuItemType::Redo => Some(sel!(redo:)),
            PredefinedMenuItemType::Minimize => Some(sel!(performMiniaturize:)),
            PredefinedMenuItemType::Maximize => Some(sel!(performZoom:)),
            PredefinedMenuItemType::Fullscreen => Some(sel!(toggleFullScreen:)),
            PredefinedMenuItemType::Hide => Some(sel!(hide:)),
            PredefinedMenuItemType::HideOthers => Some(sel!(hideOtherApplications:)),
            PredefinedMenuItemType::ShowAll => Some(sel!(unhideAllApplications:)),
            PredefinedMenuItemType::CloseWindow => Some(sel!(performClose:)),
            PredefinedMenuItemType::Quit => Some(sel!(terminate:)),
            // manual implementation in `fire_menu_item_click`
            PredefinedMenuItemType::About(_) => Some(sel!(fireMenuItemAction:)),
            PredefinedMenuItemType::Services => None,
            PredefinedMenuItemType::BringAllToFront => Some(sel!(arrangeInFront:)),
            PredefinedMenuItemType::None => None,
        }
    }
}

impl dyn IsMenuItem + '_ {
    fn make_ns_item_for_menu(&self, menu_id: u32) -> crate::Result<Retained<NSMenuItem>> {
        match self.kind() {
            MenuItemKind::Submenu(i) => i.inner.borrow_mut().create_ns_item_for_submenu(menu_id),
            MenuItemKind::MenuItem(i) => i.inner.borrow_mut().create_ns_item_for_menu_item(menu_id),
            MenuItemKind::Predefined(i) => i
                .inner
                .borrow_mut()
                .create_ns_item_for_predefined_menu_item(menu_id),
            MenuItemKind::Check(i) => i
                .inner
                .borrow_mut()
                .create_ns_item_for_check_menu_item(menu_id),
            MenuItemKind::Icon(i) => i
                .inner
                .borrow_mut()
                .create_ns_item_for_icon_menu_item(menu_id),
        }
    }
}

define_class!(
    #[unsafe(super(NSMenuItem))]
    #[name = "MudaMenuItem"]
    #[thread_kind = MainThreadOnly]
    // FIXME: Use `Rc` or something else to access the MenuChild.
    #[ivars = Cell<*const MenuChild>]
    struct MenuItem;

    impl MenuItem {
        #[unsafe(method(fireMenuItemAction:))]
        fn fire_menu_item_action(&self, _sender: Option<&AnyObject>) {
            self.fire_menu_item_click();
        }
    }
);

impl MenuItem {
    fn new(
        mtm: MainThreadMarker,
        title: &NSString,
        action: Option<Sel>,
        key_equivalent: &NSString,
    ) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(Cell::new(ptr::null()));
        unsafe {
            msg_send![super(this), initWithTitle: title, action: action, keyEquivalent: key_equivalent]
        }
    }

    fn fire_menu_item_click(&self) {
        let mtm = MainThreadMarker::from(self);
        // Create a reference to the `MenuChild` from the raw pointer
        // stored as an instance variable on the native menu item
        let item =
            unsafe { self.ivars().get().as_ref() }.expect("MenuItem's MenuChild pointer was unset");

        if let Some(PredefinedMenuItemType::About(about_meta)) = &item.predefined_item_type {
            match about_meta {
                Some(about_meta) => {
                    let mut keys: Vec<&NSString> = Default::default();
                    let mut objects: Vec<Retained<AnyObject>> = Default::default();

                    if let Some(name) = &about_meta.name {
                        keys.push(unsafe { NSAboutPanelOptionApplicationName });
                        objects.push(Retained::into_super(Retained::into_super(
                            NSString::from_str(name),
                        )));
                    }

                    if let Some(version) = &about_meta.version {
                        keys.push(unsafe { NSAboutPanelOptionApplicationVersion });
                        objects.push(Retained::into_super(Retained::into_super(
                            NSString::from_str(version),
                        )));
                    }

                    if let Some(short_version) = &about_meta.short_version {
                        keys.push(unsafe { NSAboutPanelOptionVersion });
                        objects.push(Retained::into_super(Retained::into_super(
                            NSString::from_str(short_version),
                        )));
                    }

                    if let Some(copyright) = &about_meta.copyright {
                        keys.push(ns_string!(NSAboutPanelOptionCopyright));
                        objects.push(Retained::into_super(Retained::into_super(
                            NSString::from_str(copyright),
                        )));
                    }

                    if let Some(icon) = &about_meta.icon {
                        keys.push(unsafe { NSAboutPanelOptionApplicationIcon });
                        objects.push(Retained::into_super(Retained::into_super(
                            icon.inner.to_nsimage(None),
                        )));
                    }

                    if let Some(credits) = &about_meta.credits {
                        keys.push(unsafe { NSAboutPanelOptionCredits });
                        objects.push(Retained::into_super(Retained::into_super(
                            NSAttributedString::from_nsstring(&NSString::from_str(credits)),
                        )));
                    }

                    let dict = NSDictionary::from_retained_objects(&keys, &objects);

                    unsafe {
                        NSApplication::sharedApplication(mtm)
                            .orderFrontStandardAboutPanelWithOptions(&dict)
                    };
                }

                None => {
                    unsafe {
                        NSApplication::sharedApplication(mtm)
                            .orderFrontStandardAboutPanel(Some(self))
                    };
                }
            }
        } else {
            if item.item_type == MenuItemType::Check {
                item.set_checked(!item.is_checked());
            }

            let id = (*item).id().clone();
            MenuEvent::send(crate::MenuEvent { id });
        }
    }

    fn create(
        mtm: MainThreadMarker,
        title: &str,
        selector: Option<Sel>,
        key_accelerator: &Option<KeyAccelerator>,
    ) -> crate::Result<Retained<MenuItem>> {
        let title = NSString::from_str(title);

        let key_equivalent = key_accelerator
            .as_ref()
            .map(|accel| accel.key_equivalent())
            .transpose()?
            .unwrap_or_default();
        let key_equivalent = NSString::from_str(&key_equivalent);

        let modifier_mask = key_accelerator
            .as_ref()
            .map(KeyAccelerator::modifier_mask)
            .unwrap_or_else(NSEventModifierFlags::empty);

        let item = MenuItem::new(mtm, &title, selector, &key_equivalent);
        item.setKeyEquivalentModifierMask(modifier_mask);

        Ok(item)
    }
}

fn menuitem_set_icon(menuitem: &NSMenuItem, icon: Option<&Icon>) {
    if let Some(icon) = icon {
        unsafe {
            let nsimage = icon.inner.to_nsimage(Some(18.));
            menuitem.setImage(Some(&nsimage));
        }
    } else {
        unsafe {
            menuitem.setImage(None);
        }
    }
}

fn menuitem_set_native_icon(menuitem: &NSMenuItem, icon: Option<NativeIcon>) {
    if let Some(icon) = icon {
        unsafe {
            let named_img = icon.named_img();
            let nsimage = NSImage::imageNamed(named_img).unwrap();
            let size = NSSize::new(18.0, 18.0);
            nsimage.setSize(size);
            menuitem.setImage(Some(&nsimage));
        }
    } else {
        unsafe {
            menuitem.setImage(None);
        }
    }
}

unsafe fn show_context_menu(
    ns_menu: &NSMenu,
    view: *const c_void,
    position: Option<Position>,
) -> bool {
    // SAFETY: Caller verifies that the view is valid.
    let view: &NSView = unsafe { &*view.cast() };

    let window = view.window().expect("view must be installed in a window");
    let scale_factor = window.backingScaleFactor();
    let (location, in_view) = if let Some(pos) = position.map(|p| p.to_logical(scale_factor)) {
        let view_rect = view.frame();
        let location = NSPoint::new(pos.x, view_rect.size.height - pos.y);
        (location, Some(view))
    } else {
        let mouse_location = unsafe { NSEvent::mouseLocation() };
        let pos = LogicalPosition {
            x: mouse_location.x,
            y: mouse_location.y,
        };
        let location = NSPoint::new(pos.x, pos.y);
        (location, None)
    };

    unsafe { ns_menu.popUpMenuPositioningItem_atLocation_inView(None, location, in_view) }
}

impl NativeIcon {
    unsafe fn named_img(self) -> &'static NSImageName {
        use objc2_app_kit as appkit;
        match self {
            NativeIcon::Add => appkit::NSImageNameAddTemplate,
            NativeIcon::StatusAvailable => appkit::NSImageNameStatusAvailable,
            NativeIcon::StatusUnavailable => appkit::NSImageNameStatusUnavailable,
            NativeIcon::StatusPartiallyAvailable => appkit::NSImageNameStatusPartiallyAvailable,
            NativeIcon::Advanced => appkit::NSImageNameAdvanced,
            NativeIcon::Bluetooth => appkit::NSImageNameBluetoothTemplate,
            NativeIcon::Bookmarks => appkit::NSImageNameBookmarksTemplate,
            NativeIcon::Caution => appkit::NSImageNameCaution,
            NativeIcon::ColorPanel => appkit::NSImageNameColorPanel,
            NativeIcon::ColumnView => appkit::NSImageNameColumnViewTemplate,
            NativeIcon::Computer => appkit::NSImageNameComputer,
            NativeIcon::EnterFullScreen => appkit::NSImageNameEnterFullScreenTemplate,
            NativeIcon::Everyone => appkit::NSImageNameEveryone,
            NativeIcon::ExitFullScreen => appkit::NSImageNameExitFullScreenTemplate,
            NativeIcon::FlowView => appkit::NSImageNameFlowViewTemplate,
            NativeIcon::Folder => appkit::NSImageNameFolder,
            NativeIcon::FolderBurnable => appkit::NSImageNameFolderBurnable,
            NativeIcon::FolderSmart => appkit::NSImageNameFolderSmart,
            NativeIcon::FollowLinkFreestanding => appkit::NSImageNameFollowLinkFreestandingTemplate,
            NativeIcon::FontPanel => appkit::NSImageNameFontPanel,
            NativeIcon::GoLeft => appkit::NSImageNameGoLeftTemplate,
            NativeIcon::GoRight => appkit::NSImageNameGoRightTemplate,
            NativeIcon::Home => appkit::NSImageNameHomeTemplate,
            NativeIcon::IChatTheater => appkit::NSImageNameIChatTheaterTemplate,
            NativeIcon::IconView => appkit::NSImageNameIconViewTemplate,
            NativeIcon::Info => appkit::NSImageNameInfo,
            NativeIcon::InvalidDataFreestanding => {
                appkit::NSImageNameInvalidDataFreestandingTemplate
            }
            NativeIcon::LeftFacingTriangle => appkit::NSImageNameLeftFacingTriangleTemplate,
            NativeIcon::ListView => appkit::NSImageNameListViewTemplate,
            NativeIcon::LockLocked => appkit::NSImageNameLockLockedTemplate,
            NativeIcon::LockUnlocked => appkit::NSImageNameLockUnlockedTemplate,
            NativeIcon::MenuMixedState => appkit::NSImageNameMenuMixedStateTemplate,
            NativeIcon::MenuOnState => appkit::NSImageNameMenuOnStateTemplate,
            NativeIcon::MobileMe => appkit::NSImageNameMobileMe,
            NativeIcon::MultipleDocuments => appkit::NSImageNameMultipleDocuments,
            NativeIcon::Network => appkit::NSImageNameNetwork,
            NativeIcon::Path => appkit::NSImageNamePathTemplate,
            NativeIcon::PreferencesGeneral => appkit::NSImageNamePreferencesGeneral,
            NativeIcon::QuickLook => appkit::NSImageNameQuickLookTemplate,
            NativeIcon::RefreshFreestanding => appkit::NSImageNameRefreshFreestandingTemplate,
            NativeIcon::Refresh => appkit::NSImageNameRefreshTemplate,
            NativeIcon::Remove => appkit::NSImageNameRemoveTemplate,
            NativeIcon::RevealFreestanding => appkit::NSImageNameRevealFreestandingTemplate,
            NativeIcon::RightFacingTriangle => appkit::NSImageNameRightFacingTriangleTemplate,
            NativeIcon::Share => appkit::NSImageNameShareTemplate,
            NativeIcon::Slideshow => appkit::NSImageNameSlideshowTemplate,
            NativeIcon::SmartBadge => appkit::NSImageNameSmartBadgeTemplate,
            NativeIcon::StatusNone => appkit::NSImageNameStatusNone,
            NativeIcon::StopProgressFreestanding => {
                appkit::NSImageNameStopProgressFreestandingTemplate
            }
            NativeIcon::StopProgress => appkit::NSImageNameStopProgressTemplate,
            NativeIcon::TrashEmpty => appkit::NSImageNameTrashEmpty,
            NativeIcon::TrashFull => appkit::NSImageNameTrashFull,
            NativeIcon::User => appkit::NSImageNameUser,
            NativeIcon::UserAccounts => appkit::NSImageNameUserAccounts,
            NativeIcon::UserGroup => appkit::NSImageNameUserGroup,
            NativeIcon::UserGuest => appkit::NSImageNameUserGuest,
        }
    }
}
