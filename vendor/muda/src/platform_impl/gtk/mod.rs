// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod accelerator;
mod icon;

pub(crate) use icon::PlatformIcon;

use crate::{
    accelerator::KeyAccelerator,
    dpi::Position,
    icon::{Icon, NativeIcon},
    items::*,
    util::{AddOp, Counter},
    IsMenuItem, MenuEvent, MenuId, MenuItemKind, MenuItemType,
};
use accelerator::{from_gtk_mnemonic, parse_key_accelerator, to_gtk_mnemonic};
use glib::translate::ToGlibPtr;
use gtk::{gdk, glib, prelude::*, AboutDialog, Container, Orientation};
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

static COUNTER: Counter = Counter::new();

macro_rules! is_item_supported {
    ($item:tt) => {{
        let child = $item.child();
        let child_ = child.borrow();
        let supported = if let Some(predefined_item_type) = &child_.predefined_item_type {
            matches!(
                predefined_item_type,
                PredefinedMenuItemType::Separator
                    | PredefinedMenuItemType::Copy
                    | PredefinedMenuItemType::Cut
                    | PredefinedMenuItemType::Paste
                    | PredefinedMenuItemType::SelectAll
                    | PredefinedMenuItemType::About(_)
            )
        } else {
            true
        };
        drop(child_);
        supported
    }};
}

macro_rules! return_if_item_not_supported {
    ($item:tt) => {
        if !is_item_supported!($item) {
            return Ok(());
        }
    };
}

pub struct Menu {
    id: MenuId,
    children: Vec<Rc<RefCell<MenuChild>>>,
    // TODO: maybe save a reference to the window?
    gtk_menubars: HashMap<u32, gtk::MenuBar>,
    accel_group: Option<gtk::AccelGroup>,
    gtk_menu: (u32, Option<gtk::Menu>), // dedicated menu for tray or context menus
}

impl Drop for Menu {
    fn drop(&mut self) {
        for (id, menu) in &self.gtk_menubars {
            drop_children_from_menu_and_destroy(*id, menu, &self.children);
            unsafe { menu.destroy() }
        }

        if let (id, Some(menu)) = &self.gtk_menu {
            drop_children_from_menu_and_destroy(*id, menu, &self.children);
            unsafe { menu.destroy() }
        }
    }
}

impl Menu {
    pub fn new(id: Option<MenuId>) -> Self {
        Self {
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            children: Vec::new(),
            gtk_menubars: HashMap::new(),
            accel_group: None,
            gtk_menu: (COUNTER.next(), None),
        }
    }

    pub fn id(&self) -> &MenuId {
        &self.id
    }

    pub fn add_menu_item(&mut self, item: &dyn crate::IsMenuItem, op: AddOp) -> crate::Result<()> {
        if is_item_supported!(item) {
            for (menu_id, menu_bar) in &self.gtk_menubars {
                let gtk_item =
                    item.make_gtk_menu_item(*menu_id, self.accel_group.as_ref(), true, true)?;
                match op {
                    AddOp::Append => menu_bar.append(&gtk_item),
                    AddOp::Insert(position) => menu_bar.insert(&gtk_item, position as i32),
                }
                gtk_item.show();
            }

            if let (menu_id, Some(menu)) = &self.gtk_menu {
                let gtk_item =
                    item.make_gtk_menu_item(*menu_id, self.accel_group.as_ref(), true, false)?;
                match op {
                    AddOp::Append => menu.append(&gtk_item),
                    AddOp::Insert(position) => menu.insert(&gtk_item, position as i32),
                }
                gtk_item.show();
            }
        }

        match op {
            AddOp::Append => self.children.push(item.child()),
            AddOp::Insert(position) => self.children.insert(position, item.child()),
        }

        Ok(())
    }

    fn add_menu_item_with_id(&self, item: &dyn crate::IsMenuItem, id: u32) -> crate::Result<()> {
        return_if_item_not_supported!(item);

        for (menu_id, menu_bar) in self.gtk_menubars.iter().filter(|m| *m.0 == id) {
            let gtk_item =
                item.make_gtk_menu_item(*menu_id, self.accel_group.as_ref(), true, true)?;
            menu_bar.append(&gtk_item);
            gtk_item.show();
        }

        Ok(())
    }

    fn add_menu_item_to_context_menu(&self, item: &dyn crate::IsMenuItem) -> crate::Result<()> {
        return_if_item_not_supported!(item);

        if let (menu_id, Some(menu)) = &self.gtk_menu {
            let gtk_item =
                item.make_gtk_menu_item(*menu_id, self.accel_group.as_ref(), true, false)?;
            menu.append(&gtk_item);
            gtk_item.show();
        }

        Ok(())
    }

    pub fn remove(&mut self, item: &dyn crate::IsMenuItem) -> crate::Result<()> {
        self.remove_inner(item, true, None)
    }

