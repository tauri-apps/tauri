// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Off-screen rendering: a CEF browser with no native view of its own, whose
//! pixels the application receives and puts on screen itself.
//!
//! The runtime keeps the browser's IPC, protocols and lifecycle; the three
//! pieces here are the application's:
//!
//! - an [`Offscreen`] on the webview's runtime attributes, whose factory builds
//!   the [`RenderHandler`] CEF paints into ([`ExampleRenderHandler`]);
//! - [`on_window_event`], registered through `Cef::on_window_event`, through
//!   which the native window's redraws, pointer, keyboard and focus arrive:
//!   presented with `softbuffer`, forwarded to the browser host;
//! - `windowless_rendering_enabled` in the CEF settings, which `runtime_config`
//!   sets when `CEF_EXAMPLE_OFFSCREEN=on`. It is a process-wide setting CEF
//!   advises against when nothing renders off-screen, hence the switch.
//!
//! The native scene is a striped background drawn around and behind the page:
//! the webview's inset bounds show around it, and through it when the page is
//! transparent.
//!
//! Everything CEF and winit call here runs on the main thread: the runtime
//! drives CEF through its external message pump, so CEF's UI thread is the event
//! loop's. That is why the per-window state lives in a thread local, and why it
//! is never borrowed across a call into CEF, which may call the render handler
//! back synchronously.

use std::{
  cell::RefCell,
  collections::HashMap,
  num::NonZeroU32,
  sync::{
    Arc, Mutex,
    atomic::{AtomicU32, Ordering},
  },
};

use serde::Serialize;
use tauri::{
  AppHandle, LogicalPosition, LogicalSize, State, WebviewUrl, WebviewWindowBuilder,
  webview::{NewWindowResponse, WebviewBuilder},
  window::WindowBuilder,
};
use tauri_runtime_cef::{
  AsCefWebviewAttributes, Offscreen, OffscreenView,
  cef::{
    Browser, BrowserHost, CursorType, ImplBrowser, ImplBrowserHost, ImplRenderHandler,
    KeyEvent as CefKeyEvent, KeyEventType, MouseButtonType, MouseEvent, PaintElementType,
    Rect as CefRect, RenderHandler, ScreenInfo, WrapRenderHandler, rc::Rc as _,
    sys::cef_event_flags_t, wrap_render_handler,
  },
};
use winit::{
  cursor::CursorIcon,
  event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent},
  event_loop::ActiveEventLoop,
  keyboard::{Key, ModifiersState, NamedKey},
  window::Window,
};

use crate::runtime_config::RuntimeConfig;

/// The latest images CEF painted for one webview, shared with the commands the
/// page and the end-to-end suite read them back through.
#[derive(Default)]
pub struct Frames {
  view: Option<Frame>,
  popup: Option<Frame>,
  /// Where the page's popup widget (an open `<select>`) goes, in view DIPs.
  popup_rect: Option<CefRect>,
  paints: u64,
  popup_paints: u64,
}

/// One painted image: premultiplied BGRA, the `u32` value being `0xAARRGGBB`.
struct Frame {
  width: usize,
  height: usize,
  pixels: Vec<u32>,
}

/// Every off-screen window's frames, by window label.
#[derive(Default)]
pub struct Offscreens(Mutex<HashMap<String, Arc<Mutex<Frames>>>>);

/// The main-thread state of one off-screen window.
struct Embedded {
  view: OffscreenView,
  frames: Arc<Mutex<Frames>>,
  browser: Option<Browser>,
  /// Holds the native window, so it is dropped on `Destroyed`, before the
  /// runtime lets the window go.
  surface: Option<softbuffer::Surface<Arc<dyn Window>, Arc<dyn Window>>>,
  /// The last pointer position, in view DIPs, for `PointerLeft`.
  pointer: (i32, i32),
  modifiers: ModifiersState,
  buttons: u32,
}

thread_local! {
  static EMBEDDED: RefCell<HashMap<String, Embedded>> = RefCell::default();
}

