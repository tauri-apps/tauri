// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod accelerator;
mod icon;
mod mnemonic;

#[cfg(all(feature = "linux-ksni", target_os = "linux"))]
use std::sync::Arc;
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

#[cfg(all(feature = "linux-ksni", target_os = "linux"))]
use arc_swap::ArcSwap;
use dpi::Position;
use gtk::{gdk::Rectangle, gio, prelude::*};
pub(crate) use icon::PlatformIcon;
use mnemonic::to_gtk_mnemonic;

use crate::{
    accelerator::KeyAccelerator,
    util::{AddOp, Counter},
    Icon, IsMenuItem, MenuEvent, MenuId, MenuItemKind, MenuItemType, NativeIcon,
    PredefinedMenuItemType,
};

static COUNTER: Counter = Counter::new();

const DEFAULT_ACTION_GROUP: &str = "muda";
const ACTION_GROUP_DATA_KEY: &str = "mudaActionGroup";

#[cfg(all(feature = "linux-ksni", target_os = "linux"))]
fn compat_placeholder() -> Arc<ArcSwap<crate::CompatMenuItem>> {
    Arc::new(ArcSwap::from_pointee(crate::CompatMenuItem::Separator))
}

enum GtkMenuBar {
    MenuBar {
        widget: gtk::PopoverMenuBar,
        menu: gio::Menu,
        app: gtk::Application,
    },
    ContextMenu {
        widget: gtk::PopoverMenu,
        menu: gio::Menu,
        app: gtk::Application,
    },
}

impl GtkMenuBar {
    fn new(app: gtk::Application) -> Self {
        let menu = gio::Menu::new();
        let widget = gtk::PopoverMenuBar::from_model(Some(&menu));
        Self::MenuBar { widget, menu, app }
    }

    fn new_context(app: gtk::Application) -> Self {
        let menu = gio::Menu::new();
        let widget = gtk::PopoverMenu::from_model(Some(&menu));
        Self::ContextMenu { widget, menu, app }
    }

    fn application(&self) -> &gtk::Application {
        match self {
            GtkMenuBar::MenuBar { app, .. } => app,
            GtkMenuBar::ContextMenu { app, .. } => app,
        }
    }

    fn menu_bar(&self) -> &gtk::PopoverMenuBar {
        match self {
            GtkMenuBar::MenuBar { widget, .. } => widget,
            _ => unreachable!("This is a bug report to https://github.com/tauri-apps/muda"),
        }
    }

    fn context_menu(&self) -> &gtk::PopoverMenu {
        match self {
            GtkMenuBar::ContextMenu { widget, .. } => widget,
            _ => unreachable!("This is a bug report to https://github.com/tauri-apps/muda"),
        }
    }

    fn menu(&self) -> &gio::Menu {
        match self {
            GtkMenuBar::MenuBar { menu, .. } => menu,
            GtkMenuBar::ContextMenu { menu, .. } => menu,
        }
    }

    /// Bind a widget to a `custom` slot in the generated menu.
    ///
    /// GTK4's `PopoverMenu` does not render icons set on a `gio::MenuItem`.
    /// To show one we mark the model item with a `custom` attribute and
    /// attach the real icon+label widget here; the binding reaches slots in
    /// nested submenus too, so a single call per item is enough.
    fn add_custom_child(&self, id: &str, widget: &impl IsA<gtk::Widget>) {
        match self {
            GtkMenuBar::MenuBar { widget: w, .. } => {
                w.add_child(widget, id);
            }
            GtkMenuBar::ContextMenu { widget: w, .. } => {
                w.add_child(widget, id);
            }
        }
    }
}

/// Build the icon + label row shown in place of a default menu item.
///
/// A flat `Button` carrying the model item's action, so activating it fires
/// the same `MenuEvent` a normal row would and `PopoverMenu` closes on click.
fn custom_menu_row(icon: &crate::icon::Icon, label: &str, action: &str) -> gtk::Button {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let image = gtk::Image::from_gicon(&icon.inner.to_bytes_icon());
    image.set_pixel_size(16);
    let label = gtk::Label::new(Some(&strip_mnemonic(label)));
    label.set_xalign(0.0);
    label.set_hexpand(true);
    row.append(&image);
    row.append(&label);

    let button = gtk::Button::builder()
        .child(&row)
        .has_frame(false)
        .action_name(action)
        .build();
    button.add_css_class("model");
    button
}

fn strip_mnemonic(text: &str) -> String {
    text.replace("__", "\u{0}")
        .replace('_', "")
        .replace('\u{0}', "_")
}

pub struct Menu {
    id: MenuId,
    instances: HashMap<u32, GtkMenuBar>,
    ctx_menu_id: u32,
    children: Vec<Rc<RefCell<MenuChild>>>,
}

impl Menu {
    pub fn new(id: Option<MenuId>) -> Self {
        Self {
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            instances: HashMap::new(),
            ctx_menu_id: COUNTER.next(),
            children: Vec::new(),
        }
    }

    pub fn id(&self) -> &MenuId {
        &self.id
    }

    pub fn add_menu_item(&mut self, item: &dyn IsMenuItem, op: AddOp) -> crate::Result<()> {
        match op {
            AddOp::Append => self.children.push(item.child()),
            AddOp::Insert(i) => self.children.insert(i, item.child()),
        }

        for (menu_id, menu_bar) in &self.instances {
            let parent_menu = menu_bar.menu();
            let gtk_item =
                item.make_gtk_menu_item(menu_bar.application(), *menu_id, parent_menu)?;
            match op {
                AddOp::Append => parent_menu.append_item(&gtk_item),
                AddOp::Insert(position) => parent_menu.insert_item(position as i32, &gtk_item),
            }
        }

        #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
        crate::send_menu_update();

        Ok(())
    }

