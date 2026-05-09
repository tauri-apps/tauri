// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod accelerator;
mod dark_menu_bar;
mod icon;
mod util;

use self::dark_menu_bar::{WM_UAHDRAWMENU, WM_UAHDRAWMENUITEM};
pub(crate) use self::icon::WinIcon as PlatformIcon;

use crate::{
    accelerator::KeyAccelerator,
    dpi::Position,
    icon::{Icon, NativeIcon},
    items::PredefinedMenuItemType,
    util::{AddOp, Counter},
    AboutMetadata, IsMenuItem, MenuEvent, MenuId, MenuItemKind, MenuItemType, MenuTheme,
};
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
};
use util::{decode_wide, encode_wide, Accel};
use windows_sys::Win32::{
    Foundation::{FALSE, LPARAM, LRESULT, POINT, WPARAM},
    Graphics::Gdi::{ClientToScreen, HBITMAP},
    UI::{
        Input::KeyboardAndMouse::{
            GetActiveWindow, SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_CONTROL,
        },
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            AppendMenuW, CreateAcceleratorTableW, CreateMenu, CreatePopupMenu,
            DestroyAcceleratorTable, DestroyMenu, DrawMenuBar, EnableMenuItem, GetCursorPos,
            GetMenu, GetMenuItemInfoW, InsertMenuW, PostMessageW, PostQuitMessage, RemoveMenu,
            SendMessageW, SetForegroundWindow, SetMenu, SetMenuItemInfoW, ShowWindow,
            TrackPopupMenu, HACCEL, HMENU, MENUITEMINFOW, MFS_CHECKED, MFS_DISABLED, MF_BYCOMMAND,
            MF_BYPOSITION, MF_CHECKED, MF_DISABLED, MF_ENABLED, MF_GRAYED, MF_POPUP, MF_SEPARATOR,
            MF_STRING, MF_UNCHECKED, MIIM_BITMAP, MIIM_STATE, MIIM_STRING, SW_HIDE, SW_MAXIMIZE,
            SW_MINIMIZE, TPM_LEFTALIGN, TPM_RETURNCMD, WM_CLOSE, WM_COMMAND, WM_NCACTIVATE,
            WM_NCPAINT,
        },
    },
};

type Hwnd = isize;

static COUNTER: Counter = Counter::new_with_start(1000);

macro_rules! inner_menu_child_and_flags {
    ($item:ident) => {{
        let mut flags = 0;
        let child = match $item.kind() {
            MenuItemKind::Submenu(i) => {
                flags |= MF_POPUP;
                i.inner
            }
            MenuItemKind::MenuItem(i) => {
                flags |= MF_STRING;
                i.inner
            }

            MenuItemKind::Predefined(i) => {
                let child = i.inner;
                let child_ = child.borrow();
                match child_.predefined_item_type.as_ref().unwrap() {
                    PredefinedMenuItemType::None => return Ok(()),
                    PredefinedMenuItemType::Separator => {
                        flags |= MF_SEPARATOR;
                    }
                    _ => {
                        flags |= MF_STRING;
                    }
                }
                drop(child_);
                child
            }
            MenuItemKind::Check(i) => {
                let child = i.inner;
                flags |= MF_STRING;
                if child.borrow().checked {
                    flags |= MF_CHECKED;
                }
                child
            }
            MenuItemKind::Icon(i) => {
                flags |= MF_STRING;
                i.inner
            }
        };

        (child, flags)
    }};
}

type AccelWrapper = (HACCEL, HashMap<u32, Accel>);

#[derive(Debug)]
pub(crate) struct Menu {
    id: MenuId,
    internal_id: u32,
    hmenu: HMENU,
    hpopupmenu: HMENU,
    hwnds: Rc<RefCell<HashMap<Hwnd, MenuTheme>>>,
    haccel_store: Rc<RefCell<AccelWrapper>>,
    children: Vec<Rc<RefCell<MenuChild>>>,
}

impl Drop for Menu {
    fn drop(&mut self) {
        let hwnds = self.hwnds.borrow().keys().copied().collect::<Vec<_>>();
        for hwnd in &hwnds {
            let _ = unsafe { self.remove_for_hwnd(*hwnd) };
        }

        fn remove_from_children_stores(internal_id: u32, children: &Vec<Rc<RefCell<MenuChild>>>) {
            for child in children {
                let mut child_ = child.borrow_mut();
                child_.root_menu_haccel_stores.remove(&internal_id);
                if child_.item_type == MenuItemType::Submenu {
                    remove_from_children_stores(internal_id, child_.children.as_ref().unwrap());
                }
            }
        }

        remove_from_children_stores(self.internal_id, &self.children);

        for child in &self.children {
            let child_ = child.borrow();
            let id = if child_.item_type == MenuItemType::Submenu {
                child_.hmenu as _
            } else {
                child_.internal_id
            };
            unsafe {
                RemoveMenu(self.hpopupmenu, id, MF_BYCOMMAND);
                RemoveMenu(self.hmenu, id, MF_BYCOMMAND);
            }
        }

        unsafe {
            for hwnd in &hwnds {
                SetMenu(*hwnd as _, std::ptr::null_mut());
                RemoveWindowSubclass(*hwnd as _, Some(menu_subclass_proc), MENU_SUBCLASS_ID);
            }
            DestroyMenu(self.hmenu);
            DestroyMenu(self.hpopupmenu);
        }
    }
}