    fn remove_inner(
        &mut self,
        item: &dyn crate::IsMenuItem,
        remove_from_cache: bool,
        id: Option<u32>,
    ) -> crate::Result<()> {
        // get child
        let child = {
            let index = self
                .children
                .iter()
                .position(|e| e.borrow().id == item.id())
                .ok_or(crate::Error::NotAChildOfThisMenu)?;
            if remove_from_cache {
                self.children.remove(index)
            } else {
                self.children.get(index).cloned().unwrap()
            }
        };

        for (menu_id, menu_bar) in &self.gtk_menubars {
            // check if we are removing this item from all gtk_menubars
            //      which is usually when this is the item the user is actaully removing
            // or if we are removing from a specific menu (id)
            //      which is when the actual item being removed is a submenu
            //      and we are iterating through its children and removing
            //      each child gtk items that are related to this submenu.
            if id.map(|i| i == *menu_id).unwrap_or(true) {
                // bail this is not a supported item like a close_window predefined menu item
                if is_item_supported!(item) {
                    let mut child_ = child.borrow_mut();

                    if child_.item_type == MenuItemType::Submenu {
                        let menus = child_.gtk_menus.as_ref().unwrap().get(menu_id).cloned();
                        if let Some(menus) = menus {
                            for (id, menu) in menus {
                                // iterate through children and only remove the gtk items
                                // related to this submenu
                                for item in child_.items() {
                                    child_.remove_inner(item.as_ref(), false, Some(id))?;
                                }
                                unsafe { menu.destroy() };
                            }
                        }
                        child_.gtk_menus.as_mut().unwrap().remove(menu_id);
                    }

                    // remove all the gtk items that are related to this gtk menubar and destroy it
                    if let Some(items) = child_.gtk_menu_items.borrow_mut().remove(menu_id) {
                        for item in items {
                            menu_bar.remove(&item);
                            if let Some(accel_group) = &child_.accel_group {
                                if let Some((mods, key)) = child_.gtk_accelerator {
                                    item.remove_accelerator(accel_group, key, mods);
                                }
                            }
                            unsafe { item.destroy() };
                        }
                    };
                }
            }
        }

        // remove from the gtk menu assigned to the context menu
        if remove_from_cache {
            if let (id, Some(menu)) = &self.gtk_menu {
                let child_ = child.borrow_mut();
                if let Some(items) = child_.gtk_menu_items.borrow_mut().remove(id) {
                    for item in items {
                        menu.remove(&item);
                        if let Some(accel_group) = &child_.accel_group {
                            if let Some((mods, key)) = child_.gtk_accelerator {
                                item.remove_accelerator(accel_group, key, mods);
                            }
                        }
                        unsafe { item.destroy() };
                    }
                };
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

    pub fn init_for_gtk_window<W, C>(
        &mut self,
        window: &W,
        container: Option<&C>,
    ) -> crate::Result<()>
    where
        W: IsA<gtk::Window>,
        W: IsA<gtk::Container>,
        C: IsA<gtk::Container>,
    {
        let id = window.as_ptr() as u32;

        if self.accel_group.is_none() {
            self.accel_group = Some(gtk::AccelGroup::new());
        }

        // This is the first time this method has been called on this window
        // so we need to create the menubar and its parent box
        if let Entry::Vacant(e) = self.gtk_menubars.entry(id) {
            let menu_bar = gtk::MenuBar::new();
            e.insert(menu_bar);
        } else {
            return Err(crate::Error::AlreadyInitialized);
        }

        // Construct the entries of the menubar
        let menu_bar = &self.gtk_menubars[&id];

        window.add_accel_group(self.accel_group.as_ref().unwrap());

        for item in self.items() {
            self.add_menu_item_with_id(item.as_ref(), id)?;
        }

        // add the menubar to the specified widget, otherwise to the window
        if let Some(container) = container {
            if container.type_().name() == "GtkBox" {
                let gtk_box = container.dynamic_cast_ref::<gtk::Box>().unwrap();
                gtk_box.pack_start(menu_bar, false, false, 0);
                gtk_box.reorder_child(menu_bar, 0);
            } else {
                container.add(menu_bar);
            }
        } else {
            window.add(menu_bar);
        }

        // Show the menubar
        menu_bar.show();

        Ok(())
    }

    pub fn remove_for_gtk_window<W>(&mut self, window: &W) -> crate::Result<()>
    where
        W: IsA<gtk::Window>,
    {
        let id = window.as_ptr() as u32;

        // Remove from our cache
        let menu_bar = self
            .gtk_menubars
            .remove(&id)
            .ok_or(crate::Error::NotInitialized)?;

        for item in self.items() {
            let _ = self.remove_inner(item.as_ref(), false, Some(id));
        }

        // Remove the [`gtk::Menubar`] from the widget tree
        unsafe { menu_bar.destroy() };
        // Detach the accelerators from the window
        window.remove_accel_group(self.accel_group.as_ref().unwrap());
        Ok(())
    }

    pub fn hide_for_gtk_window<W>(&mut self, window: &W) -> crate::Result<()>
    where
        W: IsA<gtk::Window>,
    {
        self.gtk_menubars
            .get(&(window.as_ptr() as u32))
            .ok_or(crate::Error::NotInitialized)?
            .hide();
        Ok(())
    }

    pub fn show_for_gtk_window<W>(&self, window: &W) -> crate::Result<()>
    where
        W: IsA<gtk::Window>,
    {
        self.gtk_menubars
            .get(&(window.as_ptr() as u32))
            .ok_or(crate::Error::NotInitialized)?
            .show_all();
        Ok(())
    }

    pub fn is_visible_on_gtk_window<W>(&self, window: &W) -> bool
    where
        W: IsA<gtk::Window>,
    {
        self.gtk_menubars
            .get(&(window.as_ptr() as u32))
            .map(|m| m.get_visible())
            .unwrap_or(false)
    }

    pub fn gtk_menubar_for_gtk_window<W>(&self, window: &W) -> Option<gtk::MenuBar>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        self.gtk_menubars.get(&(window.as_ptr() as u32)).cloned()
    }

    pub fn show_context_menu_for_gtk_window(
        &mut self,
        widget: &impl IsA<gtk::Widget>,
        position: Option<Position>,
    ) -> bool {
        show_context_menu(self.gtk_context_menu(), widget, position)
    }

    pub fn gtk_context_menu(&mut self) -> gtk::Menu {
        let mut add_items = false;

        {
            if self.gtk_menu.1.is_none() {
                self.gtk_menu.1 = Some(gtk::Menu::new());
                add_items = true;
            }
        }

        if add_items {
            for item in self.items() {
                self.add_menu_item_to_context_menu(item.as_ref()).unwrap();
            }
        }

        self.gtk_menu.1.as_ref().unwrap().clone()
    }
}

/// A generic child in a menu
#[derive(Debug, Default)]
pub struct MenuChild {
    // shared fields between submenus and menu items
    item_type: MenuItemType,
    text: String,
    enabled: bool,
    id: MenuId,

    gtk_menu_items: Rc<RefCell<HashMap<u32, Vec<gtk::MenuItem>>>>,

    // menu item fields
    accelerator: Option<KeyAccelerator>,
    gtk_accelerator: Option<(gdk::ModifierType, u32)>,

    // predefined menu item fields
    predefined_item_type: Option<PredefinedMenuItemType>,

    // check menu item fields
    checked: Option<Rc<AtomicBool>>,
    is_syncing_checked_state: Option<Rc<AtomicBool>>,

    // icon menu item fields
    icon: Option<Icon>,

    // submenu fields
    pub children: Option<Vec<Rc<RefCell<MenuChild>>>>,
    gtk_menus: Option<HashMap<u32, Vec<(u32, gtk::Menu)>>>,
    gtk_menu: Option<(u32, Option<gtk::Menu>)>, // dedicated menu for tray or context menus
    accel_group: Option<gtk::AccelGroup>,
}

impl Drop for MenuChild {
    fn drop(&mut self) {
        if self.item_type == MenuItemType::Submenu {
            for menus in self.gtk_menus.as_ref().unwrap().values() {
                for (id, menu) in menus {
                    drop_children_from_menu_and_destroy(*id, menu, self.children.as_ref().unwrap());
                    unsafe { menu.destroy() };
                }
            }

            if let Some((id, Some(menu))) = &self.gtk_menu {
                drop_children_from_menu_and_destroy(*id, menu, self.children.as_ref().unwrap());
                unsafe { menu.destroy() };
            }
        }

        for items in self.gtk_menu_items.borrow().values() {
            for item in items {
                if let Some(accel_group) = &self.accel_group {
                    if let Some((mods, key)) = self.gtk_accelerator {
                        item.remove_accelerator(accel_group, key, mods);
                    }
                }
                unsafe { item.destroy() };
            }
        }
    }
}

fn drop_children_from_menu_and_destroy(
    id: u32,
    menu: &impl IsA<Container>,
    children: &Vec<Rc<RefCell<MenuChild>>>,
) {
    for child in children {
        let mut child_ = child.borrow_mut();
        {
            let mut menu_items = child_.gtk_menu_items.borrow_mut();
            if let Some(items) = menu_items.remove(&id) {
                for item in items {
                    menu.remove(&item);
                    if let Some(accel_group) = &child_.accel_group {
                        if let Some((mods, key)) = child_.gtk_accelerator {
                            item.remove_accelerator(accel_group, key, mods);
                        }
                    }
                    unsafe { item.destroy() }
                }
            }
        }

        if child_.item_type == MenuItemType::Submenu {
            if let Some(menus) = child_.gtk_menus.as_mut().unwrap().remove(&id) {
                for (id, menu) in menus {
                    let children = child_.children.as_ref().unwrap();
                    drop_children_from_menu_and_destroy(id, &menu, children);
                    unsafe { menu.destroy() }
                }
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
            text: text.to_string(),
            enabled,
            accelerator: key_accelerator,
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            item_type: MenuItemType::MenuItem,
            gtk_menu_items: Rc::new(RefCell::new(HashMap::new())),
            accel_group: None,
            checked: None,
            children: None,
            gtk_accelerator: None,
            gtk_menu: None,
            gtk_menus: None,
            icon: None,
            is_syncing_checked_state: None,
            predefined_item_type: None,
        }
    }

    pub fn new_submenu(text: &str, enabled: bool, id: Option<MenuId>) -> Self {
        Self {
            text: text.to_string(),
            enabled,
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            children: Some(Vec::new()),
            item_type: MenuItemType::Submenu,
            gtk_menu: Some((COUNTER.next(), None)),
            gtk_menu_items: Rc::new(RefCell::new(HashMap::new())),
            gtk_menus: Some(HashMap::new()),
            accel_group: None,
            gtk_accelerator: None,
            icon: None,
            is_syncing_checked_state: None,
            predefined_item_type: None,
            accelerator: None,
            checked: None,
        }
    }

    pub(crate) fn new_predefined(item_type: PredefinedMenuItemType, text: Option<String>) -> Self {
        Self {
            text: text.unwrap_or_else(|| item_type.text().to_string()),
            enabled: true,
            accelerator: item_type.accelerator().map(Into::into),
            id: MenuId(COUNTER.next().to_string()),
            item_type: MenuItemType::Predefined,
            predefined_item_type: Some(item_type),
            gtk_menu_items: Rc::new(RefCell::new(HashMap::new())),
            accel_group: None,
            checked: None,
            children: None,
            gtk_accelerator: None,
            gtk_menu: None,
            gtk_menus: None,
            icon: None,
            is_syncing_checked_state: None,
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
            text: text.to_string(),
            enabled,
            checked: Some(Rc::new(AtomicBool::new(checked))),
            is_syncing_checked_state: Some(Rc::new(AtomicBool::new(false))),
            accelerator: key_accelerator,
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            item_type: MenuItemType::Check,
            gtk_menu_items: Rc::new(RefCell::new(HashMap::new())),
            accel_group: None,
            children: None,
            gtk_accelerator: None,
            gtk_menu: None,
            gtk_menus: None,
            icon: None,
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
            text: text.to_string(),
            enabled,
            icon,
            accelerator: key_accelerator,
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            item_type: MenuItemType::Icon,
            gtk_menu_items: Rc::new(RefCell::new(HashMap::new())),
            accel_group: None,
            checked: None,
            children: None,
            gtk_accelerator: None,
            gtk_menu: None,
            gtk_menus: None,
            is_syncing_checked_state: None,
            predefined_item_type: None,
        }
    }

    pub fn new_native_icon(
        text: &str,
        enabled: bool,
        _native_icon: Option<NativeIcon>,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        Self {
            text: text.to_string(),
            enabled,
            accelerator: key_accelerator,
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            item_type: MenuItemType::Icon,
            gtk_menu_items: Rc::new(RefCell::new(HashMap::new())),
            accel_group: None,
            checked: None,
            children: None,
            gtk_accelerator: None,
            gtk_menu: None,
            gtk_menus: None,
            icon: None,
            is_syncing_checked_state: None,
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
        match self
            .gtk_menu_items
            .borrow()
            .values()
            .collect::<Vec<_>>()
            .first()
            .map(|v| v.first())
            .map(|e| e.map(|i| i.label().map(from_gtk_mnemonic)))
        {
            Some(Some(Some(text))) => text,
            _ => self.text.clone(),
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        let text = to_gtk_mnemonic(text);
        for items in self.gtk_menu_items.borrow().values() {
            for i in items {
                i.set_label(&text);
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self
            .gtk_menu_items
            .borrow()
            .values()
            .collect::<Vec<_>>()
            .first()
            .map(|v| v.first())
            .map(|e| e.map(|i| i.is_sensitive()))
        {
            Some(Some(enabled)) => enabled,
            _ => self.enabled,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        for items in self.gtk_menu_items.borrow().values() {
            for i in items {
                i.set_sensitive(enabled);
            }
        }
    }

    pub fn set_key_accelerator(
        &mut self,
        accelerator: Option<KeyAccelerator>,
    ) -> crate::Result<()> {
        let prev_accel = self.gtk_accelerator.as_ref();
        let new_accel = accelerator
            .as_ref()
            .map(parse_key_accelerator)
            .transpose()?;

        for items in self.gtk_menu_items.borrow().values() {
            for i in items {
                if let Some((mods, key)) = prev_accel {
                    if let Some(accel_group) = &self.accel_group {
                        i.remove_accelerator(accel_group, *key, *mods);
                    }
                }
                if let Some((mods, key)) = new_accel {
                    if let Some(accel_group) = &self.accel_group {
                        i.add_accelerator(
                            "activate",
                            accel_group,
                            key,
                            mods,
                            gtk::AccelFlags::VISIBLE,
                        )
                    }
                }
            }
        }

        self.gtk_accelerator = new_accel;
        self.accelerator = accelerator;

        Ok(())
    }
}

/// CheckMenuItem methods
impl MenuChild {
    pub fn is_checked(&self) -> bool {
        match self
            .gtk_menu_items
            .borrow()
            .values()
            .collect::<Vec<_>>()
            .first()
            .map(|v| v.first())
            .map(|e| e.map(|i| i.downcast_ref::<gtk::CheckMenuItem>().unwrap().is_active()))
        {
            Some(Some(checked)) => checked,
            _ => self.checked.as_ref().unwrap().load(Ordering::Relaxed),
        }
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked
            .as_ref()
            .unwrap()
            .store(checked, Ordering::Release);
        let is_syncing = self.is_syncing_checked_state.as_ref().unwrap();
        is_syncing.store(true, Ordering::Release);
        for items in self.gtk_menu_items.borrow().values() {
            for i in items {
                i.downcast_ref::<gtk::CheckMenuItem>()
                    .unwrap()
                    .set_active(checked);
            }
        }
        is_syncing.store(false, Ordering::Release);
    }
}

/// IconMenuItem methods
impl MenuChild {
    pub fn set_icon(&mut self, icon: Option<Icon>) {
        self.icon.clone_from(&icon);

        let pixbuf = icon.as_ref().map(|i| i.inner.to_pixbuf_scale(16, 16));
        for items in self.gtk_menu_items.borrow().values() {
            for i in items {
                let box_container = i.child().unwrap().downcast::<gtk::Box>().unwrap();
                let children = box_container.children();

                // Check if the first child is an image (it might not be if the item
                // was created for menu bar without an icon)
                if let Some(image) = children
                    .first()
                    .and_then(|c| c.downcast_ref::<gtk::Image>())
                {
                    if icon.is_some() {
                        // Image exists and we're setting an icon, update it
                        image.set_pixbuf(pixbuf.as_ref());
                    } else {
                        // Image exists but we're removing the icon, remove the widget to avoid padding
                        box_container.remove(image);
                    }
                } else if icon.is_some() {
                    // No image widget exists yet, but we're setting an icon, so create one
                    let image = gtk::Image::from_pixbuf(pixbuf.as_ref());
                    box_container.pack_start(&image, false, false, 0);
                    box_container.reorder_child(&image, 0);
                    image.show();
                }
            }
        }
    }
}

/// Submenu methods
impl MenuChild {
    pub fn add_menu_item(&mut self, item: &dyn crate::IsMenuItem, op: AddOp) -> crate::Result<()> {
        if is_item_supported!(item) {
            for menus in self.gtk_menus.as_ref().unwrap().values() {
                for (menu_id, menu) in menus {
                    let gtk_item =
                        item.make_gtk_menu_item(*menu_id, self.accel_group.as_ref(), true, false)?;
                    match op {
                        AddOp::Append => menu.append(&gtk_item),
                        AddOp::Insert(position) => menu.insert(&gtk_item, position as i32),
                    }
                    gtk_item.show();
                }
            }

            if let Some((menu_id, Some(menu))) = self.gtk_menu.as_ref() {
                let gtk_item =
                    item.make_gtk_menu_item(*menu_id, self.accel_group.as_ref(), true, false)?;
                match op {
                    AddOp::Append => menu.append(&gtk_item),
                    AddOp::Insert(position) => menu.insert(&gtk_item, position as i32),
                }
                gtk_item.show();
            }
        }

        match op {
            AddOp::Append => self.children.as_mut().unwrap().push(item.child()),
            AddOp::Insert(position) => self
                .children
                .as_mut()
                .unwrap()
                .insert(position, item.child()),
        }

        Ok(())
    }

    fn add_menu_item_with_id(&self, item: &dyn crate::IsMenuItem, id: u32) -> crate::Result<()> {
        return_if_item_not_supported!(item);

        for menus in self.gtk_menus.as_ref().unwrap().values() {
            for (menu_id, menu) in menus.iter().filter(|m| m.0 == id) {
                let gtk_item =
                    item.make_gtk_menu_item(*menu_id, self.accel_group.as_ref(), true, false)?;
                menu.append(&gtk_item);
                gtk_item.show();
            }
        }

        Ok(())
    }

    fn add_menu_item_to_context_menu(&self, item: &dyn crate::IsMenuItem) -> crate::Result<()> {
        return_if_item_not_supported!(item);

        if let Some((menu_id, Some(menu))) = self.gtk_menu.as_ref() {
            let gtk_item =
                item.make_gtk_menu_item(*menu_id, self.accel_group.as_ref(), true, false)?;
            menu.append(&gtk_item);
            gtk_item.show();
        }

        Ok(())
    }

    pub fn remove(&mut self, item: &dyn crate::IsMenuItem) -> crate::Result<()> {
        self.remove_inner(item, true, None)
    }

    fn remove_inner(
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

        for menus in self.gtk_menus.as_ref().unwrap().values() {
            for (menu_id, menu) in menus {
                // check if we are removing this item from all gtk_menus
                //      which is usually when this is the item the user is actaully removing
                // or if we are removing from a specific menu (id)
                //      which is when the actual item being removed is a submenu
                //      and we are iterating through its children and removing
                //      each child gtk items that are related to this submenu.
                if id.map(|i| i == *menu_id).unwrap_or(true) {
                    // bail this is not a supported item like a close_window predefined menu item
                    if is_item_supported!(item) {
                        let mut child_ = child.borrow_mut();

                        if child_.item_type == MenuItemType::Submenu {
                            let menus = child_.gtk_menus.as_ref().unwrap().get(menu_id).cloned();
                            if let Some(menus) = menus {
                                for (id, menu) in menus {
                                    // iterate through children and only remove the gtk items
                                    // related to this submenu
                                    for item in child_.items() {
                                        child_.remove_inner(item.as_ref(), false, Some(id))?;
                                    }
                                    unsafe { menu.destroy() };
                                }
                            }
                            child_.gtk_menus.as_mut().unwrap().remove(menu_id);
                        }

                        // remove all the gtk items that are related to this gtk menu and destroy it
                        if let Some(items) = child_.gtk_menu_items.borrow_mut().remove(menu_id) {
                            for item in items {
                                menu.remove(&item);
                                if let Some(accel_group) = &child_.accel_group {
                                    if let Some((mods, key)) = child_.gtk_accelerator {
                                        item.remove_accelerator(accel_group, key, mods);
                                    }
                                }
                                unsafe { item.destroy() };
                            }
                        };
                    }
                }
            }
        }

        // remove from the gtk menu assigned to the context menu
        if remove_from_cache {
            if let (id, Some(menu)) = self.gtk_menu.as_ref().unwrap() {
                let child_ = child.borrow_mut();
                if let Some(items) = child_.gtk_menu_items.borrow_mut().remove(id) {
                    for item in items {
                        menu.remove(&item);
                        if let Some(accel_group) = &child_.accel_group {
                            if let Some((mods, key)) = child_.gtk_accelerator {
                                item.remove_accelerator(accel_group, key, mods);
                            }
                        }
                        unsafe { item.destroy() };
                    }
                };
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

    pub fn show_context_menu_for_gtk_window(
        &mut self,
        widget: &impl IsA<gtk::Widget>,
        position: Option<Position>,
    ) -> bool {
        show_context_menu(self.gtk_context_menu(), widget, position)
    }

    pub fn gtk_context_menu(&mut self) -> gtk::Menu {
        let mut add_items = false;
        {
            let gtk_menu = self.gtk_menu.as_mut().unwrap();
            if gtk_menu.1.is_none() {
                gtk_menu.1 = Some(gtk::Menu::new());
                add_items = true;
            }
        }

        if add_items {
            for item in self.items() {
                self.add_menu_item_to_context_menu(item.as_ref()).unwrap();
            }
        }

        self.gtk_menu.as_ref().unwrap().1.as_ref().unwrap().clone()
    }
}

macro_rules! register_accel {
    ($self:ident, $item:ident, $accel_group:ident) => {
        $self.gtk_accelerator = $self
            .accelerator
            .as_ref()
            .map(parse_key_accelerator)
            .transpose()?;

        if let Some((mods, key)) = &$self.gtk_accelerator {
            if let Some(accel_group) = $accel_group {
                $item.add_accelerator(
                    "activate",
                    accel_group,
                    *key,
                    *mods,
                    gtk::AccelFlags::VISIBLE,
                )
            }
        }
    };
}

/// Gtk menu item creation methods
impl MenuChild {
    fn create_gtk_item_for_submenu(
        &mut self,
        menu_id: u32,
        accel_group: Option<&gtk::AccelGroup>,
        add_to_cache: bool,
        for_menu_bar: bool,
    ) -> crate::Result<gtk::MenuItem> {
        let submenu = gtk::Menu::new();

        let image = self
            .icon
            .as_ref()
            .map(|icon| gtk::Image::from_pixbuf(Some(&icon.inner.to_pixbuf_scale(16, 16))))
            .unwrap_or_default();

        let label = gtk::AccelLabel::builder()
            .label(to_gtk_mnemonic(&self.text))
            .use_underline(true)
            .xalign(0.0)
            .build();

        let box_container = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        if !for_menu_bar {
            let style_context = box_container.style_context();
            let css_provider = gtk::CssProvider::new();
            let theme = r#"
            box {
                margin-left: -22px;
                }
                "#;
            let _ = css_provider.load_from_data(theme.as_bytes());
            style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }
        if !for_menu_bar || self.icon.is_some() {
            box_container.pack_start(&image, false, false, 0);
        }
        box_container.pack_start(&label, true, true, 0);
        box_container.show_all();

        let item = gtk::MenuItem::builder()
            .child(&box_container)
            .sensitive(self.enabled)
            .build();

        item.set_submenu(Some(&submenu));
        item.show();

        self.accel_group = accel_group.cloned();

        let mut id = 0;
        if add_to_cache {
            id = COUNTER.next();

            self.gtk_menu_items
                .borrow_mut()
                .entry(menu_id)
                .or_default()
                .push(item.clone());
            self.gtk_menus
                .as_mut()
                .unwrap()
                .entry(menu_id)
                .or_default()
                .push((id, submenu.clone()));
        }

        for child_item in self.items() {
            if add_to_cache {
                self.add_menu_item_with_id(child_item.as_ref(), id)?;
            } else {
                let gtk_child_item = child_item.make_gtk_menu_item(0, None, false, false)?;
                submenu.append(&gtk_child_item);
            }
        }

        Ok(item)
    }

    fn create_gtk_item_for_menu_item(
        &mut self,
        menu_id: u32,
        accel_group: Option<&gtk::AccelGroup>,
        add_to_cache: bool,
    ) -> crate::Result<gtk::MenuItem> {
        let item = gtk::MenuItem::builder()
            .label(to_gtk_mnemonic(&self.text))
            .use_underline(true)
            .sensitive(self.enabled)
            .build();

        self.accel_group = accel_group.cloned();

        register_accel!(self, item, accel_group);

        let id = self.id.clone();
        item.connect_activate(move |_| {
            MenuEvent::send(crate::MenuEvent { id: id.clone() });
        });

        if add_to_cache {
            self.gtk_menu_items
                .borrow_mut()
                .entry(menu_id)
                .or_default()
                .push(item.clone());
        }

        Ok(item)
    }

    fn create_gtk_item_for_predefined_menu_item(
        &mut self,
        menu_id: u32,
        accel_group: Option<&gtk::AccelGroup>,
        add_to_cache: bool,
    ) -> crate::Result<gtk::MenuItem> {
        let text = self.text.clone();
        self.gtk_accelerator = self
            .accelerator
            .as_ref()
            .map(parse_key_accelerator)
            .transpose()?;
        let predefined_item_type = self.predefined_item_type.clone().unwrap();

        let make_item = || {
            gtk::MenuItem::builder()
                .label(to_gtk_mnemonic(&text))
                .use_underline(true)
                .sensitive(true)
                .build()
        };
        let register_accel = |item: &gtk::MenuItem| {
            if let Some((mods, key)) = &self.gtk_accelerator {
                if let Some(accel_group) = accel_group {
                    item.add_accelerator(
                        "activate",
                        accel_group,
                        *key,
                        *mods,
                        gtk::AccelFlags::VISIBLE,
                    )
                }
            }
        };

        let item = match predefined_item_type {
            PredefinedMenuItemType::Separator => {
                gtk::SeparatorMenuItem::new().upcast::<gtk::MenuItem>()
            }
            PredefinedMenuItemType::Copy
            | PredefinedMenuItemType::Cut
            | PredefinedMenuItemType::Paste
            | PredefinedMenuItemType::SelectAll => {
                let item = make_item();
                let (mods, key) = {
                    let key_accel: crate::accelerator::KeyAccelerator =
                        predefined_item_type.accelerator().unwrap().into();
                    parse_key_accelerator(&key_accel).unwrap()
                };
                item.child()
                    .unwrap()
                    .downcast::<gtk::AccelLabel>()
                    .unwrap()
                    .set_accel(key, mods);
                item.connect_activate(move |_| {
                    // TODO: wayland
                    #[cfg(feature = "libxdo")]
                    if let Ok(xdo) = libxdo::XDo::new(None) {
                        let _ = xdo.send_keysequence(predefined_item_type.xdo_keys(), 0);
                    }
                });
                item
            }
            PredefinedMenuItemType::About(metadata) => {
                let item = make_item();
                register_accel(&item);
                item.connect_activate(move |_| {
                    if let Some(metadata) = &metadata {
                        let mut builder = AboutDialog::builder().modal(true).resizable(false);

                        if let Some(name) = &metadata.name {
                            builder = builder.program_name(name);
                        }
                        if let Some(version) = &metadata.full_version() {
                            builder = builder.version(version);
                        }
                        if let Some(authors) = &metadata.authors {
                            builder = builder.authors(authors.clone());
                        }
                        if let Some(comments) = &metadata.comments {
                            builder = builder.comments(comments);
                        }
                        if let Some(copyright) = &metadata.copyright {
                            builder = builder.copyright(copyright);
                        }
                        if let Some(license) = &metadata.license {
                            builder = builder.license(license);
                        }
                        if let Some(website) = &metadata.website {
                            builder = builder.website(website);
                        }
                        if let Some(website_label) = &metadata.website_label {
                            builder = builder.website_label(website_label);
                        }
                        if let Some(icon) = &metadata.icon {
                            builder = builder.logo(&icon.inner.to_pixbuf());
                        }

                        let about = builder.build();
                        about.run();
                        unsafe {
                            about.destroy();
                        }
                    }
                });
                item
            }
            _ => unreachable!(),
        };

        if add_to_cache {
            self.gtk_menu_items
                .borrow_mut()
                .entry(menu_id)
                .or_default()
                .push(item.clone());
        }
        Ok(item)
    }

    fn create_gtk_item_for_check_menu_item(
        &mut self,
        menu_id: u32,
        accel_group: Option<&gtk::AccelGroup>,
        add_to_cache: bool,
    ) -> crate::Result<gtk::MenuItem> {
        let item = gtk::CheckMenuItem::builder()
            .label(to_gtk_mnemonic(&self.text))
            .use_underline(true)
            .sensitive(self.enabled)
            .active(self.checked.as_ref().unwrap().load(Ordering::Relaxed))
            .build();

        self.accel_group = accel_group.cloned();

        register_accel!(self, item, accel_group);

        let id = self.id.clone();
        let is_syncing_checked_state = self.is_syncing_checked_state.clone().unwrap();
        let checked = self.checked.clone().unwrap();
        let store = self.gtk_menu_items.clone();
        item.connect_toggled(move |i| {
            let should_dispatch = is_syncing_checked_state
                .compare_exchange(false, true, Ordering::Release, Ordering::Relaxed)
                .is_ok();

            if should_dispatch {
                let c = i.is_active();
                checked.store(c, Ordering::Release);

                for items in store.borrow().values() {
                    for i in items {
                        i.downcast_ref::<gtk::CheckMenuItem>()
                            .unwrap()
                            .set_active(c);
                    }
                }

                is_syncing_checked_state.store(false, Ordering::Release);

                MenuEvent::send(crate::MenuEvent { id: id.clone() });
            }
        });

        let item = item.upcast::<gtk::MenuItem>();

        if add_to_cache {
            self.gtk_menu_items
                .borrow_mut()
                .entry(menu_id)
                .or_default()
                .push(item.clone());
        }

        Ok(item)
    }

    fn create_gtk_item_for_icon_menu_item(
        &mut self,
        menu_id: u32,
        accel_group: Option<&gtk::AccelGroup>,
        add_to_cache: bool,
        for_menu_bar: bool,
    ) -> crate::Result<gtk::MenuItem> {
        let image = self
            .icon
            .as_ref()
            .map(|i| gtk::Image::from_pixbuf(Some(&i.inner.to_pixbuf_scale(16, 16))))
            .unwrap_or_default();

        self.accel_group = accel_group.cloned();

        let label = gtk::AccelLabel::builder()
            .label(to_gtk_mnemonic(&self.text))
            .use_underline(true)
            .xalign(0.0)
            .build();

        let box_container = gtk::Box::new(Orientation::Horizontal, 6);
        if !for_menu_bar {
            let style_context = box_container.style_context();
            let css_provider = gtk::CssProvider::new();
            let theme = r#"
            box {
                margin-left: -22px;
                }
                "#;
            let _ = css_provider.load_from_data(theme.as_bytes());
            style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }
        if !for_menu_bar || self.icon.is_some() {
            box_container.pack_start(&image, false, false, 0);
        }
        box_container.pack_start(&label, true, true, 0);
        box_container.show_all();

        let item = gtk::MenuItem::builder()
            .child(&box_container)
            .sensitive(self.enabled)
            .build();

        register_accel!(self, item, accel_group);

        let id = self.id.clone();
        item.connect_activate(move |_| {
            MenuEvent::send(crate::MenuEvent { id: id.clone() });
        });

        if add_to_cache {
            self.gtk_menu_items
                .borrow_mut()
                .entry(menu_id)
                .or_default()
                .push(item.clone());
        }

        Ok(item)
    }
}

impl MenuItemKind {
    fn make_gtk_menu_item(
        &self,
        menu_id: u32,
        accel_group: Option<&gtk::AccelGroup>,
        add_to_cache: bool,
        for_menu_bar: bool,
    ) -> crate::Result<gtk::MenuItem> {
        let mut child = self.child_mut();
        match child.item_type() {
            MenuItemType::Submenu => {
                child.create_gtk_item_for_submenu(menu_id, accel_group, add_to_cache, for_menu_bar)
            }
            MenuItemType::MenuItem => {
                child.create_gtk_item_for_menu_item(menu_id, accel_group, add_to_cache)
            }
            MenuItemType::Predefined => {
                child.create_gtk_item_for_predefined_menu_item(menu_id, accel_group, add_to_cache)
            }
            MenuItemType::Check => {
                child.create_gtk_item_for_check_menu_item(menu_id, accel_group, add_to_cache)
            }
            MenuItemType::Icon => child.create_gtk_item_for_icon_menu_item(
                menu_id,
                accel_group,
                add_to_cache,
                for_menu_bar,
            ),
        }
    }
}

impl dyn IsMenuItem + '_ {
    fn make_gtk_menu_item(
        &self,
        menu_id: u32,
        accel_group: Option<&gtk::AccelGroup>,
        add_to_cache: bool,
        for_menu_bar: bool,
    ) -> crate::Result<gtk::MenuItem> {
        self.kind()
            .make_gtk_menu_item(menu_id, accel_group, add_to_cache, for_menu_bar)
    }
}

fn show_context_menu(
    gtk_menu: gtk::Menu,
    widget: &impl IsA<gtk::Widget>,
    position: Option<Position>,
) -> bool {
    let (pos, window) = if let Some(pos) = position {
        let window = widget.window();
        (
            pos.to_logical::<i32>(window.as_ref().map(|w| w.scale_factor()).unwrap_or(1) as _)
                .into(),
            window,
        )
    } else {
        let window = widget.screen().and_then(|s| s.root_window());
        (
            window
                .as_ref()
                .and_then(|w| {
                    w.display()
                        .default_seat()
                        .and_then(|s| s.pointer())
                        .map(|s| {
                            let p = s.position();
                            (p.1, p.2)
                        })
                })
                .unwrap_or_default(),
            window,
        )
    };

    if let Some(window) = window {
        let mut event = gdk::Event::new(gdk::EventType::ButtonPress);
        event.set_device(
            window
                .display()
                .default_seat()
                .and_then(|d| d.pointer())
                .as_ref(),
        );

        // Set the time of the event otherwise GTK will close the menu
        // when right click is released
        let event_ffi: *mut gdk::ffi::GdkEvent = event.to_glib_none().0;
        if !event_ffi.is_null() {
            let time = glib::monotonic_time() / 1000;
            unsafe {
                (*event_ffi).button.time = time as _;
            }
        }

        let (tx, rx) = crossbeam_channel::unbounded();
        let tx_clone = tx.clone();
        let id = gtk_menu.connect_cancel(move |_| tx_clone.send(false).unwrap_or(()));
        let id2 = gtk_menu.connect_selection_done(move |_| tx.send(true).unwrap_or(()));
        gtk_menu.popup_at_rect(
            &window,
            &gdk::Rectangle::new(pos.0, pos.1, 0, 0),
            gdk::Gravity::NorthWest,
            gdk::Gravity::NorthWest,
            Some(&event),
        );

        loop {
            gtk::main_iteration();

            match rx.try_recv() {
                Ok(result) => {
                    gtk_menu.disconnect(id);
                    gtk_menu.disconnect(id2);
                    return result;
                }
                Err(err) => {
                    if err.is_disconnected() {
                        gtk_menu.disconnect(id);
                        gtk_menu.disconnect(id2);
                        return false;
                    }
                }
            }
        }
    }

    false
}

impl PredefinedMenuItemType {
    #[cfg(feature = "libxdo")]
    fn xdo_keys(&self) -> &str {
        match self {
            PredefinedMenuItemType::Copy => "ctrl+c",
            PredefinedMenuItemType::Cut => "ctrl+X",
            PredefinedMenuItemType::Paste => "ctrl+v",
            PredefinedMenuItemType::SelectAll => "ctrl+a",
            _ => unreachable!(),
        }
    }
}
