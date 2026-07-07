// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused)]
use std::rc::Rc;

use muda::{
    accelerator::{Accelerator, Code, Key, KeyAccelerator, Modifiers},
    dpi::Position,
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
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Window, WindowBuilder},
};
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use wry::WebViewBuilderExtUnix;
use wry::{http::Request, WebViewBuilder};

enum UserEvent {
    MenuEvent(muda::MenuEvent),
}

fn main() -> wry::Result<()> {
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
        menu_bar.append(&app_m).unwrap();
        app_m
            .append_items(&[
                &PredefinedMenuItem::about(None, None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ])
            .unwrap();
    }

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
    let icon = load_icon(std::path::Path::new(path));

    let file_m = Submenu::new("&File", true);
    let edit_m = Submenu::new("&Edit", true);
    let window_m = Submenu::new("&Window", true);

    window_m.set_icon(Some(icon.clone()));

    menu_bar
        .append_items(&[&file_m, &edit_m, &window_m])
        .unwrap();

    let custom_i_1 = MenuItem::new("C&ustom 1", true, None);
    custom_i_1.set_key_accelerator(Some(KeyAccelerator::new(
        Some(Modifiers::CONTROL),
        Key::Character("+".into()),
    )));

    let image_item = IconMenuItem::new(
        "Image custom 1",
        true,
        Some(icon),
        Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyC)),
    );

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

    file_m
        .append_items(&[
            &custom_i_1,
            &image_item,
            &window_m,
            &PredefinedMenuItem::separator(),
            &check_custom_i_1,
            &check_custom_i_2,
        ])
        .unwrap();

    window_m
        .append_items(&[
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
        ])
        .unwrap();

    edit_m
        .append_items(&[
            &copy_i,
            &PredefinedMenuItem::separator(),
            &cut_i,
            &PredefinedMenuItem::separator(),
            &paste_i,
        ])
        .unwrap();

    #[cfg(target_os = "windows")]
    unsafe {
        menu_bar.init_for_hwnd(window.hwnd() as _).unwrap();
        menu_bar.init_for_hwnd(window2.hwnd() as _).unwrap();
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
        // menu_bar
        //     .init_for_gtk_window(window.gtk_window(), window.default_vbox())
        //     .unwrap();
        // menu_bar
        //     .init_for_gtk_window(window2.gtk_window(), window2.default_vbox())
        //     .unwrap();
        let _ = (&menu_bar, &window, &window2);
    }
    #[cfg(target_os = "macos")]
    {
        menu_bar.init_for_nsapp();
        window_m.set_as_windows_menu_for_nsapp();
    }

    #[cfg(windows)]
    let condition = "e.button !== 2";
    #[cfg(not(windows))]
    let condition = "e.button == 2 && e.buttons === 0";
    let html: String = format!(
        r#"
    <html>
    <body>
        <style>
            * {{
                padding: 0;
                margin: 0;
                box-sizing: border-box;
                font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
            }}
            main {{
                    width: 100vw;
                    height: 100vh;
            }}
            @media (prefers-color-scheme: dark) {{
                main {{
                    color: #fff;
                    background: #2f2f2f;
                }}
            }}
        </style>
        <main>
            <h4> WRYYYYYYYYYYYYYYYYYYYYYY! </h4>
        </main>
        <script>
            window.addEventListener('contextmenu', (e) => {{
                e.preventDefault();
                console.log(e)
                // contextmenu was requested from keyboard
                if ({condition}) {{
                    window.ipc.postMessage(`showContextMenuPos:${{e.clientX}},${{e.clientY}}`);
                }}
            }})
            let x = true;
            window.addEventListener('mouseup', (e) => {{
                if (e.button === 2) {{
                    if (x) {{
                        window.ipc.postMessage(`showContextMenuPos:${{e.clientX}},${{e.clientY}}`);
                    }} else {{
                        window.ipc.postMessage(`showContextMenu`);
                    }}
                    x = !x;
                }}
            }})
        </script>
    </body>
    </html>
  "#,
    );

    let window = Rc::new(window);
    let window2 = Rc::new(window2);

    let create_ipc_handler = |window: &Rc<Window>| {
        let window = window.clone();
        let file_m_c = file_m.clone();
        let menu_bar = menu_bar.clone();
        move |req: Request<String>| {
            let req = req.body();
            if req == "showContextMenu" {
                show_context_menu(&window, &file_m_c, None)
            } else if let Some(rest) = req.strip_prefix("showContextMenuPos:") {
                let (x, mut y) = rest
                    .split_once(',')
                    .map(|(x, y)| (x.parse::<i32>().unwrap(), y.parse::<i32>().unwrap()))
                    .unwrap();

                #[cfg(any(
                    target_os = "linux",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd"
                ))]
                {
                    // Pending a tao release on gtk4; muda's gtk API now takes gtk4 types.
                    // if let Some(menu_bar) = menu_bar
                    //     .clone()
                    //     .gtk_menubar_for_gtk_window(window.gtk_window())
                    // {
                    //     use gtk::prelude::*;
                    //     y += menu_bar.allocated_height();
                    // }
                    let _ = &mut y;
                }

                show_context_menu(&window, &file_m_c, Some(Position::Logical((x, y).into())))
            }
        }
    };

    fn create_webview(window: &Rc<Window>) -> WebViewBuilder<'_> {
        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        return WebViewBuilder::new(window);
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        WebViewBuilder::new_gtk(window.default_vbox().unwrap())
    };

    let webview = create_webview(&window)
        .with_html(&html)
        .with_ipc_handler(create_ipc_handler(&window))
        .build()?;
    let webview2 = create_webview(&window2)
        .with_html(html)
        .with_ipc_handler(create_ipc_handler(&window2))
        .build()?;

    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if event.id == custom_i_1.id() {
                    file_m.insert(&MenuItem::new("New Menu Item", true, None), 2);
                }
                println!("{event:?}");
            }
            _ => {}
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