wrap_render_handler! {
  struct ExampleRenderHandler {
    label: String,
    view: OffscreenView,
    frames: Arc<Mutex<Frames>>,
  }

  impl RenderHandler {
    fn view_rect(&self, browser: Option<&mut Browser>, rect: Option<&mut CefRect>) {
      self.remember(browser);
      let Some(rect) = rect else { return };
      let (_, (width, height)) = logical_bounds(&self.view);
      // CEF asserts on an empty view, which is what a minimized window has.
      *rect = CefRect {
        x: 0,
        y: 0,
        width: (width.round() as i32).max(1),
        height: (height.round() as i32).max(1),
      };
    }

    fn screen_info(
      &self,
      _browser: Option<&mut Browser>,
      screen_info: Option<&mut ScreenInfo>,
    ) -> std::os::raw::c_int {
      let (Some(info), Some(window)) = (screen_info, self.view.window()) else {
        return 0;
      };
      // What CEF scales the paint buffer by: the view rect is in DIPs, the
      // pixels it paints are physical.
      info.device_scale_factor = window.scale_factor() as f32;
      1
    }

    fn on_popup_show(&self, _browser: Option<&mut Browser>, show: std::os::raw::c_int) {
      if show == 0 {
        let mut frames = self.frames.lock().unwrap();
        frames.popup = None;
        frames.popup_rect = None;
        drop(frames);
        self.view.request_redraw();
      }
    }

    fn on_popup_size(&self, _browser: Option<&mut Browser>, rect: Option<&CefRect>) {
      self.frames.lock().unwrap().popup_rect = rect.cloned();
    }

    fn on_paint(
      &self,
      browser: Option<&mut Browser>,
      type_: PaintElementType,
      _dirty_rects: Option<&[CefRect]>,
      buffer: *const u8,
      width: std::os::raw::c_int,
      height: std::os::raw::c_int,
    ) {
      self.remember(browser);
      let (Ok(width), Ok(height)) = (usize::try_from(width), usize::try_from(height)) else {
        return;
      };
      if buffer.is_null() {
        return;
      }
      // CEF only lends the buffer for the duration of this call.
      let bytes = unsafe { std::slice::from_raw_parts(buffer, width * height * 4) };
      let frame = Frame {
        width,
        height,
        pixels: bytes
          .as_chunks::<4>()
          .0
          .iter()
          .map(|bgra| u32::from_le_bytes(*bgra))
          .collect(),
      };
      let mut frames = self.frames.lock().unwrap();
      if type_ == PaintElementType::POPUP {
        frames.popup_paints += 1;
        frames.popup = Some(frame);
      } else {
        frames.paints += 1;
        frames.view = Some(frame);
      }
      drop(frames);
      self.view.request_redraw();
    }
  }
}

impl ExampleRenderHandler {
  /// Keeps the browser the render handler is first called with, which is how
  /// the input forwarding below reaches its host.
  fn remember(&self, browser: Option<&mut Browser>) {
    let Some(browser) = browser else { return };
    EMBEDDED.with(|embedded| {
      if let Ok(mut embedded) = embedded.try_borrow_mut()
        && let Some(embedded) = embedded.get_mut(&self.label)
        && embedded.browser.is_none()
      {
        embedded.browser = Some(browser.clone());
      }
    });
  }
}

/// The webview's position and size relative to the window, in logical pixels.
fn logical_bounds(view: &OffscreenView) -> ((f64, f64), (f64, f64)) {
  let bounds = view.bounds();
  let position = bounds.position.to_logical::<f64>(1.0);
  let size = bounds.size.to_logical::<f64>(1.0);
  ((position.x, position.y), (size.width, size.height))
}

