// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused)]
use std::collections::HashMap;

use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    dpi::{PhysicalPosition, Position},
    AboutMetadata, CheckMenuItem, ContextMenu, IconMenuItem, Menu, MenuEvent, MenuItem,
    PredefinedMenuItem, Submenu,
};
#[cfg(target_os = "macos")]
use winit::platform::macos::{EventLoopBuilderExtMacOS, WindowExtMacOS};
#[cfg(target_os = "windows")]
use winit::platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows};
#[cfg(any(windows, target_os = "macos"))]
use winit::raw_window_handle::*;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, Event, MouseButton, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowAttributes, WindowId},
};

enum AppEvent {
    MenuEvent(muda::MenuEvent),
}

fn main() {
    let mut event_loop_builder = EventLoop::<AppEvent>::with_user_event();

    let menu_bar = Menu::new();

    // setup accelerator handler on Windows
    #[cfg(target_os = "windows")]
    {
        let menu_bar = menu_bar.clone();
        event_loop_builder.with_msg_hook(move |msg| {
            use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};
            unsafe {
                let msg = msg as *const MSG;
                let translated = TranslateAcceleratorW((*msg).hwnd, menu_bar.haccel() as _, msg);
                translated == 1
            }
        });
    }
    #[cfg(target_os = "macos")]
    event_loop_builder.with_default_menu(false);

    let event_loop = event_loop_builder.build().unwrap();

    // set a menu event handler that wakes up the event loop
    let proxy = event_loop.create_proxy();
    muda::MenuEvent::set_event_handler(Some(move |event| {
        proxy.send_event(AppEvent::MenuEvent(event));
    }));

    let mut app = App {
        app_menu: AppMenu::new(menu_bar),
        windows: HashMap::default(),
        cursor_position: PhysicalPosition::new(0., 0.),
        use_window_pos: false,
    };

    event_loop.run_app(&mut app).unwrap()
}

struct App {
    app_menu: AppMenu,
    windows: HashMap<WindowId, Window>,
    cursor_position: PhysicalPosition<f64>,
    use_window_pos: bool,
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {}

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            let window_attrs = WindowAttributes::default().with_title("Window 1");
            let window = event_loop.create_window(window_attrs).unwrap();

            let window_attrs2 = WindowAttributes::default().with_title("Window 2");
            let window2 = event_loop.create_window(window_attrs2).unwrap();

            #[cfg(target_os = "windows")]
            {
                use winit::raw_window_handle::*;
                if let RawWindowHandle::Win32(handle) = window.window_handle().unwrap().as_raw() {
                    unsafe { self.app_menu.menu_bar.init_for_hwnd(handle.hwnd.get()) };
                }
                if let RawWindowHandle::Win32(handle) = window2.window_handle().unwrap().as_raw() {
                    unsafe { self.app_menu.menu_bar.init_for_hwnd(handle.hwnd.get()) };
                }
            }
            #[cfg(target_os = "macos")]
            {
                self.app_menu.menu_bar.init_for_nsapp();
                self.app_menu.window_menu.set_as_windows_menu_for_nsapp();
                self.app_menu.custom_help_menu.set_as_help_menu_for_nsapp();
            }

            self.windows.insert(window.id(), window);
            self.windows.insert(window2.id(), window2);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = position;
            }

            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state: ElementState::Pressed,
                ..
            } => {
                show_context_menu(
                    self.windows.get(&window_id).unwrap(),
                    &self.app_menu.file_menu,
                    if self.use_window_pos {
                        Some(self.cursor_position.into())
                    } else {
                        None
                    },
                );
                self.use_window_pos = !self.use_window_pos;
            }

            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::MenuEvent(event) => {
                println!("{event:?}");

                if event.id == self.app_menu.custom_item.id() {
                    let new_item = MenuItem::new("New Menu Item", true, None);
                    self.app_menu.file_menu.insert(&new_item, 2);
                }
            }
        }
    }
}

struct AppMenu {
    menu_bar: Menu,
    file_menu: Submenu,
    edit_menu: Submenu,
    window_menu: Submenu,
    custom_help_menu: Submenu,
    custom_item: MenuItem,
}

impl AppMenu {
    fn new(menu_bar: Menu) -> Self {
        #[cfg(target_os = "macos")]
        {
            let app_menu = Submenu::new("App", true);
            app_menu.append_items(&[
                &PredefinedMenuItem::about(None, None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ]);
            menu_bar.append(&app_menu);
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
        let icon = load_icon(std::path::Path::new(path));

        let file_menu = Submenu::new("&File", true);
        let edit_menu = Submenu::new("&Edit", true);
        let window_menu = Submenu::new("&Window", true);

        window_menu.set_icon(Some(icon.clone()));

        menu_bar.append_items(&[&file_menu, &edit_menu, &window_menu]);

        let custom_i_1 = MenuItem::new(
            "C&ustom 1",
            true,
            Some(Accelerator::new(Some(Modifiers::ALT), Code::KeyC)),
        );

        let image_item = IconMenuItem::new("Image Custom 1", true, Some(icon), None);

        let check_custom_i_1 = CheckMenuItem::new("Check Custom 1", true, true, None);
        let check_custom_i_2 = CheckMenuItem::new("Check Custom 2", false, true, None);
        let check_custom_i_3 = CheckMenuItem::new(
            "Check Custom 3",
            true,
            true,
            Some(Accelerator::new(Some(Modifiers::SHIFT), Code::KeyD)),
        );

        let copy_i = PredefinedMenuItem::copy(None);
        let cut_i = PredefinedMenuItem::cut(None);
        let paste_i = PredefinedMenuItem::paste(None);

        file_menu.append_items(&[
            &custom_i_1,
            &image_item,
            &window_menu,
            &PredefinedMenuItem::separator(),
            &check_custom_i_1,
            &check_custom_i_2,
        ]);

        window_menu.append_items(&[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::close_window(Some("Close")),
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::bring_all_to_front(None),
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("winit".to_string()),
                    version: Some("1.2.3".to_string()),
                    copyright: Some("Copyright winit".to_string()),
                    ..Default::default()
                }),
            ),
            &check_custom_i_3,
            &image_item,
            &custom_i_1,
        ]);

        edit_menu.append_items(&[&copy_i, &PredefinedMenuItem::separator(), &paste_i]);

        let custom_help_menu = Submenu::new("Help", true);
        custom_help_menu.append_items(&[&MenuItem::new(
            "Supposed to show a search bar on macOS",
            true,
            None,
        )]);
        menu_bar.append(&custom_help_menu);

        Self {
            menu_bar,
            file_menu,
            edit_menu,
            window_menu,
            custom_help_menu,
            custom_item: custom_i_1,
        }
    }
}

fn show_context_menu(window: &Window, menu: &dyn ContextMenu, position: Option<Position>) {
    println!("Show context menu at position {position:?}");
    #[cfg(target_os = "windows")]
    {
        if let RawWindowHandle::Win32(handle) = window.window_handle().unwrap().as_raw() {
            unsafe { menu.show_context_menu_for_hwnd(handle.hwnd.get(), position) };
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let RawWindowHandle::AppKit(handle) = window.window_handle().unwrap().as_raw() {
            unsafe { menu.show_context_menu_for_nsview(handle.ns_view.as_ptr() as _, position) };
        }
    }
}

fn load_icon(path: &std::path::Path) -> muda::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    muda::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