    pub fn add_menu_item_with_id(&mut self, item: &dyn IsMenuItem, id: u32) -> crate::Result<()> {
        for (menu_id, menu_bar) in self.instances.iter().filter(|m| *m.0 == id) {
            let parent_menu = menu_bar.menu();
            let gtk_item =
                item.make_gtk_menu_item(menu_bar.application(), *menu_id, parent_menu)?;
            parent_menu.append_item(&gtk_item);
        }

        Ok(())
    }

    /// Attach the icon+label widgets for every `custom` slot in the tree to
    /// the given instance. Run after the instance's model is fully built.
    fn bind_custom_children(&self, menu_bar: &GtkMenuBar) {
        fn walk(children: &[Rc<RefCell<MenuChild>>], menu_bar: &GtkMenuBar) {
            for child in children {
                let child = child.borrow();
                if let Some((id, row)) = child.custom_child() {
                    menu_bar.add_custom_child(&id, &row);
                }
                walk(&child.children, menu_bar);
            }
        }
        walk(&self.children, menu_bar);
    }

    pub fn remove(&mut self, item: &dyn IsMenuItem) -> crate::Result<()> {
        let child = item.child();
        let child_id = child.borrow().id().clone();

        // Find position of item in children
        let position = self
            .children
            .iter()
            .position(|c| c.borrow().id() == &child_id);

        let Some(position) = position else {
            return Err(crate::Error::NotAChildOfThisMenu);
        };

        // Remove from all GIO menus at the same position
        for (menu_id, menu_bar) in &self.instances {
            menu_bar.menu().remove(position as i32);
            // Clean up the item's instances for this menu
            item.child().borrow_mut().instances.remove(menu_id);
        }

        // Remove from children
        self.children.remove(position);

        #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
        crate::send_menu_update();

        Ok(())
    }

    pub fn items(&self) -> Vec<MenuItemKind> {
        self.children
            .iter()
            .map(|c| c.borrow().kind(c.clone()))
            .collect()
    }

    #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
    pub fn compat_items(&self) -> Vec<Arc<ArcSwap<crate::CompatMenuItem>>> {
        self.children
            .iter()
            .map(|child| child.borrow().compat_child())
            .collect()
    }

    pub fn init_for_gtk_window<W, C>(
        &mut self,
        window: &W,
        container: Option<&C>,
    ) -> crate::Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window>,
        W: gtk::prelude::IsA<gtk::Widget>,
        C: gtk::prelude::IsA<gtk::Widget>,
    {
        let id = window.as_ptr() as u32;

        let Some(app) = window.application() else {
            return Err(crate::Error::GtkWindowWithoutApplication);
        };

        // This is the first time this method has been called on this window
        // so we need to create the menubar
        if let Entry::Vacant(e) = self.instances.entry(id) {
            e.insert(GtkMenuBar::new(app.clone()));
        } else {
            return Err(crate::Error::AlreadyInitialized);
        }

        let action_group = action_group_from_app(&app);
        window.insert_action_group(DEFAULT_ACTION_GROUP, Some(&action_group));

        for item in self.items() {
            self.add_menu_item_with_id(item.as_ref(), id)?;
        }

        self.bind_custom_children(&self.instances[&id]);

        let menu_bar = self.instances[&id].menu_bar();

        // add the menubar to the specified widget, otherwise to the window
        if let Some(container) = container {
            if container.type_().name() == "GtkBox" {
                let gtk_box = container.dynamic_cast_ref::<gtk::Box>().unwrap();
                gtk_box.prepend(menu_bar);
            } else if container.type_().name() == "GtkFixed" {
                let gtk_box = container.dynamic_cast_ref::<gtk::Fixed>().unwrap();
                gtk_box.put(menu_bar, 0., 0.);
            } else if container.type_().name() == "GtkStack" {
                let gtk_box = container.dynamic_cast_ref::<gtk::Stack>().unwrap();
                gtk_box.add_child(menu_bar);
            }
        } else {
            window.set_child(Some(menu_bar));
        }

        // show the menu bar
        menu_bar.set_visible(true);

        Ok(())
    }

    pub fn remove_for_gtk_window<W>(&mut self, window: &W) -> crate::Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window>,
        W: gtk::prelude::IsA<gtk::Widget>,
    {
        let id = window.as_ptr() as u32;

        let Some(menu_bar) = self.instances.remove(&id) else {
            return Err(crate::Error::NotInitialized);
        };

        window.insert_action_group(DEFAULT_ACTION_GROUP, None::<&gio::SimpleActionGroup>);

        // Unparent the menu bar widget to remove it from the window
        menu_bar.menu_bar().unparent();

        // Clean up children instances for this menu
        for child in &self.children {
            child.borrow_mut().instances.remove(&id);
        }

        Ok(())
    }