impl Menu {
    pub fn new(id: Option<MenuId>) -> Self {
        let internal_id = COUNTER.next();
        Self {
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            internal_id,
            hmenu: unsafe { CreateMenu() },
            hpopupmenu: unsafe { CreatePopupMenu() },
            haccel_store: Rc::new(RefCell::new((std::ptr::null_mut(), HashMap::new()))),
            children: Vec::new(),
            hwnds: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn id(&self) -> &MenuId {
        &self.id
    }

    pub fn add_menu_item(&mut self, item: &dyn IsMenuItem, op: AddOp) -> crate::Result<()> {
        let (child, mut flags) = inner_menu_child_and_flags!(item);

        {
            child
                .borrow_mut()
                .root_menu_haccel_stores
                .insert(self.internal_id, self.haccel_store.clone());
        }

        {
            let child_ = child.borrow();
            if !child_.enabled {
                flags |= MF_GRAYED;
            }

            let mut text = child_.text.clone();

            if let Some(accelerator) = &child_.accelerator {
                let accel_str = accelerator.to_string();

                text.push('\t');
                text.push_str(&accel_str);

                AccelAction::add(
                    &mut self.haccel_store.borrow_mut(),
                    child_.internal_id(),
                    accelerator,
                )?;
            }

            let id = child_.internal_id() as usize;

            let text = encode_wide(text);
            unsafe {
                match op {
                    AddOp::Append => {
                        AppendMenuW(self.hmenu, flags, id, text.as_ptr());
                        AppendMenuW(self.hpopupmenu, flags, id, text.as_ptr());
                    }
                    AddOp::Insert(position) => {
                        InsertMenuW(
                            self.hmenu,
                            position as _,
                            flags | MF_BYPOSITION,
                            id,
                            text.as_ptr(),
                        );
                        InsertMenuW(
                            self.hpopupmenu,
                            position as _,
                            flags | MF_BYPOSITION,
                            id,
                            text.as_ptr(),
                        );
                    }
                }
            }
        }

        {
            let child_ = child.borrow();
            let item_type = child_.item_type();

            // Set icons for both regular menu items and submenus
            if matches!(item_type, MenuItemType::Icon | MenuItemType::Submenu) {
                let hbitmap = child_
                    .icon
                    .as_ref()
                    .map(|i| unsafe { i.inner.to_hbitmap() })
                    .unwrap_or(std::ptr::null_mut());

                let info = create_icon_item_info(hbitmap);

                unsafe {
                    SetMenuItemInfoW(self.hmenu, child_.internal_id(), FALSE, &info);
                    SetMenuItemInfoW(self.hpopupmenu, child_.internal_id(), FALSE, &info);
                };
            }
        }

        // redraw the menu bar
        for hwnd in self.hwnds.borrow().keys() {
            unsafe { DrawMenuBar(*hwnd as _) };
        }

        {
            let mut child_ = child.borrow_mut();
            child_
                .parents_hemnu
                .push((self.hmenu, Some(self.hwnds.clone())));
            child_.parents_hemnu.push((self.hpopupmenu, None));
        }

        {
            match op {
                AddOp::Append => self.children.push(child),
                AddOp::Insert(position) => self.children.insert(position, child),
            }
        }

        Ok(())
    }

    pub fn remove(&mut self, item: &dyn IsMenuItem) -> crate::Result<()> {
        let id = item.child().borrow().internal_id();
        unsafe {
            RemoveMenu(self.hmenu, id, MF_BYCOMMAND);
            RemoveMenu(self.hpopupmenu, id, MF_BYCOMMAND);

            // redraw the menu bar
            for hwnd in self.hwnds.borrow().keys() {
                DrawMenuBar(*hwnd as _);
            }
        }

        let child = item.child();

        {
            let mut child = child.borrow_mut();
            let index = child
                .parents_hemnu
                .iter()
                .position(|&(h, _)| h == self.hmenu)
                .ok_or(crate::Error::NotAChildOfThisMenu)?;
            child.parents_hemnu.remove(index);
            let index = child
                .parents_hemnu
                .iter()
                .position(|&(h, _)| h == self.hpopupmenu)
                .ok_or(crate::Error::NotAChildOfThisMenu)?;
            child.parents_hemnu.remove(index);
        }

        let index = self
            .children
            .iter()
            .position(|e| e.borrow().internal_id() == id)
            .ok_or(crate::Error::NotAChildOfThisMenu)?;
        self.children.remove(index);

        Ok(())
    }

    pub fn items(&self) -> Vec<MenuItemKind> {
        self.children
            .iter()
            .map(|c| c.borrow().kind(c.clone()))
            .collect()
    }

    fn find_by_id(&self, id: u32) -> Option<Rc<RefCell<MenuChild>>> {
        find_by_id(id, &self.children)
    }

    pub fn haccel(&self) -> isize {
        self.haccel_store.borrow().0 as _
    }

    pub fn hpopupmenu(&self) -> isize {
        self.hpopupmenu as _
    }

    pub unsafe fn init_for_hwnd_with_theme(
        &mut self,
        hwnd: isize,
        theme: MenuTheme,
    ) -> crate::Result<()> {
        if self.hwnds.borrow().contains_key(&hwnd) {
            return Err(crate::Error::AlreadyInitialized);
        }

        self.hwnds.borrow_mut().insert(hwnd, theme);

        // SAFETY: HWND validity is upheld by caller
        SetMenu(hwnd as _, self.hmenu);
        SetWindowSubclass(
            hwnd as _,
            Some(menu_subclass_proc),
            MENU_SUBCLASS_ID,
            dwrefdata_from_obj(self),
        );
        DrawMenuBar(hwnd as _);

        Ok(())
    }

    pub unsafe fn init_for_hwnd(&mut self, hwnd: isize) -> crate::Result<()> {
        self.init_for_hwnd_with_theme(hwnd, MenuTheme::Auto)
    }

    pub unsafe fn remove_for_hwnd(&mut self, hwnd: isize) -> crate::Result<()> {
        self.hwnds
            .borrow_mut()
            .remove(&hwnd)
            .ok_or(crate::Error::NotInitialized)?;

        // SAFETY: HWND validity is upheld by caller
        SetMenu(hwnd as _, std::ptr::null_mut());
        DrawMenuBar(hwnd as _);

        Ok(())
    }

    pub unsafe fn attach_menu_subclass_for_hwnd(&self, hwnd: isize) {
        // SAFETY: HWND validity is upheld by caller
        SetWindowSubclass(
            hwnd as _,
            Some(menu_subclass_proc),
            MENU_SUBCLASS_ID,
            dwrefdata_from_obj(self),
        );
    }

    pub unsafe fn detach_menu_subclass_from_hwnd(&self, hwnd: isize) {
        // SAFETY: HWND validity is upheld by caller
        RemoveWindowSubclass(hwnd as _, Some(menu_subclass_proc), MENU_SUBCLASS_ID);
    }

    pub unsafe fn hide_for_hwnd(&self, hwnd: isize) -> crate::Result<()> {
        if !self.hwnds.borrow().contains_key(&hwnd) {
            return Err(crate::Error::NotInitialized);
        }

        // SAFETY: HWND validity is upheld by caller
        SetMenu(hwnd as _, std::ptr::null_mut());
        DrawMenuBar(hwnd as _);

        Ok(())
    }

    pub unsafe fn show_for_hwnd(&self, hwnd: isize) -> crate::Result<()> {
        if !self.hwnds.borrow().contains_key(&hwnd) {
            return Err(crate::Error::NotInitialized);
        }

        // SAFETY: HWND validity is upheld by caller
        SetMenu(hwnd as _, self.hmenu);
        DrawMenuBar(hwnd as _);

        Ok(())
    }

    pub unsafe fn is_visible_on_hwnd(&self, hwnd: isize) -> bool {
        self.hwnds
            .borrow()
            .get(&hwnd)
            // SAFETY: HWND validity is upheld by caller
            .map(|_| !unsafe { GetMenu(hwnd as _) }.is_null())
            .unwrap_or(false)
    }

    pub unsafe fn show_context_menu_for_hwnd(
        &mut self,
        hwnd: isize,
        position: Option<Position>,
    ) -> bool {
        let rc = show_context_menu(hwnd as _, self.hpopupmenu, position);
        if let Some(item) = rc.and_then(|rc| self.find_by_id(rc)) {
            unsafe {
                menu_selected(hwnd as _, &mut item.borrow_mut());
            }
            return true;
        }
        false
    }

    pub unsafe fn set_theme_for_hwnd(&self, hwnd: isize, theme: MenuTheme) -> crate::Result<()> {
        if !self.hwnds.borrow().contains_key(&hwnd) {
            return Err(crate::Error::NotInitialized);
        }

        // SAFETY: HWND validity is upheld by caller
        SendMessageW(hwnd as _, MENU_UPDATE_THEME, 0, theme as _);

        Ok(())
    }
}

type ParentMenu = (HMENU, Option<Rc<RefCell<HashMap<Hwnd, MenuTheme>>>>);

/// A generic child in a menu
#[derive(Debug)]
pub(crate) struct MenuChild {
    // shared fields between submenus and menu items
    item_type: MenuItemType,
    text: String,
    enabled: bool,
    parents_hemnu: Vec<ParentMenu>,
    root_menu_haccel_stores: HashMap<u32, Rc<RefCell<AccelWrapper>>>,

    // menu item fields
    internal_id: u32,
    id: MenuId,
    accelerator: Option<KeyAccelerator>,

    // predefined menu item fields
    predefined_item_type: Option<PredefinedMenuItemType>,

    // check menu item fields
    checked: bool,

    // icon menu item fields
    icon: Option<Icon>,

    // submenu fields
    hmenu: HMENU,
    hpopupmenu: HMENU,
    pub children: Option<Vec<Rc<RefCell<MenuChild>>>>,
}

impl Drop for MenuChild {
    fn drop(&mut self) {
        if self.item_type == MenuItemType::Submenu {
            unsafe {
                DestroyMenu(self.hmenu);
                DestroyMenu(self.hpopupmenu);
            }
        }

        if self.accelerator.is_some() {
            for store in self.root_menu_haccel_stores.values() {
                AccelAction::remove(&mut store.borrow_mut(), self.internal_id)
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
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::MenuItem,
            text: text.to_string(),
            enabled,
            parents_hemnu: Vec::new(),
            internal_id,
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            accelerator: key_accelerator,
            root_menu_haccel_stores: HashMap::new(),
            predefined_item_type: None,
            icon: None,
            checked: false,
            children: None,
            hmenu: std::ptr::null_mut(),
            hpopupmenu: std::ptr::null_mut(),
        }
    }

    pub fn new_submenu(text: &str, enabled: bool, id: Option<MenuId>) -> Self {
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::Submenu,
            text: text.to_string(),
            enabled,
            parents_hemnu: Vec::new(),
            children: Some(Vec::new()),
            hmenu: unsafe { CreateMenu() },
            internal_id,
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            hpopupmenu: unsafe { CreatePopupMenu() },
            root_menu_haccel_stores: HashMap::new(),
            predefined_item_type: None,
            icon: None,
            checked: false,
            accelerator: None,
        }
    }

    pub fn new_predefined(item_type: PredefinedMenuItemType, text: Option<String>) -> Self {
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::Predefined,
            text: text.unwrap_or_else(|| item_type.text().to_string()),
            enabled: true,
            parents_hemnu: Vec::new(),
            internal_id,
            id: MenuId::new(internal_id.to_string()),
            accelerator: item_type.accelerator().map(Into::into),
            predefined_item_type: Some(item_type),
            root_menu_haccel_stores: HashMap::new(),
            icon: None,
            checked: false,
            children: None,
            hmenu: std::ptr::null_mut(),
            hpopupmenu: std::ptr::null_mut(),
        }
    }

    pub fn new_check(
        text: &str,
        enabled: bool,
        checked: bool,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::Check,
            text: text.to_string(),
            enabled,
            parents_hemnu: Vec::new(),
            internal_id,
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            accelerator: key_accelerator,
            checked,
            root_menu_haccel_stores: HashMap::new(),
            predefined_item_type: None,
            icon: None,
            children: None,
            hmenu: std::ptr::null_mut(),
            hpopupmenu: std::ptr::null_mut(),
        }
    }