/// Opens a window whose single webview renders off-screen, inset from the
/// window's edges so the native scene shows around it.
#[tauri::command]
pub fn open_offscreen_window(
  app: AppHandle,
  config: State<'_, RuntimeConfig>,
  offscreens: State<'_, Offscreens>,
  transparent: bool,
) -> Result<String, String> {
  if !config.offscreen() {
    return Err(
      "windowless rendering is a process-wide CEF setting: relaunch with CEF_EXAMPLE_OFFSCREEN=on"
        .into(),
    );
  }

  static NEXT: AtomicU32 = AtomicU32::new(1);
  let label = format!("offscreen-{}", NEXT.fetch_add(1, Ordering::Relaxed));
  let (width, height, inset) = (820., 620., 32.);

  let window = WindowBuilder::new(&app, &label)
    .title("Off-screen rendering")
    .inner_size(width, height)
    .build()
    .map_err(|error| error.to_string())?;

  let frames = Arc::new(Mutex::new(Frames::default()));
  offscreens
    .0
    .lock()
    .unwrap()
    .insert(label.clone(), frames.clone());

  // Called on CEF's UI thread once the browser is being created, with the view
  // the runtime keeps up to date: bounds, visibility and the host window.
  let factory_label = label.clone();
  let mut offscreen = Offscreen::new(move |view| {
    EMBEDDED.with_borrow_mut(|embedded| {
      embedded.insert(
        factory_label.clone(),
        Embedded {
          view: view.clone(),
          frames: frames.clone(),
          browser: None,
          surface: None,
          pointer: (0, 0),
          modifiers: ModifiersState::empty(),
          buttons: 0,
        },
      )
    });
    ExampleRenderHandler::new(factory_label.clone(), view, frames.clone())
  });
  // Without a native view, Chromium has nowhere to set the cursor itself.
  let cursor_label = label.clone();
  offscreen.cursor_handler = Some(Arc::new(move |cursor, _| set_cursor(&cursor_label, cursor)));

  let query = if transparent { "?transparent" } else { "" };
  let popup_app = app.clone();
  let mut builder = WebviewBuilder::new(
    format!("{label}-page"),
    WebviewUrl::App(format!("offscreen.html{query}").into()),
  )
  .auto_resize()
  .position(LogicalPosition::new(inset, inset))
  .size(LogicalSize::new(width - 2. * inset, height - 2. * inset))
  .transparent(transparent)
  // A popup of an off-screen browser is a windowed one either way: answered
  // with a Tauri window, or left to CEF.
  .on_new_window(move |url, features| {
    if url.fragment() == Some("cef-owned") {
      return NewWindowResponse::Allow;
    }
    static NEXT_POPUP: AtomicU32 = AtomicU32::new(1);
    let label = format!(
      "popup-offscreen-{}",
      NEXT_POPUP.fetch_add(1, Ordering::Relaxed)
    );
    match WebviewWindowBuilder::new(&popup_app, label, WebviewUrl::External(url.clone()))
      .window_features(features)
      .title(url.as_str())
      .build()
    {
      Ok(window) => NewWindowResponse::Create { window },
      Err(_) => NewWindowResponse::Deny,
    }
  });

  match builder
    .runtime_specific_attributes_mut()
    .as_cef_webview_attributes_mut()
  {
    Some(attributes) => attributes.offscreen = Some(offscreen),
    None => return Err("not running on the CEF runtime".into()),
  }

  window
    .add_child(builder)
    .map_err(|error| error.to_string())?;
  Ok(label)
}

/// What CEF painted for an off-screen window so far.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OffscreenStats {
  paints: u64,
  popup_paints: u64,
  /// The last view image's size, in physical pixels.
  frame_size: Option<[usize; 2]>,
  /// Where the popup widget is shown, in view DIPs, while one is.
  popup_rect: Option<[i32; 4]>,
  /// The view pixel at the requested physical coordinates, as premultiplied
  /// `[r, g, b, a]`.
  pixel: Option<[u8; 4]>,
}

#[tauri::command]
pub fn offscreen_stats(
  offscreens: State<'_, Offscreens>,
  label: String,
  x: Option<usize>,
  y: Option<usize>,
) -> Result<OffscreenStats, String> {
  let frames = offscreens
    .0
    .lock()
    .unwrap()
    .get(&label)
    .cloned()
    .ok_or_else(|| format!("no off-screen window labeled {label}"))?;
  let frames = frames.lock().unwrap();
  let pixel = match (&frames.view, x, y) {
    (Some(frame), Some(x), Some(y)) if x < frame.width && y < frame.height => {
      let [b, g, r, a] = frame.pixels[y * frame.width + x].to_le_bytes();
      Some([r, g, b, a])
    }
    _ => None,
  };
  Ok(OffscreenStats {
    paints: frames.paints,
    popup_paints: frames.popup_paints,
    frame_size: frames
      .view
      .as_ref()
      .map(|frame| [frame.width, frame.height]),
    popup_rect: frames
      .popup_rect
      .as_ref()
      .map(|rect| [rect.x, rect.y, rect.width, rect.height]),
    pixel,
  })
}