    pub fn hide_for_gtk_window<W>(&self, window: &W) -> crate::Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        let id = window.as_ptr() as u32;
        let Some(menu_bar) = self.instances.get(&id) else {
            return Err(crate::Error::NotInitialized);
        };
        menu_bar.menu_bar().set_visible(false);
        Ok(())
    }

    pub fn show_for_gtk_window<W>(&self, window: &W) -> crate::Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        let id = window.as_ptr() as u32;
        let Some(menu_bar) = self.instances.get(&id) else {
            return Err(crate::Error::NotInitialized);
        };
        menu_bar.menu_bar().set_visible(true);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn is_visible_on_gtk_window<W>(&self, window: &W) -> bool
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        let id = window.as_ptr() as u32;
        self.instances
            .get(&id)
            .map(|m| m.menu_bar().is_visible())
            .unwrap_or(false)
    }

    pub fn gtk_menubar_for_gtk_window<W>(&self, window: &W) -> Option<gtk::PopoverMenuBar>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        let id = window.as_ptr() as u32;
        self.instances.get(&id).map(|m| m.menu_bar().clone())
    }

    pub fn gtk_context_menu(&mut self) -> gtk::PopoverMenu {
        if !self.instances.contains_key(&self.ctx_menu_id) {
            let app = gio::Application::default()
                .and_downcast::<gtk::Application>()
                .expect("`gtk_context_menu` requires a running `gtk::Application`");
            let menu = GtkMenuBar::new_context(app);
            self.instances.insert(self.ctx_menu_id, menu);
            for item in self.items() {
                let _ = self.add_menu_item_with_id(item.as_ref(), self.ctx_menu_id);
            }
            self.bind_custom_children(&self.instances[&self.ctx_menu_id]);
        }
        self.instances
            .get(&self.ctx_menu_id)
            .unwrap()
            .context_menu()
            .clone()
    }

    pub fn show_context_menu_for_gtk_window(
        &mut self,
        window: &gtk::Window,
        position: Option<Position>,
    ) -> bool {
        let Some(app) = window.application() else {
            return false; // TODO: better error
        };

        // Rebuild the context instance every time so it reflects the current
        // labels, icons, and checked state rather than a stale first build.
        self.instances.remove(&self.ctx_menu_id);

        {
            let action_group = action_group_from_app(&app);
            window.insert_action_group(DEFAULT_ACTION_GROUP, Some(&action_group));

            let menu = GtkMenuBar::new_context(app);

            self.instances.insert(self.ctx_menu_id, menu);

            for item in self.items() {
                let _ = self.add_menu_item_with_id(item.as_ref(), self.ctx_menu_id);
            }
            self.bind_custom_children(&self.instances[&self.ctx_menu_id]);
        }

        let (x, y) = match position {
            Some(p) => p.to_logical::<i32>(window.scale_factor() as _).into(),
            None => get_cursor_pos(window),
        };

        // SAFETY: it is guaranteed to exist due to the check above
        let menu = self.instances.get(&self.ctx_menu_id).unwrap();
        let context_menu = menu.context_menu();

        if context_menu.parent().is_some() {
            context_menu.unparent();
        }
        context_menu.set_parent(window);

        context_menu.popup();
        context_menu.set_pointing_to(Some(&Rectangle::new(x, y, 0, 0)));

        true
    }
}

#[derive(Clone)]
enum GtkMenuChild {
    Item {
        item: gio::MenuItem,
        app: gtk::Application,
        parent_menu: gio::Menu,
    },
    Submenu {
        id: u32,
        item: gio::MenuItem,
        menu: gio::Menu,
        app: gtk::Application,
        parent_menu: gio::Menu,
    },
    ContextMenu {
        id: u32,
        widget: gtk::PopoverMenu,
        menu: gio::Menu,
        app: gtk::Application,
    },
}

impl GtkMenuChild {
    fn id(&self) -> u32 {
        match self {
            GtkMenuChild::Submenu { id, .. } => *id,
            GtkMenuChild::ContextMenu { id, .. } => *id,
            _ => unreachable!("This is a bug report to https://github.com/tauri-apps/muda"),
        }
    }

    fn application(&self) -> &gtk::Application {
        match self {
            GtkMenuChild::Submenu { app, .. } => app,
            GtkMenuChild::ContextMenu { app, .. } => app,
            GtkMenuChild::Item { app, .. } => app,
        }
    }

    fn parent_menu(&self) -> &gio::Menu {
        match self {
            GtkMenuChild::Item { parent_menu, .. } => parent_menu,
            GtkMenuChild::Submenu { parent_menu, .. } => parent_menu,
            _ => unreachable!("This is a bug report to https://github.com/tauri-apps/muda"),
        }
    }

    fn menu(&self) -> &gio::Menu {
        match self {
            GtkMenuChild::Submenu { menu, .. } => menu,
            GtkMenuChild::ContextMenu { menu, .. } => menu,
            _ => unreachable!("This is a bug report to https://github.com/tauri-apps/muda"),
        }
    }

    fn context_menu(&self) -> &gtk::PopoverMenu {
        match self {
            GtkMenuChild::ContextMenu { widget, .. } => widget,
            _ => unreachable!("This is a bug report to https://github.com/tauri-apps/muda"),
        }
    }
}

pub struct MenuChild {
    id: MenuId,
    text: String,
    enabled: bool,
    key_accelerator: Option<KeyAccelerator>,

    checked: bool,

    icon: Option<Icon>,

    predefined_item_type: Option<PredefinedMenuItemType>,

    type_: MenuItemType,

    instances: HashMap<u32, Vec<GtkMenuChild>>,
    ctx_menu_id: u32,
    children: Vec<Rc<RefCell<MenuChild>>>,

    action: Option<gio::SimpleAction>,

    #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
    compat: Arc<ArcSwap<crate::CompatMenuItem>>,
}

#[cfg(all(feature = "linux-ksni", target_os = "linux"))]
impl MenuChild {
    pub(crate) fn compat_child(&self) -> Arc<ArcSwap<crate::CompatMenuItem>> {
        self.sync_compat();
        self.compat.clone()
    }