    pub fn new_icon(
        text: &str,
        enabled: bool,
        icon: Option<Icon>,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::Icon,
            text: text.to_string(),
            enabled,
            parents_hemnu: Vec::new(),
            internal_id,
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            accelerator: key_accelerator,
            icon,
            root_menu_haccel_stores: HashMap::new(),
            predefined_item_type: None,
            checked: false,
            children: None,
            hmenu: std::ptr::null_mut(),
            hpopupmenu: std::ptr::null_mut(),
        }
    }

    pub fn new_native_icon(
        text: &str,
        enabled: bool,
        _native_icon: Option<NativeIcon>,
        key_accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::Icon,
            text: text.to_string(),
            enabled,
            parents_hemnu: Vec::new(),
            internal_id,
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            accelerator: key_accelerator,
            root_menu_haccel_stores: HashMap::new(),
            predefined_item_type: None,
            icon: None,
            checked: false,
            children: None,
            hmenu: std::ptr::null_mut(),
            hpopupmenu: std::ptr::null_mut(),
        }
    }
}

/// Shared methods
impl MenuChild {
    pub fn item_type(&self) -> MenuItemType {
        self.item_type
    }

    pub fn id(&self) -> &MenuId {
        &self.id
    }

    pub fn internal_id(&self) -> u32 {
        match self.item_type() {
            MenuItemType::Submenu => self.hmenu as u32,
            _ => self.internal_id,
        }
    }