/// The native window events of every window, registered with
/// `Cef::on_window_event`. It runs before the runtime handles the event, and
/// only observes: the runtime's own handling (closing, resizing the webview)
/// still happens.
pub fn on_window_event(
  label: &str,
  _event_loop: &dyn ActiveEventLoop,
  window: Arc<dyn Window>,
  event: &WindowEvent,
) {
  if let WindowEvent::Destroyed = event {
    // Drops the surface, and with it this module's last hold on the window.
    EMBEDDED.with_borrow_mut(|embedded| embedded.remove(label));
    return;
  }

  let Some((view, host)) = EMBEDDED.with_borrow(|embedded| {
    embedded.get(label).map(|embedded| {
      (
        embedded.view.clone(),
        embedded.browser.as_ref().and_then(|browser| browser.host()),
      )
    })
  }) else {
    return;
  };

  match event {
    WindowEvent::RedrawRequested => present(label, &window, &view),
    WindowEvent::Focused(focused) => {
      if let Some(host) = host {
        host.set_focus(i32::from(*focused));
      }
    }
    WindowEvent::ModifiersChanged(modifiers) => {
      with_embedded(label, |embedded| embedded.modifiers = modifiers.state());
    }
    WindowEvent::PointerMoved { position, .. } => {
      let Some(host) = host else { return };
      let event = mouse_event(label, &view, &window, position.x, position.y);
      host.send_mouse_move_event(Some(&event), 0);
    }
    WindowEvent::PointerLeft { .. } => {
      let Some(host) = host else { return };
      let event = with_embedded(label, |embedded| MouseEvent {
        x: embedded.pointer.0,
        y: embedded.pointer.1,
        modifiers: flags(embedded),
      });
      if let Some(event) = event {
        host.send_mouse_move_event(Some(&event), 1);
      }
    }
    WindowEvent::PointerButton {
      state,
      position,
      button,
      ..
    } => {
      let Some(host) = host else { return };
      let (kind, flag) = match button.clone().mouse_button() {
        Some(MouseButton::Left) => (MouseButtonType::LEFT, 16),
        Some(MouseButton::Middle) => (MouseButtonType::MIDDLE, 32),
        Some(MouseButton::Right) => (MouseButtonType::RIGHT, 64),
        _ => return,
      };
      let pressed = *state == ElementState::Pressed;
      with_embedded(label, |embedded| {
        if pressed {
          embedded.buttons |= flag;
        } else {
          embedded.buttons &= !flag;
        }
      });
      let event = mouse_event(label, &view, &window, position.x, position.y);
      host.send_mouse_click_event(Some(&event), kind, i32::from(!pressed), 1);
    }
    WindowEvent::MouseWheel { delta, .. } => {
      let Some(host) = host else { return };
      // Chromium scrolls by pixels; a wheel notch is 120 of them on Windows,
      // the convention CEF follows everywhere.
      let (dx, dy) = match delta {
        MouseScrollDelta::LineDelta(x, y) => (x * 120., y * 120.),
        MouseScrollDelta::PixelDelta(delta) => (delta.x as f32, delta.y as f32),
        _ => return,
      };
      // winit's convention is positive for content moving right and down, the
      // same as CEF's, but the GTK 4 backend passes GDK's deltas through
      // unchanged, and GDK's are positive for scrolling down.
      #[cfg(target_os = "linux")]
      let (dx, dy) = (-dx, -dy);
      let event = with_embedded(label, |embedded| MouseEvent {
        x: embedded.pointer.0,
        y: embedded.pointer.1,
        modifiers: flags(embedded),
      });
      if let Some(event) = event {
        host.send_mouse_wheel_event(Some(&event), dx as i32, dy as i32);
      }
    }
    WindowEvent::KeyboardInput { event, .. } => {
      let Some(host) = host else { return };
      let modifiers = with_embedded(label, |embedded| flags(embedded)).unwrap_or(0);
      let pressed = event.state == ElementState::Pressed;
      host.send_key_event(Some(&CefKeyEvent {
        type_: if pressed {
          KeyEventType::RAWKEYDOWN
        } else {
          KeyEventType::KEYUP
        },
        modifiers,
        windows_key_code: windows_key_code(&event.logical_key),
        ..Default::default()
      }));
      // Text goes in as character events. A shortcut is not text.
      let shortcut = modifiers & (CONTROL_DOWN | COMMAND_DOWN) != 0;
      if pressed
        && !shortcut
        && let Some(text) = &event.text
      {
        send_text(&host, text, modifiers);
      }
    }
    // What an input method composed. An application that wants the preedit
    // shown inline forwards it to `ime_set_composition` instead.
    WindowEvent::Ime(Ime::Commit(text)) => {
      if let Some(host) = host {
        send_text(&host, text, 0);
      }
    }
    _ => {}
  }
}