    pub(crate) fn compat_items(&self) -> Vec<Arc<ArcSwap<crate::CompatMenuItem>>> {
        self.children
            .iter()
            .map(|child| child.borrow().compat_child())
            .collect()
    }

    fn sync_compat(&self) {
        self.compat.store(Arc::new(self.to_compat_item()));
    }

    fn to_compat_item(&self) -> crate::CompatMenuItem {
        let label = crate::strip_mnemonic(&self.text);
        match self.type_ {
            MenuItemType::Check => crate::CompatCheckmarkItem {
                id: self.id.0.clone(),
                label,
                enabled: self.enabled,
                checked: self.is_checked(),
            }
            .into(),
            MenuItemType::Submenu => crate::CompatSubMenuItem {
                label,
                enabled: self.enabled,
                submenu: self.compat_items(),
            }
            .into(),
            MenuItemType::Predefined => match self.predefined_item_type.as_ref() {
                Some(PredefinedMenuItemType::Separator) => crate::CompatMenuItem::Separator,
                Some(PredefinedMenuItemType::About(metadata)) => crate::CompatStandardItem {
                    id: self.id.0.clone(),
                    label,
                    enabled: self.enabled,
                    icon: None,
                    about_metadata: metadata.clone(),
                }
                .into(),
                _ => crate::CompatStandardItem {
                    id: self.id.0.clone(),
                    label,
                    enabled: self.enabled,
                    icon: None,
                    about_metadata: None,
                }
                .into(),
            },
            MenuItemType::Icon | MenuItemType::MenuItem => crate::CompatStandardItem {
                id: self.id.0.clone(),
                label,
                enabled: self.enabled,
                icon: self.icon.as_ref().map(|icon| icon.inner.png_data()),
                about_metadata: None,
            }
            .into(),
        }
    }

    fn notify_compat_changed(&self) {
        self.sync_compat();
        crate::send_menu_update();
    }
}

impl MenuChild {
    pub fn new_submenu(text: &str, enabled: bool, id: Option<MenuId>) -> Self {
        Self {
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            text: text.to_string(),
            enabled,
            checked: false,
            icon: None,
            key_accelerator: None,
            predefined_item_type: None,
            type_: MenuItemType::Submenu,
            ctx_menu_id: COUNTER.next(),
            instances: HashMap::new(),
            children: Vec::new(),
            action: None,
            #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
            compat: compat_placeholder(),
        }
    }

    fn create_gtk_item_for_submenu(
        &mut self,
        app: &gtk::Application,
        menu_id: u32,
        parent_menu: &gio::Menu,
    ) -> crate::Result<gio::MenuItem> {
        let menu = gio::Menu::new();
        let item = gio::MenuItem::new_submenu(Some(&to_gtk_mnemonic(&self.text)), &menu);
        item.set_detailed_action(&self.detailed_action());

        if self.action.is_none() {
            let action_group = action_group_from_app(app);

            let action = gio::SimpleAction::new(self.id.as_ref(), None);
            action.connect_activate(|_, _| ());
            action.set_enabled(self.enabled);
            action_group.add_action(&action);

            self.action = Some(action);
        }

        let id = COUNTER.next();
        let child = GtkMenuChild::Submenu {
            item: item.clone(),
            menu,
            id,
            app: app.clone(),
            parent_menu: parent_menu.clone(),
        };

        self.instances.entry(menu_id).or_default().push(child);

        for item in self.items() {
            self.add_menu_item_with_id(item.as_ref(), id)?;
        }

        Ok(item)
    }

    pub fn add_menu_item(&mut self, item: &dyn IsMenuItem, op: AddOp) -> crate::Result<()> {
        match op {
            AddOp::Append => self.children.push(item.child()),
            AddOp::Insert(i) => self.children.insert(i, item.child()),
        }

        #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
        self.notify_compat_changed();

        for menus in self.instances.values() {
            for gtk_child in menus {
                let parent_menu = gtk_child.menu();
                let gtk_item =
                    item.make_gtk_menu_item(gtk_child.application(), gtk_child.id(), parent_menu)?;

                match op {
                    AddOp::Append => parent_menu.append_item(&gtk_item),
                    AddOp::Insert(position) => parent_menu.insert_item(position as i32, &gtk_item),
                }
            }
        }

        Ok(())
    }

    pub fn add_menu_item_with_id(&self, item: &dyn IsMenuItem, id: u32) -> crate::Result<()> {
        for menus in self.instances.values() {
            for gtk_child in menus.iter().filter(|m| m.id() == id) {
                let parent_menu = gtk_child.menu();
                let gtk_item =
                    item.make_gtk_menu_item(gtk_child.application(), gtk_child.id(), parent_menu)?;
                parent_menu.append_item(&gtk_item);
            }
        }

        Ok(())
    }

    pub fn remove(&mut self, item: &dyn IsMenuItem) -> crate::Result<()> {
        let child = item.child();
        let child_id = child.borrow().id().clone();

        // Find position of item in children
        let position = self
            .children
            .iter()
            .position(|c| c.borrow().id() == &child_id);

        let Some(position) = position else {
            return Err(crate::Error::NotAChildOfThisMenu);
        };

        // Remove from all submenu GIO menus at the same position
        for menus in self.instances.values() {
            for gtk_child in menus {
                // Get the submenu's gio::Menu and remove at position
                gtk_child.menu().remove(position as i32);
            }
        }

        // Clean up the item's instances
        // For submenus, we need to clear instances that belong to this submenu's children
        let child_ref = item.child();
        let mut item_child = child_ref.borrow_mut();
        for menus in self.instances.values() {
            for gtk_child in menus {
                item_child.instances.remove(&gtk_child.id());
            }
        }

        // Remove from children
        self.children.remove(position);

        #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
        self.notify_compat_changed();

        Ok(())
    }

