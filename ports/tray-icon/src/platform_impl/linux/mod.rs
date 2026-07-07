// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod icon;
mod menu;
mod tray;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use crossbeam_channel::{unbounded, Sender};
use once_cell::sync::Lazy;

pub(crate) use icon::PlatformIcon;
use tray::Tray;

use crate::{icon::Icon, TrayIconAttributes, TrayIconId};

pub struct TrayIcon {
    tray_handle: ksni::Handle<Tray>,
    shutdown: Arc<AtomicBool>,
}

static MENU_UPDATE_SUBSCRIBERS: Lazy<Mutex<Vec<Sender<()>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
static MENU_UPDATE_DISPATCHER: Lazy<()> = Lazy::new(|| {
    thread::spawn(|| {
        while muda::recv_menu_update().is_ok() {
            let Ok(mut subscribers) = MENU_UPDATE_SUBSCRIBERS.lock() else {
                continue;
            };
            subscribers.retain(|subscriber| subscriber.send(()).is_ok());
        }
    });
});

fn subscribe_menu_updates() -> crossbeam_channel::Receiver<()> {
    Lazy::force(&MENU_UPDATE_DISPATCHER);

    let (sender, receiver) = unbounded();
    if let Ok(mut subscribers) = MENU_UPDATE_SUBSCRIBERS.lock() {
        subscribers.push(sender);
    }
    receiver
}

impl TrayIcon {
    pub fn new(id: TrayIconId, attrs: TrayIconAttributes) -> crate::Result<Self> {
        let icon = attrs.icon.map(|icon| icon.inner.into());
        let title = attrs.title.unwrap_or_default();
        let tooltip = attrs.tooltip.unwrap_or_default();
        let menu = attrs
            .menu
            .as_ref()
            .map(|menu| menu.compat_items())
            .unwrap_or_default();

        let shutdown = Arc::new(AtomicBool::new(false));
        let tray_service = ksni::TrayService::new(Tray::new(id, icon, title, tooltip, menu));
        let tray_handle = tray_service.handle();
        tray_service.spawn();

        let update_tray_handle = tray_handle.clone();
        let update_shutdown = shutdown.clone();
        let update_receiver = subscribe_menu_updates();
        thread::spawn(move || {
            while update_receiver.recv().is_ok() {
                if update_shutdown.load(Ordering::Relaxed) {
                    break;
                }
                update_tray_handle.update(|_| {});
            }
        });

        Ok(Self {
            tray_handle,
            shutdown,
        })
    }

    pub fn set_icon(&mut self, icon: Option<Icon>) -> crate::Result<()> {
        let icon = icon.map(|icon| icon.inner.into());
        self.tray_handle.update(|tray| {
            tray.set_icon(icon);
        });
        Ok(())
    }

    pub fn set_menu(&mut self, menu: Option<Box<dyn crate::menu::ContextMenu>>) {
        let menu = menu
            .as_ref()
            .map(|menu| menu.compat_items())
            .unwrap_or_default();
        self.tray_handle.update(|tray| {
            tray.set_menu(menu);
        });
    }

    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) -> crate::Result<()> {
        let tooltip = tooltip
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or_default()
            .to_string();
        self.tray_handle.update(|tray| {
            tray.set_tooltip(tooltip);
        });
        Ok(())
    }

    pub fn set_title<S: AsRef<str>>(&mut self, title: Option<S>) {
        let title = title
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or_default()
            .to_string();
        self.tray_handle.update(|tray| {
            tray.set_title(title);
        });
    }

    pub fn set_visible(&mut self, visible: bool) -> crate::Result<()> {
        self.tray_handle.update(|tray| {
            if visible {
                tray.set_status(ksni::Status::Active);
            } else {
                tray.set_status(ksni::Status::Passive);
            }
        });
        Ok(())
    }

    pub fn set_temp_dir_path<P: AsRef<std::path::Path>>(&mut self, _path: Option<P>) {}

    pub fn rect(&self) -> Option<crate::Rect> {
        None
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        muda::send_menu_update();
    }
}