// `c_uint` everywhere but Windows, where bindgen makes it a `c_int`.
#[allow(clippy::unnecessary_cast)]
const SHIFT_DOWN: u32 = cef_event_flags_t::EVENTFLAG_SHIFT_DOWN.0 as u32;
#[allow(clippy::unnecessary_cast)]
const CONTROL_DOWN: u32 = cef_event_flags_t::EVENTFLAG_CONTROL_DOWN.0 as u32;
#[allow(clippy::unnecessary_cast)]
const ALT_DOWN: u32 = cef_event_flags_t::EVENTFLAG_ALT_DOWN.0 as u32;
#[allow(clippy::unnecessary_cast)]
const COMMAND_DOWN: u32 = cef_event_flags_t::EVENTFLAG_COMMAND_DOWN.0 as u32;

/// CEF's event flags for the held modifiers and mouse buttons.
fn flags(embedded: &Embedded) -> u32 {
  let mut flags = embedded.buttons;
  for (held, flag) in [
    (embedded.modifiers.shift_key(), SHIFT_DOWN),
    (embedded.modifiers.control_key(), CONTROL_DOWN),
    (embedded.modifiers.alt_key(), ALT_DOWN),
    (embedded.modifiers.meta_key(), COMMAND_DOWN),
  ] {
    if held {
      flags |= flag;
    }
  }
  flags
}

fn with_embedded<T>(label: &str, f: impl FnOnce(&mut Embedded) -> T) -> Option<T> {
  EMBEDDED.with_borrow_mut(|embedded| embedded.get_mut(label).map(f))
}

/// A mouse event at a physical window position, in the view's DIPs.
fn mouse_event(
  label: &str,
  view: &OffscreenView,
  window: &Arc<dyn Window>,
  x: f64,
  y: f64,
) -> MouseEvent {
  let scale = window.scale_factor();
  let ((left, top), _) = logical_bounds(view);
  let position = (
    (x / scale - left).round() as i32,
    (y / scale - top).round() as i32,
  );
  with_embedded(label, |embedded| {
    embedded.pointer = position;
    MouseEvent {
      x: position.0,
      y: position.1,
      modifiers: flags(embedded),
    }
  })
  .unwrap_or_default()
}

fn send_text(host: &BrowserHost, text: &str, modifiers: u32) {
  for unit in text.encode_utf16() {
    host.send_key_event(Some(&CefKeyEvent {
      type_: KeyEventType::CHAR,
      modifiers,
      windows_key_code: i32::from(unit),
      character: unit,
      unmodified_character: unit,
      ..Default::default()
    }));
  }
}