    pub fn items(&self) -> Vec<MenuItemKind> {
        self.children
            .iter()
            .map(|c| c.borrow().kind(c.clone()))
            .collect()
    }

    pub fn gtk_context_menu(&mut self) -> gtk::PopoverMenu {
        if !self.instances.contains_key(&self.ctx_menu_id) {
            let app = gio::Application::default()
                .and_downcast::<gtk::Application>()
                .expect("`gtk_context_menu` requires a running `gtk::Application`");
            let menu = gio::Menu::new();
            let widget = gtk::PopoverMenu::from_model(Some(&menu));
            let menu = GtkMenuChild::ContextMenu {
                id: self.ctx_menu_id,
                widget,
                menu,
                app,
            };
            self.instances.insert(self.ctx_menu_id, vec![menu]);
            for item in self.items() {
                let _ = self.add_menu_item_with_id(item.as_ref(), self.ctx_menu_id);
            }
        }
        self.instances
            .get(&self.ctx_menu_id)
            .unwrap()
            .first()
            .unwrap()
            .context_menu()
            .clone()
    }

    pub fn show_context_menu_for_gtk_window(
        &mut self,
        window: &gtk::Window,
        position: Option<Position>,
    ) -> bool {
        let Some(app) = window.application() else {
            return false; // TODO: better error
        };

        if !self.instances.contains_key(&self.ctx_menu_id) {
            let menu = gio::Menu::new();
            let widget = gtk::PopoverMenu::from_model(Some(&menu));

            let action_group = action_group_from_app(&app);
            window.insert_action_group(DEFAULT_ACTION_GROUP, Some(&action_group));

            let menu = GtkMenuChild::ContextMenu {
                id: self.ctx_menu_id,
                widget,
                menu,
                app,
            };

            self.instances.insert(self.ctx_menu_id, vec![menu]);

            for item in self.items() {
                let _ = self.add_menu_item_with_id(item.as_ref(), self.ctx_menu_id);
            }
        }

        // SAFETY: it is guaranteed to exist due to the check above
        let menus = self.instances.get(&self.ctx_menu_id).unwrap();
        let menu = menus.first().unwrap();

        let (x, y) = match position {
            Some(p) => p.to_logical::<i32>(window.scale_factor() as _).into(),
            None => get_cursor_pos(window),
        };

        let context_menu = menu.context_menu();

        if context_menu.parent().is_some() {
            context_menu.unparent();
        }
        context_menu.set_parent(window);

        context_menu.popup();
        context_menu.set_pointing_to(Some(&Rectangle::new(x, y, 0, 0)));

        true
    }
}

impl MenuChild {
    pub fn new(
        text: &str,
        enabled: bool,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        Self {
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            text: text.to_string(),
            enabled,
            key_accelerator,
            icon: None,
            checked: false,
            predefined_item_type: None,
            type_: MenuItemType::MenuItem,
            ctx_menu_id: 0,
            instances: HashMap::new(),
            children: Vec::new(),
            action: None,
            #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
            compat: compat_placeholder(),
        }
    }

    fn create_gtk_item_for_menu_item(
        &mut self,
        app: &gtk::Application,
        menu_id: u32,
        parent_menu: &gio::Menu,
    ) -> crate::Result<gio::MenuItem> {
        let detailed_action = self.detailed_action();
        let item = gio::MenuItem::new(Some(&to_gtk_mnemonic(&self.text)), Some(&detailed_action));

        if let Some(accelerator) = &self.key_accelerator {
            app.set_accels_for_action(&detailed_action, &[&accelerator.to_gtk()]);
        }

        if self.action.is_none() {
            let action_group = action_group_from_app(app);

            let action = gio::SimpleAction::new(self.id.as_ref(), None);
            let id = self.id.clone();
            action.connect_activate(move |_, _| MenuEvent::send(MenuEvent { id: id.clone() }));
            action.set_enabled(self.enabled);
            action_group.add_action(&action);

            self.action = Some(action);
        }

        let child = GtkMenuChild::Item {
            item: item.clone(),
            app: app.clone(),
            parent_menu: parent_menu.clone(),
        };
        self.instances.entry(menu_id).or_default().push(child);

        Ok(item)
    }

    pub fn id(&self) -> &MenuId {
        &self.id
    }

    fn detailed_action(&self) -> String {
        format!("{DEFAULT_ACTION_GROUP}.{}", self.id.as_ref())
    }

