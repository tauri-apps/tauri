// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused)]
use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    dpi::{PhysicalPosition, Position},
    AboutMetadata, CheckMenuItem, ContextMenu, IconMenuItem, Menu, MenuEvent, MenuItem,
    PredefinedMenuItem, Submenu,
};
#[cfg(target_os = "macos")]
use tao::platform::macos::WindowExtMacOS;
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "windows")]
use tao::platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows};
use tao::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

enum UserEvent {
    MenuEvent(muda::MenuEvent),
}

fn main() {
    let mut event_loop_builder = EventLoopBuilder::<UserEvent>::with_user_event();

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

    let event_loop = event_loop_builder.build();

    // set a menu event handler that wakes up the event loop
    let proxy = event_loop.create_proxy();
    muda::MenuEvent::set_event_handler(Some(move |event| {
        proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let window = WindowBuilder::new()
        .with_title("Window 1")
        .build(&event_loop)
        .unwrap();
    let window2 = WindowBuilder::new()
        .with_title("Window 2")
        .build(&event_loop)
        .unwrap();

    #[cfg(target_os = "macos")]
    {
        let app_m = Submenu::new("App", true);
        menu_bar.append(&app_m);
        app_m.append_items(&[
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
    }

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
    let icon = load_icon(std::path::Path::new(path));

    let file_m = Submenu::new("&File", true);
    let edit_m = Submenu::new("&Edit", true);
    let window_m = Submenu::new("&Window", true);
    let help_m = Submenu::new("&Custom Help", true);

    window_m.set_icon(Some(icon.clone()));

    menu_bar.append_items(&[&file_m, &edit_m, &window_m, &help_m]);

    let custom_i_1 = MenuItem::with_id(
        "custom-i-1",
        "C&ustom 1",
        true,
        Some(Accelerator::new(Some(Modifiers::ALT), Code::KeyC)),
    );

    let image_item = IconMenuItem::with_id(
        "image-custom-1",
        "Image custom 1",
        true,
        Some(icon),
        Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyC)),
    );

    let check_custom_i_1 =
        CheckMenuItem::with_id("check-custom-1", "Check Custom 1", true, true, None);
    let check_custom_i_2 =
        CheckMenuItem::with_id("check-custom-2", "Check Custom 2", false, true, None);
    let check_custom_i_3 = CheckMenuItem::with_id(
        "check-custom-3",
        "Check Custom 3",
        true,
        true,
        Some(Accelerator::new(Some(Modifiers::SHIFT), Code::KeyD)),
    );

    let copy_i = PredefinedMenuItem::copy(None);
    let cut_i = PredefinedMenuItem::cut(None);
    let paste_i = PredefinedMenuItem::paste(None);

    file_m.append_items(&[
        &custom_i_1,
        &image_item,
        &window_m,
        &PredefinedMenuItem::separator(),
        &check_custom_i_1,
        &check_custom_i_2,
    ]);

    window_m.append_items(&[
        &PredefinedMenuItem::minimize(None),
        &PredefinedMenuItem::maximize(None),
        &PredefinedMenuItem::close_window(Some("Close")),
        &PredefinedMenuItem::fullscreen(None),
        &PredefinedMenuItem::bring_all_to_front(None),
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("tao".to_string()),
                version: Some("1.2.3".to_string()),
                copyright: Some("Copyright tao".to_string()),
                ..Default::default()
            }),
        ),
        &check_custom_i_3,
        &image_item,
        &custom_i_1,
    ]);

    help_m.append_items(&[&MenuItem::new("Supposed to show search", true, None)]);

    edit_m.append_items(&[&copy_i, &PredefinedMenuItem::separator(), &paste_i]);

    #[cfg(target_os = "windows")]
    unsafe {
        menu_bar.init_for_hwnd(window.hwnd() as _);
        menu_bar.init_for_hwnd(window2.hwnd() as _);
    }
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        // Pending a tao release on gtk4; muda's gtk API now takes gtk4 types.
        // menu_bar.init_for_gtk_window(window.gtk_window(), window.default_vbox());
        // menu_bar.init_for_gtk_window(window2.gtk_window(), window2.default_vbox());
        let _ = (&menu_bar, &window, &window2);
    }
    #[cfg(target_os = "macos")]
    {
        menu_bar.init_for_nsapp();
        window_m.set_as_windows_menu_for_nsapp();
        help_m.set_as_help_menu_for_nsapp();
    }

    let menu_channel = MenuEvent::receiver();
    let mut window_cursor_position = PhysicalPosition { x: 0., y: 0. };
    let mut use_window_pos = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                window_id,
                ..
            } => {
                window_cursor_position.x = position.x;
                window_cursor_position.y = position.y;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button: MouseButton::Right,
                        ..
                    },
                window_id,
                ..
            } => {
                show_context_menu(
                    if window_id == window.id() {
                        &window
                    } else {
                        &window2
                    },
                    &file_m,
                    if use_window_pos {
                        Some(window_cursor_position.into())
                    } else {
                        None
                    },
                );
                use_window_pos = !use_window_pos;
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }

            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if event.id == custom_i_1.id() {
                    file_m.insert(&MenuItem::new("New Menu Item", true, None), 2);
                }
                println!("{event:?}");
            }
            _ => (),
        }
    })
}

fn show_context_menu(window: &Window, menu: &dyn ContextMenu, position: Option<Position>) {
    println!("Show context menu at position {position:?}");
    #[cfg(target_os = "windows")]
    unsafe {
        menu.show_context_menu_for_hwnd(window.hwnd() as _, position);
    }
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        // Pending a tao release on gtk4; muda's gtk API now takes gtk4 types.
        // menu.show_context_menu_for_gtk_window(window.gtk_window().as_ref(), position);
        let _ = (window, menu, position);
    }
    #[cfg(target_os = "macos")]
    unsafe {
        menu.show_context_menu_for_nsview(window.ns_view() as _, position);
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
