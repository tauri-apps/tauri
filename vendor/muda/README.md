### muda

Menu Utilities library for Desktop Applications.

[![Documentation](https://img.shields.io/docsrs/muda)](https://docs.rs/muda/latest/muda/)

## Platforms supported:

- Windows
- macOS
- Linux (gtk Only)
- FreeBSD (gtk Only)

## Platform-specific notes:

- On Windows, accelerators don't work unless the win32 message loop calls
  [`TranslateAcceleratorW`](https://docs.rs/windows-sys/latest/windows_sys/Win32/UI/WindowsAndMessaging/fn.TranslateAcceleratorW.html).
  See [`Menu::init_for_hwnd`](https://docs.rs/muda/latest/x86_64-pc-windows-msvc/muda/struct.Menu.html#method.init_for_hwnd) for more details

### Cargo Features

- `common-controls-v6`: Use `TaskDialogIndirect` API from `ComCtl32.dll` v6 on Windows for showing the predefined `About` menu item dialog.
- `libxdo`: Enables linking to `libxdo` on Linux or FreeBSD which is used for the predefined `Copy`, `Cut`, `Paste` and `SelectAll` menu item.
- `serde`: Enables de/serializing the dpi types.
- `gtk`: Enables the `gtk` crate dependency on Linux or FreeBSD. This is required for `muda` to function properly on Linux or FreeBSD.

## Dependencies (Linux Only)

`gtk` is used for menus and `libxdo` is used to make the predfined `Copy`, `Cut`, `Paste` and `SelectAll` menu items work. Be sure to install following packages before building:

#### Arch Linux / Manjaro:

```sh
pacman -S gtk3 xdotool
```

#### Debian / Ubuntu:

```sh
sudo apt install libgtk-3-dev libxdo-dev
```

## Dependencies in FreeBSD

Install this dependencies in order to compile `muda`. Instructions using `pkg`:

```sh
pkg install -y rust glib pkgconf gtk3
```

## Example

Create the menu and add your items

```rs
let menu = Menu::new();
let menu_item2 = MenuItem::new("Menu item #2", false, None);
let submenu = Submenu::with_items("Submenu Outer", true,&[
  &MenuItem::new("Menu item #1", true, Some(Accelerator::new(Some(Modifiers::ALT), Code::KeyD))),
  &PredefinedMenuItem::separator(),
  &menu_item2,
  &MenuItem::new("Menu item #3", true, None),
  &PredefinedMenuItem::separator(),
  &Submenu::with_items("Submenu Inner", true,&[
    &MenuItem::new("Submenu item #1", true, None),
    &PredefinedMenuItem::separator(),
    &menu_item2,
  ])
]);

```

Then add your root menu to a Window on Windows and Linux
or use it as your global app menu on macOS

```rs
// --snip--
#[cfg(target_os = "windows")]
unsafe { menu.init_for_hwnd(window.hwnd() as isize) };
#[cfg(target_os = "linux")]
menu.init_for_gtk_window(&gtk_window, Some(&vertical_gtk_box));
#[cfg(target_os = "macos")]
menu.init_for_nsapp();
```

## Context menus (Popup menus)

You can also use a [`Menu`] or a [`Submenu`] show a context menu.

```rs
// --snip--
let position = muda::PhysicalPosition { x: 100., y: 120. };
#[cfg(target_os = "windows")]
unsafe { menu.show_context_menu_for_hwnd(window.hwnd() as isize, Some(position.into())) };
#[cfg(target_os = "linux")]
menu.show_context_menu_for_gtk_window(&gtk_window, Some(position.into()));
#[cfg(target_os = "macos")]
unsafe { menu.show_context_menu_for_nsview(nsview, Some(position.into())) };
```

## Processing menu events

You can use `MenuEvent::receiver` to get a reference to the `MenuEventReceiver`
which you can use to listen to events when a menu item is activated

```rs
if let Ok(event) = MenuEvent::receiver().try_recv() {
    match event.id {
        _ if event.id == save_item.id() => {
            println!("Save menu item activated");
        },
        _ => {}
    }
}
```

### Note for [winit] or [tao] users:

You should use [`MenuEvent::set_event_handler`] and forward
the menu events to the event loop by using [`EventLoopProxy`]
so that the event loop is awakened on each menu event.

```rust
enum UserEvent {
  MenuEvent(muda::MenuEvent)
}

let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();

let proxy = event_loop.create_proxy();
muda::MenuEvent::set_event_handler(Some(move |event| {
    proxy.send_event(UserEvent::MenuEvent(event));
}));
```

[`EventLoopProxy`]: https://docs.rs/winit/latest/winit/event_loop/struct.EventLoopProxy.html
[winit]: https://docs.rs/winit
[tao]: https://docs.rs/tao

## License

Apache-2.0/MIT