    pub fn item_type(&self) -> &MenuItemType {
        &self.type_
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();

        // GIO MenuItems are immutable after insertion, so we need to remove and reinsert
        let detailed_action = self.detailed_action();

        for children in self.instances.values_mut() {
            for child in children.iter_mut() {
                let parent_menu = child.parent_menu();

                // Find position of this item in parent menu by matching action name
                let mut position = None;
                for i in 0..parent_menu.n_items() {
                    if let Some(action) = parent_menu.item_attribute_value(i, "action", None) {
                        if let Some(action_str) = action.str() {
                            if action_str == detailed_action {
                                position = Some(i);
                                break;
                            }
                        }
                    }
                }

                if let Some(pos) = position {
                    // Remove old item
                    parent_menu.remove(pos);

                    // Create new item with updated text
                    let new_item = gio::MenuItem::new(
                        Some(&to_gtk_mnemonic(&self.text)),
                        Some(&detailed_action),
                    );

                    // Copy icon if present
                    if let Some(icon) = &self.icon {
                        new_item.set_icon(&icon.inner.to_bytes_icon());
                    }

                    // Insert at same position
                    parent_menu.insert_item(pos, &new_item);

                    // Update stored reference
                    match child {
                        GtkMenuChild::Item { item, .. } => *item = new_item,
                        GtkMenuChild::Submenu { item, .. } => *item = new_item,
                        _ => {}
                    }
                }
            }
        }

        #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
        self.notify_compat_changed();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if let Some(action) = self.action.as_ref() {
            action.set_enabled(enabled);
        }

        #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
        self.notify_compat_changed();
    }

    pub fn set_key_accelerator(
        &mut self,
        accelerator: Option<KeyAccelerator>,
    ) -> crate::Result<()> {
        self.key_accelerator = accelerator.clone();

        let detailed_action = self.detailed_action();
        let accelerator = accelerator.map(|a| a.to_gtk());
        let accelerator = accelerator.as_deref().map(|a| [a]).unwrap_or_default();
        for item in self.instances.values().flat_map(|v| v.iter()) {
            let app = item.application();
            app.set_accels_for_action(&detailed_action, accelerator.as_slice());
        }

        Ok(())
    }
}

impl MenuChild {
    pub fn new_predefined(item_type: PredefinedMenuItemType, text: Option<String>) -> Self {
        Self {
            id: MenuId(COUNTER.next().to_string()),
            text: text.unwrap_or_else(|| item_type.text().to_string()),
            enabled: true,
            key_accelerator: item_type.accelerator().map(KeyAccelerator::from),
            icon: None,
            checked: false,
            predefined_item_type: Some(item_type),
            type_: MenuItemType::Predefined,
            ctx_menu_id: 0,
            instances: HashMap::new(),
            children: Vec::new(),
            action: None,
            #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
            compat: compat_placeholder(),
        }
    }
}

impl MenuChild {
    pub fn new_check(
        text: &str,
        enabled: bool,
        checked: bool,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        Self {
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            text: text.to_string(),
            enabled,
            key_accelerator,
            icon: None,
            checked,
            predefined_item_type: None,
            type_: MenuItemType::Check,
            ctx_menu_id: 0,
            instances: HashMap::new(),
            children: Vec::new(),
            action: None,
            #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
            compat: compat_placeholder(),
        }
    }

    fn create_gtk_item_for_check_menu_item(
        &mut self,
        app: &gtk::Application,
        menu_id: u32,
        parent_menu: &gio::Menu,
    ) -> crate::Result<gio::MenuItem> {
        let detailed_action = self.detailed_action();
        let item = gio::MenuItem::new(Some(&to_gtk_mnemonic(&self.text)), Some(&detailed_action));

        if let Some(accelerator) = &self.key_accelerator {
            app.set_accels_for_action(&detailed_action, &[&accelerator.to_gtk()]);
        }

        if self.action.is_none() {
            let action_group = action_group_from_app(app);

            let state = &self.checked.to_variant();
            let action = gio::SimpleAction::new_stateful(self.id.as_ref(), None, state);
            let id = self.id.clone();
            action.connect_state_notify(move |_| MenuEvent::send(MenuEvent { id: id.clone() }));
            action.set_enabled(self.enabled);
            action_group.add_action(&action);

            self.action = Some(action);
        }

        let child = GtkMenuChild::Item {
            item: item.clone(),
            app: app.clone(),
            parent_menu: parent_menu.clone(),
        };
        self.instances.entry(menu_id).or_default().push(child);

        Ok(item)
    }

    pub fn is_checked(&self) -> bool {
        self.action
            .as_ref()
            .and_then(|action| action.state())
            .and_then(|s| s.get())
            .unwrap_or(self.checked)
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;

        if let Some(action) = self.action.as_ref() {
            action.set_state(&checked.to_variant());
        }

        #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
        self.notify_compat_changed();
    }
}

impl MenuChild {
    pub fn new_icon(
        text: &str,
        enabled: bool,
        icon: Option<Icon>,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        Self {
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            text: text.to_string(),
            enabled,
            key_accelerator,
            icon,
            checked: false,
            predefined_item_type: None,
            type_: MenuItemType::Icon,
            ctx_menu_id: 0,
            instances: HashMap::new(),
            children: Vec::new(),
            action: None,
            #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
            compat: compat_placeholder(),
        }
    }

    pub fn new_native_icon(
        text: &str,
        enabled: bool,
        _icon: Option<NativeIcon>,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        Self {
            id: id.unwrap_or_else(|| MenuId(COUNTER.next().to_string())),
            text: text.to_string(),
            enabled,
            key_accelerator,
            icon: None,
            checked: false,
            predefined_item_type: None,
            type_: MenuItemType::Icon,
            ctx_menu_id: 0,
            instances: HashMap::new(),
            children: Vec::new(),
            action: None,
            #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
            compat: compat_placeholder(),
        }
    }