    pub fn text(&self) -> String {
        self.parents_hemnu
            .first()
            .map(|(hmenu, _)| {
                let id = self.internal_id();
                let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
                info.cbSize = std::mem::size_of::<MENUITEMINFOW>() as _;
                info.fMask = MIIM_STRING;

                unsafe { GetMenuItemInfoW(*hmenu, id, FALSE, &mut info) };

                info.cch += 1;
                let mut dw_type_data = Vec::with_capacity(info.cch as usize);
                info.dwTypeData = dw_type_data.as_mut_ptr();

                unsafe { GetMenuItemInfoW(*hmenu, id, FALSE, &mut info) };

                let text = decode_wide(info.dwTypeData);
                text.split('\t').next().unwrap().to_string()
            })
            .unwrap_or_else(|| self.text.clone())
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        let mut text = if let Some(accelerator) = &self.accelerator {
            encode_wide(format!("{text}\t{}", accelerator))
        } else {
            encode_wide(text)
        };

        for (parent, menu_bars) in &self.parents_hemnu {
            let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
            info.cbSize = std::mem::size_of::<MENUITEMINFOW>() as _;
            info.fMask = MIIM_STRING;
            info.dwTypeData = text.as_mut_ptr();

            unsafe { SetMenuItemInfoW(*parent, self.internal_id(), FALSE, &info) };

            if let Some(menu_bars) = menu_bars {
                for hwnd in menu_bars.borrow().keys() {
                    unsafe { DrawMenuBar(*hwnd as _) };
                }
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.parents_hemnu
            .first()
            .map(|(hmenu, _)| {
                let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
                info.cbSize = std::mem::size_of::<MENUITEMINFOW>() as _;
                info.fMask = MIIM_STATE;

                unsafe { GetMenuItemInfoW(*hmenu, self.internal_id(), FALSE, &mut info) };

                (info.fState & MFS_DISABLED) == 0
            })
            .unwrap_or(self.enabled)
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        for (parent, menu_bars) in &self.parents_hemnu {
            let flag = if enabled { MF_ENABLED } else { MF_DISABLED };
            unsafe { EnableMenuItem(*parent, self.internal_id(), flag) };

            if let Some(menu_bars) = menu_bars {
                for hwnd in menu_bars.borrow().keys() {
                    unsafe { DrawMenuBar(*hwnd as _) };
                }
            };
        }
    }

    pub fn set_key_accelerator(
        &mut self,
        accelerator: Option<KeyAccelerator>,
    ) -> crate::Result<()> {
        self.accelerator = accelerator;
        self.set_text(&self.text.clone());

        for store in self.root_menu_haccel_stores.values() {
            let mut store = store.borrow_mut();
            if let Some(accelerator) = &self.accelerator {
                AccelAction::add(&mut store, self.internal_id, accelerator)?
            } else {
                AccelAction::remove(&mut store, self.internal_id)
            }
        }

        Ok(())
    }
}

/// CheckMenuItem methods
impl MenuChild {
    pub fn is_checked(&self) -> bool {
        self.parents_hemnu
            .first()
            .map(|(hmenu, _)| {
                let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
                info.cbSize = std::mem::size_of::<MENUITEMINFOW>() as _;
                info.fMask = MIIM_STATE;

                unsafe { GetMenuItemInfoW(*hmenu, self.internal_id(), FALSE, &mut info) };

                (info.fState & MFS_CHECKED) != 0
            })
            .unwrap_or(self.enabled)
    }

    pub fn set_checked(&mut self, checked: bool) {
        use windows_sys::Win32::UI::WindowsAndMessaging;

        self.checked = checked;
        for (parent, menu_bars) in &self.parents_hemnu {
            let flag = if checked { MF_CHECKED } else { MF_UNCHECKED };
            unsafe { WindowsAndMessaging::CheckMenuItem(*parent, self.internal_id(), flag) };

            if let Some(menu_bars) = menu_bars {
                for hwnd in menu_bars.borrow().keys() {
                    unsafe { DrawMenuBar(*hwnd as _) };
                }
            };
        }
    }
}

/// IconMenuItem methods
impl MenuChild {
    pub fn set_icon(&mut self, icon: Option<Icon>) {
        self.icon.clone_from(&icon);

        let hbitmap = icon
            .map(|i| unsafe { i.inner.to_hbitmap() })
            .unwrap_or(std::ptr::null_mut());

        let info = create_icon_item_info(hbitmap);

        for (parent, menu_bars) in &self.parents_hemnu {
            unsafe { SetMenuItemInfoW(*parent, self.internal_id(), FALSE, &info) };

            if let Some(menu_bars) = menu_bars {
                for hwnd in menu_bars.borrow().keys() {
                    unsafe { DrawMenuBar(*hwnd as _) };
                }
            };
        }
    }
}

/// Submenu methods
impl MenuChild {
    pub fn hpopupmenu(&self) -> isize {
        self.hpopupmenu as _
    }

    pub fn add_menu_item(&mut self, item: &dyn IsMenuItem, op: AddOp) -> crate::Result<()> {
        let (child, mut flags) = inner_menu_child_and_flags!(item);

        {
            child
                .borrow_mut()
                .root_menu_haccel_stores
                .extend(self.root_menu_haccel_stores.clone());
        }

        {
            let child_ = child.borrow();
            if !child_.enabled {
                flags |= MF_GRAYED;
            }

            let mut text = child_.text.clone();

            if let Some(accelerator) = &child_.accelerator {
                let accel_str = accelerator.to_string();

                text.push('\t');
                text.push_str(&accel_str);

                for root_menu in self.root_menu_haccel_stores.values() {
                    let mut haccel = root_menu.borrow_mut();
                    AccelAction::add(&mut haccel, child_.internal_id(), accelerator)?;
                }
            }

            let id = child_.internal_id() as usize;
            let text = encode_wide(text);
            unsafe {
                match op {
                    AddOp::Append => {
                        AppendMenuW(self.hmenu, flags, id, text.as_ptr());
                        AppendMenuW(self.hpopupmenu, flags, id, text.as_ptr());
                    }
                    AddOp::Insert(position) => {
                        InsertMenuW(
                            self.hmenu,
                            position as _,
                            flags | MF_BYPOSITION,
                            id,
                            text.as_ptr(),
                        );
                        InsertMenuW(
                            self.hpopupmenu,
                            position as _,
                            flags | MF_BYPOSITION,
                            id,
                            text.as_ptr(),
                        );
                    }
                }
            }
        }

        {
            let child_ = child.borrow();
            let item_type = child_.item_type();

            if matches!(item_type, MenuItemType::Icon | MenuItemType::Submenu) {
                let hbitmap = child_
                    .icon
                    .as_ref()
                    .map(|i| unsafe { i.inner.to_hbitmap() })
                    .unwrap_or(std::ptr::null_mut());
                let info = create_icon_item_info(hbitmap);
                unsafe {
                    SetMenuItemInfoW(self.hmenu, child_.internal_id(), FALSE, &info);
                    SetMenuItemInfoW(self.hpopupmenu, child_.internal_id(), FALSE, &info);
                };
            }
        }

        {
            let mut child_ = child.borrow_mut();
            child_.parents_hemnu.push((self.hmenu, None));
            child_.parents_hemnu.push((self.hpopupmenu, None));
        }

        {
            let children = self.children.as_mut().unwrap();
            match op {
                AddOp::Append => children.push(child),
                AddOp::Insert(position) => children.insert(position, child),
            }
        }

        Ok(())
    }

    pub fn remove(&mut self, item: &dyn IsMenuItem) -> crate::Result<()> {
        let id = item.child().borrow().internal_id();
        unsafe {
            RemoveMenu(self.hmenu, id, MF_BYCOMMAND);
            RemoveMenu(self.hpopupmenu, id, MF_BYCOMMAND);
        }

        let child = item.child();

        {
            let mut child = child.borrow_mut();
            let index = child
                .parents_hemnu
                .iter()
                .position(|&(h, _)| h == self.hmenu)
                .ok_or(crate::Error::NotAChildOfThisMenu)?;
            child.parents_hemnu.remove(index);
            let index = child
                .parents_hemnu
                .iter()
                .position(|&(h, _)| h == self.hpopupmenu)
                .ok_or(crate::Error::NotAChildOfThisMenu)?;
            child.parents_hemnu.remove(index);
        }

        let children = self.children.as_mut().unwrap();
        let index = children
            .iter()
            .position(|e| e.borrow().internal_id() == id)
            .ok_or(crate::Error::NotAChildOfThisMenu)?;
        children.remove(index);

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

    pub unsafe fn show_context_menu_for_hwnd(
        &mut self,
        hwnd: isize,
        position: Option<Position>,
    ) -> bool {
        let rc = show_context_menu(hwnd as _, self.hpopupmenu, position);
        if let Some(item) = rc.and_then(|rc| self.find_by_id(rc)) {
            unsafe {
                menu_selected(hwnd as _, &mut item.borrow_mut());
            }
            return true;
        }
        false
    }

    pub unsafe fn attach_menu_subclass_for_hwnd(&self, hwnd: isize) {
        // SAFETY: HWND validity is upheld by caller
        SetWindowSubclass(
            hwnd as _,
            Some(menu_subclass_proc),
            SUBMENU_SUBCLASS_ID,
            dwrefdata_from_obj(self),
        );
    }

    pub unsafe fn detach_menu_subclass_from_hwnd(&self, hwnd: isize) {
        // SAFETY: HWND validity is upheld by caller
        RemoveWindowSubclass(hwnd as _, Some(menu_subclass_proc), SUBMENU_SUBCLASS_ID);
    }
}

/// Internal Utilitles
impl MenuChild {
    fn find_by_id(&self, id: u32) -> Option<Rc<RefCell<MenuChild>>> {
        let children = self.children.as_ref().unwrap();
        find_by_id(id, children)
    }
}

fn find_by_id(id: u32, children: &Vec<Rc<RefCell<MenuChild>>>) -> Option<Rc<RefCell<MenuChild>>> {
    for i in children {
        let item = i.borrow();
        if item.internal_id() == id {
            return Some(i.clone());
        }

        if item.item_type() == MenuItemType::Submenu {
            if let Some(child) = item.find_by_id(id) {
                return Some(child);
            }
        }
    }
    None
}

// SAFETY:
// HWND validity is upheld by caller
unsafe fn show_context_menu(
    hwnd: windows_sys::Win32::Foundation::HWND,
    hmenu: HMENU,
    position: Option<Position>,
) -> Option<u32> {
    let result = unsafe {
        let pt = if let Some(pos) = position {
            let dpi = util::hwnd_dpi(hwnd);
            let scale_factor = util::dpi_to_scale_factor(dpi);
            let pos = pos.to_physical::<i32>(scale_factor);
            let mut pt = POINT {
                x: pos.x as _,
                y: pos.y as _,
            };
            ClientToScreen(hwnd, &mut pt);
            pt
        } else {
            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            pt
        };
        SetForegroundWindow(hwnd);
        TrackPopupMenu(
            hmenu,
            TPM_LEFTALIGN | TPM_RETURNCMD,
            pt.x,
            pt.y,
            0,
            hwnd,
            std::ptr::null(),
        )
    };
    (result > 0).then_some(result.try_into().ok()).flatten()
}

struct AccelAction;

impl AccelAction {
    fn add(
        haccel_store: &mut RefMut<AccelWrapper>,
        id: u32,
        accelerator: &KeyAccelerator,
    ) -> crate::Result<()> {
        let accel = accelerator.to_accel(id as _)?;
        haccel_store.1.insert(id, Accel(accel));
        Self::update_store(haccel_store);
        Ok(())
    }

    fn remove(haccel_store: &mut RefMut<AccelWrapper>, id: u32) {
        haccel_store.1.remove(&id);
        Self::update_store(haccel_store)
    }

    fn update_store(haccel_store: &mut RefMut<AccelWrapper>) {
        unsafe {
            DestroyAcceleratorTable(haccel_store.0);
            let len = haccel_store.1.len();
            let accels = haccel_store.1.values().map(|i| i.0).collect::<Vec<_>>();
            haccel_store.0 = CreateAcceleratorTableW(accels.as_ptr(), len as _);
        }
    }
}

fn create_icon_item_info(hbitmap: HBITMAP) -> MENUITEMINFOW {
    let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<MENUITEMINFOW>() as _;
    info.fMask = MIIM_BITMAP;
    info.hbmpItem = hbitmap;
    info
}

fn dwrefdata_from_obj<T>(obj: &T) -> usize {
    (obj as *const T) as usize
}

unsafe fn obj_from_dwrefdata<T>(dwrefdata: usize) -> &'static mut T {
    unsafe { (dwrefdata as *mut T).as_mut().unwrap() }
}

const MENU_SUBCLASS_ID: usize = 200;
const MENU_UPDATE_THEME: u32 = 201;
const SUBMENU_SUBCLASS_ID: usize = 202;

unsafe extern "system" fn menu_subclass_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    uidsubclass: usize,
    dwrefdata: usize,
) -> LRESULT {
    match msg {
        MENU_UPDATE_THEME if uidsubclass == MENU_SUBCLASS_ID => {
            let menu = obj_from_dwrefdata::<Menu>(dwrefdata);
            let theme: MenuTheme = std::mem::transmute(lparam);
            menu.hwnds.borrow_mut().insert(hwnd as _, theme);
            if GetActiveWindow() == hwnd {
                PostMessageW(hwnd, WM_NCACTIVATE, 0, 0);
                PostMessageW(hwnd, WM_NCACTIVATE, true.into(), 0);
            } else {
                PostMessageW(hwnd, WM_NCACTIVATE, true.into(), 0);
                PostMessageW(hwnd, WM_NCACTIVATE, 0, 0);
            }
            0
        }

        WM_COMMAND => {
            let id = util::LOWORD(wparam as _) as u32;

            let item = match uidsubclass {
                MENU_SUBCLASS_ID => {
                    let menu = obj_from_dwrefdata::<Menu>(dwrefdata);
                    menu.find_by_id(id)
                }
                SUBMENU_SUBCLASS_ID => {
                    let menu = obj_from_dwrefdata::<MenuChild>(dwrefdata);
                    menu.find_by_id(id)
                }
                _ => unreachable!(),
            };

            if let Some(item) = item {
                menu_selected(hwnd, &mut item.borrow_mut());
                0
            } else {
                DefSubclassProc(hwnd as _, msg, wparam, lparam)
            }
        }

        WM_UAHDRAWMENUITEM | WM_UAHDRAWMENU if uidsubclass == MENU_SUBCLASS_ID => {
            let menu = obj_from_dwrefdata::<Menu>(dwrefdata);
            let theme = menu
                .hwnds
                .borrow()
                .get(&(hwnd as _))
                .copied()
                .unwrap_or(MenuTheme::Auto);
            if theme.should_use_dark(hwnd as _) {
                dark_menu_bar::draw(hwnd as _, msg, wparam, lparam);
                0
            } else {
                DefSubclassProc(hwnd as _, msg, wparam, lparam)
            }
        }
        WM_NCACTIVATE | WM_NCPAINT => {
            // DefSubclassProc needs to be called before calling the
            // custom dark menu redraw
            let res = DefSubclassProc(hwnd as _, msg, wparam, lparam);

            let menu = obj_from_dwrefdata::<Menu>(dwrefdata);
            let theme = menu
                .hwnds
                .borrow()
                .get(&(hwnd as _))
                .copied()
                .unwrap_or(MenuTheme::Auto);
            if theme.should_use_dark(hwnd as _) {
                dark_menu_bar::draw(hwnd as _, msg, wparam, lparam);
            }

            res
        }
        _ => DefSubclassProc(hwnd as _, msg, wparam, lparam),
    }
}

unsafe fn menu_selected(hwnd: windows_sys::Win32::Foundation::HWND, item: &mut MenuChild) {
    let (mut dispatch, mut menu_id) = (true, None);

    {
        if item.item_type() == MenuItemType::Predefined {
            dispatch = false;
        } else {
            menu_id.replace(item.id.clone());
        }

        match item.item_type() {
            MenuItemType::Check => {
                let checked = !item.checked;
                item.set_checked(checked);
            }
            MenuItemType::Predefined => {
                if let Some(predefined_item_type) = &item.predefined_item_type {
                    match predefined_item_type {
                        PredefinedMenuItemType::Copy => execute_edit_command(EditCommand::Copy),
                        PredefinedMenuItemType::Cut => execute_edit_command(EditCommand::Cut),
                        PredefinedMenuItemType::Paste => execute_edit_command(EditCommand::Paste),
                        PredefinedMenuItemType::SelectAll => {
                            execute_edit_command(EditCommand::SelectAll)
                        }
                        PredefinedMenuItemType::Undo => execute_edit_command(EditCommand::Undo),
                        PredefinedMenuItemType::Redo => execute_edit_command(EditCommand::Redo),
                        PredefinedMenuItemType::Separator => {}
                        PredefinedMenuItemType::Minimize => {
                            ShowWindow(hwnd, SW_MINIMIZE);
                        }
                        PredefinedMenuItemType::Maximize => {
                            ShowWindow(hwnd, SW_MAXIMIZE);
                        }
                        PredefinedMenuItemType::Hide => {
                            ShowWindow(hwnd, SW_HIDE);
                        }
                        PredefinedMenuItemType::CloseWindow => {
                            SendMessageW(hwnd, WM_CLOSE, 0, 0);
                        }
                        PredefinedMenuItemType::Quit => {
                            PostQuitMessage(0);
                        }
                        PredefinedMenuItemType::About(Some(ref metadata)) => {
                            show_about_dialog(hwnd as _, metadata)
                        }

                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if dispatch {
        MenuEvent::send(MenuEvent {
            id: menu_id.unwrap(),
        });
    }
}

impl MenuTheme {
    fn should_use_dark(&self, hwnd: isize) -> bool {
        match self {
            MenuTheme::Dark => true,
            MenuTheme::Auto if dark_menu_bar::should_use_dark_mode(hwnd as _) => true,
            _ => false,
        }
    }
}

enum EditCommand {
    Copy,
    Cut,
    Paste,
    SelectAll,
    Undo,
    Redo,
}

fn execute_edit_command(command: EditCommand) {
    let key = match command {
        EditCommand::Copy => 0x43,      // c
        EditCommand::Cut => 0x58,       // x
        EditCommand::Paste => 0x56,     // v
        EditCommand::SelectAll => 0x41, // a
        EditCommand::Undo => 0x5A,      // z
        EditCommand::Redo => 0x59,      // y
    };

    unsafe {
        let mut inputs: [INPUT; 4] = std::mem::zeroed();
        inputs[0].r#type = INPUT_KEYBOARD;
        inputs[0].Anonymous.ki.wVk = VK_CONTROL;
        inputs[0].Anonymous.ki.dwFlags = 0;

        inputs[1].r#type = INPUT_KEYBOARD;
        inputs[1].Anonymous.ki.wVk = key;
        inputs[1].Anonymous.ki.dwFlags = 0;

        inputs[2].r#type = INPUT_KEYBOARD;
        inputs[2].Anonymous.ki.wVk = key;
        inputs[2].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

        inputs[3].r#type = INPUT_KEYBOARD;
        inputs[3].Anonymous.ki.wVk = VK_CONTROL;
        inputs[3].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

        SendInput(4, &inputs as *const _, std::mem::size_of::<INPUT>() as _);
    }
}

fn show_about_dialog(hwnd: Hwnd, metadata: &AboutMetadata) {
    use std::fmt::Write;

    let mut message = String::new();
    if let Some(name) = &metadata.name {
        let _ = writeln!(&mut message, "Name: {}", name);
    }
    if let Some(version) = &metadata.full_version() {
        let _ = writeln!(&mut message, "Version: {}", version);
    }
    if let Some(authors) = &metadata.authors {
        let _ = writeln!(&mut message, "Authors: {}", authors.join(", "));
    }
    if let Some(license) = &metadata.license {
        let _ = writeln!(&mut message, "License: {}", license);
    }
    match (&metadata.website_label, &metadata.website) {
        (Some(label), None) => {
            let _ = writeln!(&mut message, "Website: {}", label);
        }
        (None, Some(url)) => {
            let _ = writeln!(&mut message, "Website: {}", url);
        }
        (Some(label), Some(url)) => {
            let _ = writeln!(&mut message, "Website: {} {}", label, url);
        }
        _ => {}
    }
    if let Some(comments) = &metadata.comments {
        let _ = writeln!(&mut message, "\n{}", comments);
    }
    if let Some(copyright) = &metadata.copyright {
        let _ = writeln!(&mut message, "\n{}", copyright);
    }

    let message = encode_wide(message);
    let title = encode_wide(format!(
        "About {}",
        metadata.name.as_deref().unwrap_or_default()
    ));

    #[cfg(not(feature = "common-controls-v6"))]
    std::thread::spawn(move || unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION};
        MessageBoxW(
            hwnd as _,
            message.as_ptr(),
            title.as_ptr(),
            MB_ICONINFORMATION,
        );
    });

    #[cfg(feature = "common-controls-v6")]
    {
        use windows_sys::Win32::UI::Controls::{
            TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0, TASKDIALOGCONFIG_1,
            TDCBF_OK_BUTTON, TDF_ALLOW_DIALOG_CANCELLATION, TD_INFORMATION_ICON,
        };

        std::thread::spawn(move || unsafe {
            let task_dialog_config = TASKDIALOGCONFIG {
                cbSize: core::mem::size_of::<TASKDIALOGCONFIG>() as u32,
                hwndParent: hwnd as _,
                dwFlags: TDF_ALLOW_DIALOG_CANCELLATION,
                pszWindowTitle: title.as_ptr(),
                pszContent: message.as_ptr(),
                Anonymous1: TASKDIALOGCONFIG_0 {
                    pszMainIcon: TD_INFORMATION_ICON,
                },
                Anonymous2: TASKDIALOGCONFIG_1 {
                    pszFooterIcon: std::ptr::null(),
                },
                dwCommonButtons: TDCBF_OK_BUTTON,
                pButtons: std::ptr::null(),
                cButtons: 0,
                pRadioButtons: std::ptr::null(),
                cRadioButtons: 0,
                cxWidth: 0,
                hInstance: std::ptr::null_mut(),
                pfCallback: None,
                lpCallbackData: 0,
                nDefaultButton: 0,
                nDefaultRadioButton: 0,
                pszCollapsedControlText: std::ptr::null(),
                pszExpandedControlText: std::ptr::null(),
                pszExpandedInformation: std::ptr::null(),
                pszMainInstruction: std::ptr::null(),
                pszVerificationText: std::ptr::null(),
                pszFooter: std::ptr::null(),
            };

            let mut pf_verification_flag_checked = 0;
            let mut pn_button = 0;
            let mut pn_radio_button = 0;

            TaskDialogIndirect(
                &task_dialog_config,
                &mut pn_button,
                &mut pn_radio_button,
                &mut pf_verification_flag_checked,
            )
        });
    }
}