/// The Windows virtual-key code Chromium identifies a key by, for the keys an
/// editable field needs. Letters and digits are their uppercase ASCII code.
fn windows_key_code(key: &Key) -> i32 {
  match key {
    Key::Named(named) => match named {
      NamedKey::Backspace => 0x08,
      NamedKey::Tab => 0x09,
      NamedKey::Enter => 0x0D,
      NamedKey::Shift => 0x10,
      NamedKey::Control => 0x11,
      NamedKey::Alt => 0x12,
      NamedKey::Escape => 0x1B,
      NamedKey::PageUp => 0x21,
      NamedKey::PageDown => 0x22,
      NamedKey::End => 0x23,
      NamedKey::Home => 0x24,
      NamedKey::ArrowLeft => 0x25,
      NamedKey::ArrowUp => 0x26,
      NamedKey::ArrowRight => 0x27,
      NamedKey::ArrowDown => 0x28,
      NamedKey::Delete => 0x2E,
      _ => 0,
    },
    Key::Character(text) => text
      .chars()
      .next()
      .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
      .map_or(0, |c| c.to_ascii_uppercase() as i32),
    _ => 0,
  }
}

fn set_cursor(label: &str, cursor: CursorType) {
  let window = EMBEDDED.with_borrow(|embedded| embedded.get(label)?.view.window());
  let Some(window) = window else { return };
  let icon = if cursor == CursorType::HAND {
    CursorIcon::Pointer
  } else if cursor == CursorType::IBEAM {
    CursorIcon::Text
  } else {
    CursorIcon::Default
  };
  window.set_cursor(icon.into());
}

/// Draws the native scene, then the page and its popup widget over it.
fn present(label: &str, window: &Arc<dyn Window>, view: &OffscreenView) {
  let size = window.surface_size();
  let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
  else {
    return;
  };
  let Some(frames) = EMBEDDED.with_borrow(|embedded| Some(embedded.get(label)?.frames.clone()))
  else {
    return;
  };

  let mut surface = with_embedded(label, |embedded| embedded.surface.take()).flatten();
  if surface.is_none() {
    surface = softbuffer::Context::new(window.clone())
      .and_then(|context| softbuffer::Surface::new(&context, window.clone()))
      .ok();
  }
  let Some(mut surface) = surface else { return };
  if surface.resize(width, height).is_err() {
    return;
  }
  let Ok(mut buffer) = surface.buffer_mut() else {
    return;
  };

  let (width, height) = (size.width as usize, size.height as usize);
  for (index, pixel) in buffer.iter_mut().enumerate() {
    let (x, y) = (index % width, index / width);
    *pixel = if (x + y) / 24 % 2 == 0 {
      0x2b2f3a
    } else {
      0x3a4152
    };
  }

  if view.is_visible() {
    let scale = window.scale_factor();
    let ((left, top), _) = logical_bounds(view);
    let origin = (
      (left * scale).round() as isize,
      (top * scale).round() as isize,
    );
    let frames = frames.lock().unwrap();
    if let Some(frame) = &frames.view {
      blend(&mut buffer, width, height, frame, origin);
    }
    if let (Some(frame), Some(rect)) = (&frames.popup, &frames.popup_rect) {
      let popup_origin = (
        origin.0 + (f64::from(rect.x) * scale).round() as isize,
        origin.1 + (f64::from(rect.y) * scale).round() as isize,
      );
      blend(&mut buffer, width, height, frame, popup_origin);
    }
  }

  let _ = buffer.present();
  with_embedded(label, |embedded| embedded.surface = Some(surface));
}

/// Composites a premultiplied frame over the opaque native scene.
fn blend(buffer: &mut [u32], width: usize, height: usize, frame: &Frame, origin: (isize, isize)) {
  for row in 0..frame.height {
    let y = origin.1 + row as isize;
    if y < 0 || y as usize >= height {
      continue;
    }
    for column in 0..frame.width {
      let x = origin.0 + column as isize;
      if x < 0 || x as usize >= width {
        continue;
      }
      let source = frame.pixels[row * frame.width + column];
      let target = &mut buffer[y as usize * width + x as usize];
      let alpha = source >> 24;
      *target = match alpha {
        255 => source & 0x00ff_ffff,
        0 => *target,
        _ => {
          let channel = |shift: u32| {
            let s = (source >> shift) & 0xff;
            let d = (*target >> shift) & 0xff;
            (s + d * (255 - alpha) / 255).min(255) << shift
          };
          channel(16) | channel(8) | channel(0)
        }
      };
    }
  }
}