    fn create_gtk_item_for_icon_menu_item(
        &mut self,
        app: &gtk::Application,
        menu_id: u32,
        parent_menu: &gio::Menu,
    ) -> crate::Result<gio::MenuItem> {
        let detailed_action = self.detailed_action();
        let item = gio::MenuItem::new(Some(&to_gtk_mnemonic(&self.text)), Some(&detailed_action));

        if let Some(accelerator) = &self.key_accelerator {
            app.set_accels_for_action(&detailed_action, &[&accelerator.to_gtk()]);
        }

        // GTK4's PopoverMenu ignores icons set on the model item, so mark
        // this row as custom and render the icon ourselves via `add_child`.
        if self.icon.is_some() {
            item.set_attribute_value("custom", Some(&self.id.as_ref().to_variant()));
        }

        if self.action.is_none() {
            let action_group = action_group_from_app(app);

            let action = gio::SimpleAction::new(self.id.as_ref(), None);
            let id = self.id.clone();
            action.connect_activate(move |_, _| MenuEvent::send(MenuEvent { id: id.clone() }));
            action.set_enabled(self.enabled);
            action_group.add_action(&action);

            self.action = Some(action);
        }

        let child = GtkMenuChild::Item {
            item: item.clone(),
            app: app.clone(),
            parent_menu: parent_menu.clone(),
        };
        self.instances.entry(menu_id).or_default().push(child);

        Ok(item)
    }

    /// The `custom` slot id and rendered row for an icon item, if any.
    fn custom_child(&self) -> Option<(String, gtk::Button)> {
        let icon = self.icon.as_ref()?;
        Some((
            self.id.as_ref().to_string(),
            custom_menu_row(icon, &self.text, &self.detailed_action()),
        ))
    }

    fn create_gtk_item_for_predefined_menu_item(
        &mut self,
        app: &gtk::Application,
        menu_id: u32,
        parent_menu: &gio::Menu,
    ) -> crate::Result<gio::MenuItem> {
        let predefined_item_type = self.predefined_item_type.clone().unwrap();

        let (label, action_name) = match &predefined_item_type {
            // Separator - create an empty section label (GIO way of doing separators)
            PredefinedMenuItemType::Separator => {
                // For separators, we return an item with no action that acts as a visual break
                // In GIO menus, true separators are done via sections, but this provides a fallback
                let item = gio::MenuItem::new(None, None);
                let child = GtkMenuChild::Item {
                    item: item.clone(),
                    app: app.clone(),
                    parent_menu: parent_menu.clone(),
                };
                self.instances.entry(menu_id).or_default().push(child);
                return Ok(item);
            }

            // Clipboard actions (widget-scoped, work on focused text widgets)
            PredefinedMenuItemType::Copy => (self.text.clone(), "clipboard.copy"),
            PredefinedMenuItemType::Cut => (self.text.clone(), "clipboard.cut"),
            PredefinedMenuItemType::Paste => (self.text.clone(), "clipboard.paste"),
            PredefinedMenuItemType::SelectAll => (self.text.clone(), "selection.select-all"),

            // Text actions (widget-scoped, work on focused text widgets)
            PredefinedMenuItemType::Undo => (self.text.clone(), "text.undo"),
            PredefinedMenuItemType::Redo => (self.text.clone(), "text.redo"),

            // Window actions (built-in on GtkWindow)
            PredefinedMenuItemType::Minimize => (self.text.clone(), "window.minimize"),
            PredefinedMenuItemType::Maximize => (self.text.clone(), "window.toggle-maximized"),
            PredefinedMenuItemType::CloseWindow => (self.text.clone(), "window.close"),

            // Fullscreen - no built-in GAction, need custom action
            PredefinedMenuItemType::Fullscreen => {
                let action_name = format!("{DEFAULT_ACTION_GROUP}.{}_fullscreen", self.id.as_ref());

                if self.action.is_none() {
                    let action_group = action_group_from_app(app);
                    let action =
                        gio::SimpleAction::new(&format!("{}_fullscreen", self.id.as_ref()), None);
                    action.connect_activate(|_, _| {
                        // Get the focused window and toggle fullscreen
                        if let Some(app) = gio::Application::default() {
                            if let Some(app) = app.downcast_ref::<gtk::Application>() {
                                if let Some(window) = app.active_window() {
                                    if window.is_fullscreen() {
                                        window.unfullscreen();
                                    } else {
                                        window.fullscreen();
                                    }
                                }
                            }
                        }
                    });
                    action_group.add_action(&action);
                    self.action = Some(action);
                }

                let item =
                    gio::MenuItem::new(Some(&to_gtk_mnemonic(&self.text)), Some(&action_name));
                let child = GtkMenuChild::Item {
                    item: item.clone(),
                    app: app.clone(),
                    parent_menu: parent_menu.clone(),
                };
                self.instances.entry(menu_id).or_default().push(child);
                return Ok(item);
            }

            // About - custom action showing AboutDialog
            PredefinedMenuItemType::About(metadata) => {
                let action_name = format!("{DEFAULT_ACTION_GROUP}.{}_about", self.id.as_ref());

                if self.action.is_none() {
                    let action_group = action_group_from_app(app);
                    let metadata = metadata.clone();
                    let action =
                        gio::SimpleAction::new(&format!("{}_about", self.id.as_ref()), None);
                    action.connect_activate(move |_, _| {
                        if let Some(metadata) = &metadata {
                            let dialog = gtk::AboutDialog::new();
                            dialog.set_modal(true);

                            if let Some(name) = &metadata.name {
                                dialog.set_program_name(Some(name.as_str()));
                            }
                            if let Some(version) = &metadata.full_version() {
                                dialog.set_version(Some(version.as_str()));
                            }
                            if let Some(authors) = &metadata.authors {
                                let authors_refs: Vec<&str> =
                                    authors.iter().map(|s| s.as_str()).collect();
                                dialog.set_authors(&authors_refs);
                            }
                            if let Some(comments) = &metadata.comments {
                                dialog.set_comments(Some(comments));
                            }
                            if let Some(copyright) = &metadata.copyright {
                                dialog.set_copyright(Some(copyright));
                            }
                            if let Some(license) = &metadata.license {
                                dialog.set_license(Some(license));
                            }
                            if let Some(website) = &metadata.website {
                                dialog.set_website(Some(website));
                            }
                            if let Some(website_label) = &metadata.website_label {
                                dialog.set_website_label(website_label);
                            }

                            // Set transient parent if possible
                            if let Some(app) = gio::Application::default() {
                                if let Some(app) = app.downcast_ref::<gtk::Application>() {
                                    if let Some(window) = app.active_window() {
                                        dialog.set_transient_for(Some(&window));
                                    }
                                }
                            }

                            dialog.present();
                        }
                    });
                    action_group.add_action(&action);
                    self.action = Some(action);
                }

                let item =
                    gio::MenuItem::new(Some(&to_gtk_mnemonic(&self.text)), Some(&action_name));
                let child = GtkMenuChild::Item {
                    item: item.clone(),
                    app: app.clone(),
                    parent_menu: parent_menu.clone(),
                };
                self.instances.entry(menu_id).or_default().push(child);
                return Ok(item);
            }

            // Unsupported on Linux (matches GTK3 behavior)
            PredefinedMenuItemType::Quit
            | PredefinedMenuItemType::Hide
            | PredefinedMenuItemType::HideOthers
            | PredefinedMenuItemType::ShowAll
            | PredefinedMenuItemType::Services
            | PredefinedMenuItemType::BringAllToFront
            | PredefinedMenuItemType::None => {
                unreachable!("Predefined menu item type not supported on Linux")
            }
        };

        // Create menu item pointing to the action
        let item = gio::MenuItem::new(Some(&to_gtk_mnemonic(&label)), Some(action_name));

        let child = GtkMenuChild::Item {
            item: item.clone(),
            app: app.clone(),
            parent_menu: parent_menu.clone(),
        };
        self.instances.entry(menu_id).or_default().push(child);

        Ok(item)
    }

    pub fn set_icon(&mut self, icon: Option<Icon>) {
        self.icon = icon;

        // GIO MenuItems are immutable after insertion, so we need to remove and reinsert
        let detailed_action = self.detailed_action();

        for children in self.instances.values_mut() {
            for child in children.iter_mut() {
                let parent_menu = child.parent_menu();

                // Find position of this item in parent menu by matching action name
                let mut position = None;
                for i in 0..parent_menu.n_items() {
                    if let Some(action) = parent_menu.item_attribute_value(i, "action", None) {
                        if let Some(action_str) = action.str() {
                            if action_str == detailed_action {
                                position = Some(i);
                                break;
                            }
                        }
                    }
                }

                if let Some(pos) = position {
                    // Remove old item
                    parent_menu.remove(pos);

                    // Create new item with updated icon
                    let new_item = gio::MenuItem::new(
                        Some(&to_gtk_mnemonic(&self.text)),
                        Some(&detailed_action),
                    );

                    // Set icon if present
                    if let Some(icon) = &self.icon {
                        new_item.set_icon(&icon.inner.to_bytes_icon());
                    }

                    // Insert at same position
                    parent_menu.insert_item(pos, &new_item);

                    // Update stored reference
                    match child {
                        GtkMenuChild::Item { item, .. } => *item = new_item,
                        GtkMenuChild::Submenu { item, .. } => *item = new_item,
                        _ => {}
                    }
                }
            }
        }

        #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
        self.notify_compat_changed();
    }
}

impl dyn IsMenuItem + '_ {
    fn make_gtk_menu_item(
        &self,
        app: &gtk::Application,
        menu_id: u32,
        parent_menu: &gio::Menu,
    ) -> crate::Result<gio::MenuItem> {
        let kind = self.kind();
        let mut child = kind.child_mut();
        match child.item_type() {
            MenuItemType::Submenu => child.create_gtk_item_for_submenu(app, menu_id, parent_menu),
            MenuItemType::MenuItem => {
                child.create_gtk_item_for_menu_item(app, menu_id, parent_menu)
            }
            MenuItemType::Check => {
                child.create_gtk_item_for_check_menu_item(app, menu_id, parent_menu)
            }
            MenuItemType::Icon => {
                child.create_gtk_item_for_icon_menu_item(app, menu_id, parent_menu)
            }
            MenuItemType::Predefined => {
                child.create_gtk_item_for_predefined_menu_item(app, menu_id, parent_menu)
            }
        }
    }
}

/// Returns and creates the action group on this application if necessary.
fn action_group_from_app(app: &gtk::Application) -> gio::SimpleActionGroup {
    let action_group = unsafe { app.data::<gio::SimpleActionGroup>(ACTION_GROUP_DATA_KEY) };

    let action_group = if let Some(action_group) = action_group {
        unsafe { action_group.as_ref() }.clone()
    } else {
        let action_group = gio::SimpleActionGroup::new();
        unsafe { app.set_data(ACTION_GROUP_DATA_KEY, action_group.clone()) };
        action_group
    };

    action_group
}

fn get_cursor_pos(window: &gtk::Window) -> (i32, i32) {
    WidgetExt::display(window)
        .default_seat()
        .and_then(|s| s.pointer())
        .map(|p| {
            let (_, x, y) = p.surface_at_position();
            (x as _, y as _)
        })
        .unwrap_or_default()
}
