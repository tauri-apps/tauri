// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::time::{Duration, Instant};

use cef::{
  CefString, CompositionUnderline, CompositionUnderlineStyle, ImplBrowserHost as _,
  KeyEvent as CefKeyEvent, KeyEventType, MouseButtonType, MouseEvent,
};
#[cfg(target_os = "macos")]
use winit::keyboard::{NativeKeyCode, PhysicalKey};
use winit::{
  dpi::{PhysicalPosition, PhysicalSize},
  event::{ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
  keyboard::{Key, ModifiersState, NamedKey},
  window::{ImeCapabilities, ImeEnableRequest, ImeRequest, ImeRequestData},
};

use crate::{
  OffscreenView,
  offscreen::{OffscreenBounds, OffscreenImeCursorArea},
  window::AppWindow,
};

#[derive(Default)]
pub(crate) struct OffscreenInputState {
  pub(crate) focused: Option<u32>,
  captured: Option<u32>,
  hovered: Option<u32>,
  last_click: Option<(Instant, MouseButtonType, f64, f64)>,
  click_count: i32,
  cursor_x: f64,
  cursor_y: f64,
  wheel_remainder: (f64, f64),
  wheel_target: Option<u32>,
  modifiers: ModifiersState,
  pressed_buttons: u32,
  ime_cursor: Option<usize>,
  ime_cursor_area: Option<OffscreenImeCursorArea>,
  applied_cursor: Option<(u32, u64)>,
  drag_consumed: bool,
  drag_target: Option<u32>,
  drag_paths: Vec<std::path::PathBuf>,
  #[cfg(windows)]
  ime_caret_created: bool,
}

pub(crate) fn enable_ime(appwindow: &AppWindow, surface: &OffscreenView) {
  let bounds = surface.physical_bounds();
  let request_data = ImeRequestData::default().with_cursor_area(
    PhysicalPosition::new(bounds.x, bounds.y).into(),
    PhysicalSize::new(1, 1).into(),
  );
  let capabilities = ImeCapabilities::new().with_cursor_area();

  if !appwindow
    .window
    .ime_capabilities()
    .is_some_and(|enabled| enabled.cursor_area())
  {
    if appwindow.window.ime_capabilities().is_some()
      && let Err(error) = appwindow.window.request_ime_update(ImeRequest::Disable)
    {
      log::warn!("failed to reset off-screen IME state: {error}");
    }
    let enable = ImeEnableRequest::new(capabilities, request_data)
      .expect("off-screen IME capabilities match the initial cursor area");
    if let Err(error) = appwindow
      .window
      .request_ime_update(ImeRequest::Enable(enable))
    {
      log::warn!("failed to enable off-screen IME input: {error}");
    }
  }

  #[cfg(windows)]
  appwindow.install_offscreen_ime_hook();
}

pub(crate) fn handle(
  appwindow: &mut AppWindow,
  event: &WindowEvent,
  response: crate::NativeEventResponse,
) {
  let consumed = response == crate::NativeEventResponse::Handled;
  if matches!(
    event,
    WindowEvent::DragEntered { .. }
      | WindowEvent::DragPosition { .. }
      | WindowEvent::DragDropped { .. }
  ) {
    appwindow.offscreen_input.drag_consumed = consumed;
  }
  if consumed
    && !matches!(
      event,
      WindowEvent::Focused(_) | WindowEvent::ModifiersChanged(_) | WindowEvent::Ime(Ime::Disabled)
    )
    && !(appwindow.offscreen_input.captured.is_some()
      && matches!(
        event,
        WindowEvent::PointerMoved { .. }
          | WindowEvent::PointerButton { .. }
          | WindowEvent::PointerLeft { .. }
      ))
  {
    if matches!(
      event,
      WindowEvent::PointerButton {
        state: ElementState::Pressed,
        ..
      }
    ) {
      set_focus(appwindow, None);
    }
    if matches!(
      event,
      WindowEvent::PointerMoved { .. } | WindowEvent::PointerEntered { .. }
    ) {
      if let Some(previous) = visible_children(appwindow)
        .find(|child| Some(child.webview_id) == appwindow.offscreen_input.hovered)
      {
        let mouse = mouse_event(
          appwindow,
          previous.offscreen.as_ref().unwrap(),
          mouse_position(appwindow),
        );
        previous.host.send_mouse_move_event(Some(&mouse), 1);
      }
      appwindow.offscreen_input.hovered = None;
      appwindow.offscreen_input.applied_cursor = None;
    }
    return;
  }
  if !appwindow
    .children
    .iter()
    .any(|child| child.offscreen.is_some())
  {
    return;
  }
  sync_ime_cursor_area(appwindow);
  match event {
    WindowEvent::ModifiersChanged(modifiers) => {
      appwindow.offscreen_input.modifiers = modifiers.state();
    }
    WindowEvent::Focused(focused) => {
      #[cfg(windows)]
      if *focused {
        appwindow.restore_offscreen_ime_context();
      } else {
        destroy_ime_caret(appwindow);
      }
      if let Some(child) = focused_child(appwindow) {
        child.host.set_focus(i32::from(*focused));
        if !focused {
          child.host.send_capture_lost_event();
          child.host.ime_cancel_composition();
        }
      }
      if !focused {
        appwindow.offscreen_input.pressed_buttons = 0;
        appwindow.offscreen_input.captured = None;
        appwindow.offscreen_input.modifiers = ModifiersState::empty();
        clear_ime_state(appwindow);
      }
    }
    WindowEvent::PointerMoved {
      position,
      primary: true,
      ..
    }
    | WindowEvent::PointerEntered {
      position,
      primary: true,
      ..
    } => {
      appwindow.offscreen_input.cursor_x = position.x;
      appwindow.offscreen_input.cursor_y = position.y;
      let target =
        mouse_target(appwindow, position.x, position.y).map(|(child, _)| child.webview_id);
      if appwindow.offscreen_input.hovered != target {
        if let Some(previous) = visible_children(appwindow)
          .find(|child| Some(child.webview_id) == appwindow.offscreen_input.hovered)
        {
          let event = mouse_event(
            appwindow,
            previous.offscreen.as_ref().unwrap(),
            (position.x, position.y),
          );
          previous.host.send_mouse_move_event(Some(&event), 1);
        }
        appwindow.offscreen_input.hovered = target;
        if target.is_none() {
          appwindow.window.set_cursor_visible(true);
          appwindow
            .window
            .set_cursor(winit::cursor::CursorIcon::Default.into());
        }
      }
      if let Some((child, event)) = mouse_target(appwindow, position.x, position.y) {
        child.host.send_mouse_move_event(Some(&event), 0);
      }
    }
    WindowEvent::PointerLeft { .. } => {
      if appwindow.offscreen_input.captured.is_some() {
        return;
      }
      let position = mouse_position(appwindow);
      for child in visible_children(appwindow) {
        let event = mouse_event(appwindow, child.offscreen.as_ref().unwrap(), position);
        child.host.send_mouse_move_event(Some(&event), 1);
      }
      appwindow.offscreen_input.hovered = None;
      appwindow.offscreen_input.applied_cursor = None;
    }
    WindowEvent::PointerButton {
      state,
      position,
      button,
      primary: true,
      ..
    } => {
      let Some(button) = button.clone().mouse_button().and_then(cef_mouse_button) else {
        return;
      };
      appwindow.offscreen_input.cursor_x = position.x;
      appwindow.offscreen_input.cursor_y = position.y;
      let target =
        mouse_target(appwindow, position.x, position.y).map(|(child, _)| child.webview_id);
      if state.is_pressed() {
        // A consumed native-region move may be the browser's last pointer event.
        // Reestablish its pointer position before a click enters the webview.
        if let Some((child, mouse)) = mouse_target(appwindow, position.x, position.y) {
          child.host.send_mouse_move_event(Some(&mouse), 0);
        }
        appwindow.offscreen_input.hovered = target;
        set_focus(appwindow, target);
        appwindow.offscreen_input.captured = target;
        let input = &mut appwindow.offscreen_input;
        let now = Instant::now();
        let (interval, distance) = double_click_settings(appwindow.window.scale_factor());
        let repeated = input.last_click.is_some_and(|(time, previous, x, y)| {
          previous == button
            && now.duration_since(time) <= interval
            && (position.x - x).abs() < distance
            && (position.y - y).abs() < distance
        });
        input.click_count = if repeated {
          input.click_count % 3 + 1
        } else {
          1
        };
        input.last_click = Some((now, button, position.x, position.y));
      }
      update_pressed_buttons(&mut appwindow.offscreen_input, button, *state);
      if let Some((child, mut event)) = mouse_target(appwindow, position.x, position.y) {
        // AppKit/GDK report the button that generated a release event in their
        // event modifiers. CEF's OSR examples preserve it; Windows uses wParam.
        if !cfg!(windows) && *state == ElementState::Released {
          event.modifiers |= mouse_button_flag(button);
        }
        child.host.send_mouse_click_event(
          Some(&event),
          button,
          i32::from(*state == ElementState::Released),
          appwindow.offscreen_input.click_count,
        );
      }
      if appwindow.offscreen_input.pressed_buttons == 0 {
        appwindow.offscreen_input.captured = None;
      }
    }
    WindowEvent::PinchGesture { delta, .. } => {
      let position = mouse_position(appwindow);
      if let Some((child, mut event)) = mouse_target(appwindow, position.0, position.1) {
        event.modifiers |= cef_flag_bits(cef::sys::cef_event_flags_t::EVENTFLAG_CONTROL_DOWN);
        child
          .host
          .send_mouse_wheel_event(Some(&event), 0, (delta * 100.0).round() as i32);
      }
    }
    WindowEvent::MouseWheel { delta, .. } => {
      let position = mouse_position(appwindow);
      let target =
        mouse_target(appwindow, position.0, position.1).map(|(child, _)| child.webview_id);
      if appwindow.offscreen_input.wheel_target != target {
        appwindow.offscreen_input.wheel_remainder = (0.0, 0.0);
        appwindow.offscreen_input.wheel_target = target;
      }
      if let Some((x, y, precise)) = wheel_delta(
        &mut appwindow.offscreen_input.wheel_remainder,
        *delta,
        appwindow.window.scale_factor(),
      ) && let Some((child, mut mouse)) = mouse_target(appwindow, position.0, position.1)
      {
        if precise {
          mouse.modifiers |=
            cef_flag_bits(cef::sys::cef_event_flags_t::EVENTFLAG_PRECISION_SCROLLING_DELTA);
        }
        child.host.send_mouse_wheel_event(Some(&mouse), x, y);
      }
    }
    WindowEvent::KeyboardInput { event, .. } => {
      let Some(child) = focused_child(appwindow) else {
        return;
      };
      let key_code = event_windows_key_code(event);
      let native_key_code = native_key_code(event, key_code);
      let modifiers = cef_modifiers(&appwindow.offscreen_input);
      #[cfg(windows)]
      let is_system_key = i32::from(appwindow.offscreen_input.modifiers.alt_key());
      #[cfg(not(windows))]
      let is_system_key = 0;
      #[cfg(target_os = "macos")]
      let (character, unmodified_character) = macos_key_characters(event);
      #[cfg(not(target_os = "macos"))]
      let (character, unmodified_character) = (0, 0);
      let type_ = if event.state.is_pressed() {
        KeyEventType::RAWKEYDOWN
      } else {
        KeyEventType::KEYUP
      };
      child.host.send_key_event(Some(&CefKeyEvent {
        type_,
        modifiers,
        windows_key_code: key_code,
        native_key_code,
        is_system_key,
        character,
        unmodified_character,
        ..Default::default()
      }));

      if event.state.is_pressed()
        && let Some(text) = &event.text
        && send_text(
          appwindow.offscreen_input.modifiers,
          appwindow.offscreen_input.ime_cursor.is_some(),
          text,
        )
      {
        for character in text.encode_utf16() {
          child.host.send_key_event(Some(&CefKeyEvent {
            type_: KeyEventType::CHAR,
            modifiers,
            windows_key_code: i32::from(character),
            native_key_code,
            is_system_key,
            character,
            unmodified_character: character,
            ..Default::default()
          }));
        }
      }
    }
    WindowEvent::Ime(ime) => match ime {
      Ime::Preedit(text, cursor) => {
        if text.is_empty() {
          if let Some(child) = focused_child(appwindow) {
            child.host.ime_cancel_composition();
          }
          clear_ime_state(appwindow);
          return;
        }
        let selection = cursor
          .map(|(start, end)| cef::Range {
            from: text.get(..start).unwrap_or(text).encode_utf16().count() as u32,
            to: text.get(..end).unwrap_or(text).encode_utf16().count() as u32,
          })
          .unwrap_or_else(|| {
            let end = text.encode_utf16().count() as u32;
            cef::Range { from: end, to: end }
          });
        appwindow.offscreen_input.ime_cursor = Some(selection.to as usize);
        appwindow.offscreen_input.ime_cursor_area = None;
        if let Some(child) = focused_child(appwindow) {
          let replacement = invalid_cef_range();
          let underlines = composition_underlines(text, Some(&selection));
          child.host.ime_set_composition(
            Some(&CefString::from(text.as_str())),
            Some(&underlines),
            Some(&replacement),
            Some(&selection),
          );
        }
        sync_ime_cursor_area(appwindow);
      }
      Ime::Commit(text) => {
        if let Some(child) = focused_child(appwindow) {
          let replacement = invalid_cef_range();
          child
            .host
            .ime_commit_text(Some(&CefString::from(text.as_str())), Some(&replacement), 0);
        }
        clear_ime_state(appwindow);
      }
      Ime::Disabled => {
        if let Some(child) = focused_child(appwindow) {
          child.host.ime_cancel_composition();
        }
        clear_ime_state(appwindow);
        #[cfg(windows)]
        destroy_ime_caret(appwindow);
      }
      #[cfg(windows)]
      Ime::Enabled if !appwindow.offscreen_input.ime_caret_created => {
        appwindow.offscreen_input.ime_caret_created = appwindow.create_offscreen_ime_caret();
      }
      _ => {}
    },
    _ => {}
  }
}

pub(crate) fn sync_ime_cursor_area(appwindow: &mut AppWindow) {
  let Some(cursor) = appwindow.offscreen_input.ime_cursor else {
    return;
  };
  let Some(area) = focused_child(appwindow)
    .and_then(|child| child.offscreen.as_ref())
    .and_then(|surface| surface.ime_cursor_area(cursor))
  else {
    return;
  };
  if appwindow.offscreen_input.ime_cursor_area == Some(area) {
    return;
  }

  let request = ImeRequestData::default().with_cursor_area(
    PhysicalPosition::new(area.x, area.y).into(),
    PhysicalSize::new(area.width, area.height).into(),
  );
  if let Err(error) = appwindow
    .window
    .request_ime_update(ImeRequest::Update(request))
  {
    log::warn!("failed to update off-screen IME cursor area: {error}");
    return;
  }

  #[cfg(windows)]
  if appwindow.offscreen_input.ime_caret_created {
    appwindow.position_offscreen_ime_caret(area.x, area.y, area.height);
  }
  appwindow.offscreen_input.ime_cursor_area = Some(area);
}

fn clear_ime_state(appwindow: &mut AppWindow) {
  appwindow.offscreen_input.ime_cursor = None;
  appwindow.offscreen_input.ime_cursor_area = None;
}

fn invalid_cef_range() -> cef::Range {
  // CEF's Windows OSR integration requires the concrete InvalidRange value;
  // passing a null range pointer causes Chromium to discard the composition.
  cef::Range {
    from: u32::MAX,
    to: u32::MAX,
  }
}

fn composition_underlines(text: &str, selection: Option<&cef::Range>) -> Vec<CompositionUnderline> {
  let length = text.encode_utf16().count() as u32;
  if length == 0 {
    return Vec::new();
  }

  let target = selection.and_then(|range| {
    let from = range.from.min(length);
    let to = range.to.min(length);
    (from < to).then_some((from, to))
  });
  let mut underlines = Vec::with_capacity(if target.is_some() { 3 } else { 1 });
  let mut push = |from, to, thick, style| {
    if from < to {
      underlines.push(CompositionUnderline {
        range: cef::Range { from, to },
        color: 0xff000000,
        background_color: 0x00000000,
        thick,
        style,
        ..Default::default()
      });
    }
  };

  if let Some((from, to)) = target {
    push(0, from, 0, CompositionUnderlineStyle::DOT);
    push(from, to, 1, CompositionUnderlineStyle::SOLID);
    push(to, length, 0, CompositionUnderlineStyle::DOT);
  } else {
    push(0, length, 0, CompositionUnderlineStyle::DOT);
  }
  underlines
}

#[cfg(windows)]
fn destroy_ime_caret(appwindow: &mut AppWindow) {
  if appwindow.offscreen_input.ime_caret_created {
    appwindow.destroy_offscreen_ime_caret();
    appwindow.offscreen_input.ime_caret_created = false;
  }
}

pub(crate) fn set_focus(appwindow: &mut AppWindow, target: Option<u32>) {
  let unchanged = appwindow.offscreen_input.focused == target;
  if unchanged
    && (target.is_none()
      || focused_child(appwindow)
        .and_then(|child| child.offscreen.as_ref())
        .is_some_and(|view| view.popup_bounds().is_some()))
  {
    // The popup is part of the focused browser. Refocusing its main widget would dismiss it.
    return;
  }
  if !unchanged {
    if let Some(child) = focused_child(appwindow) {
      child.host.set_focus(0);
      child.host.send_capture_lost_event();
      child.host.ime_cancel_composition();
    }
    clear_ime_state(appwindow);
    appwindow.offscreen_input.pressed_buttons = 0;
    appwindow.offscreen_input.captured = None;
    appwindow.offscreen_input.focused = target;
  }
  if let Some(child) = focused_child(appwindow) {
    child.host.set_focus(1);
  }
}

fn focused_child(appwindow: &AppWindow) -> Option<&crate::webview::AppWebview> {
  visible_children(appwindow)
    .find(|child| Some(child.webview_id) == appwindow.offscreen_input.focused)
}

fn visible_children(
  appwindow: &AppWindow,
) -> impl DoubleEndedIterator<Item = &crate::webview::AppWebview> {
  appwindow.children.iter().filter(|child| {
    child
      .offscreen
      .as_ref()
      .is_some_and(|surface| surface.is_visible())
  })
}

fn mouse_target(
  appwindow: &AppWindow,
  x: f64,
  y: f64,
) -> Option<(&crate::webview::AppWebview, MouseEvent)> {
  let target = pointer_target(
    appwindow.offscreen_input.captured,
    visible_children(appwindow).map(|child| {
      let view = child.offscreen.as_ref().unwrap();
      let bounds = view.physical_bounds();
      let in_popup = view.popup_bounds().is_some_and(|popup| {
        contains(
          OffscreenBounds {
            x: bounds.x + (popup.x as f64 * bounds.scale_factor).round() as i32,
            y: bounds.y + (popup.y as f64 * bounds.scale_factor).round() as i32,
            width: (popup.width as f64 * bounds.scale_factor).round().max(0.0) as u32,
            height: (popup.height as f64 * bounds.scale_factor).round().max(0.0) as u32,
            ..bounds
          },
          x,
          y,
        )
      });
      (child.webview_id, contains(bounds, x, y) || in_popup)
    }),
  )?;
  let child = visible_children(appwindow).find(|child| child.webview_id == target)?;
  Some((
    child,
    mouse_event(appwindow, child.offscreen.as_ref().unwrap(), (x, y)),
  ))
}

fn pointer_target(
  captured: Option<u32>,
  regions: impl DoubleEndedIterator<Item = (u32, bool)>,
) -> Option<u32> {
  let mut hit = None;
  for (id, contains) in regions.rev() {
    if captured == Some(id) {
      return Some(id);
    }
    if hit.is_none() && contains {
      hit = Some(id);
    }
  }
  hit
}

/// Native file-drop mode emits Tauri events. HTML drop mode forwards to CEF.
/// Returning false suppresses Tauri's file event when the target opted into HTML drops.
pub(crate) fn drag_drop(
  appwindow: &mut AppWindow,
  event: &tauri_runtime::window::DragDropEvent,
) -> bool {
  use cef::{ImplDragData, drag_data_create};
  use tauri_runtime::window::DragDropEvent;
  if !appwindow
    .children
    .iter()
    .any(|child| child.offscreen.is_some())
  {
    return true;
  }
  let position = match event {
    DragDropEvent::Enter { paths, position } => {
      appwindow.offscreen_input.drag_paths = paths.clone();
      Some(*position)
    }
    DragDropEvent::Over { position } | DragDropEvent::Drop { position, .. } => Some(*position),
    DragDropEvent::Leave => None,
    _ => return true,
  };
  let target = position.and_then(|position| {
    mouse_target(appwindow, position.x, position.y).map(|(child, _)| child.webview_id)
  });
  let emit_tauri = target
    .and_then(|id| {
      appwindow
        .children
        .iter()
        .find(|child| child.webview_id == id)
    })
    .is_none_or(|child| child.drag_drop_handler_enabled);
  let target = target.filter(|_| !emit_tauri && !appwindow.offscreen_input.drag_consumed);
  if appwindow.offscreen_input.drag_target != target {
    if let Some(previous) = appwindow
      .children
      .iter()
      .find(|child| Some(child.webview_id) == appwindow.offscreen_input.drag_target)
    {
      previous.host.drag_target_drag_leave();
    }
    appwindow.offscreen_input.drag_target = None;
    if let Some((child, mouse)) = target.zip(position).and_then(|(id, position)| {
      let child = appwindow
        .children
        .iter()
        .find(|child| child.webview_id == id)?;
      Some((
        child,
        mouse_event(
          appwindow,
          child.offscreen.as_ref()?,
          (position.x, position.y),
        ),
      ))
    }) && let Some(mut data) = drag_data_create()
    {
      for path in &appwindow.offscreen_input.drag_paths {
        data.add_file(
          Some(&CefString::from(path.to_string_lossy().as_ref())),
          None,
        );
      }
      child.host.drag_target_drag_enter(
        Some(&mut data),
        Some(&mouse),
        cef::sys::cef_drag_operations_mask_t::DRAG_OPERATION_COPY.into(),
      );
      appwindow.offscreen_input.drag_target = target;
    }
  }
  if let Some((child, mouse)) = target.zip(position).and_then(|(id, position)| {
    let child = appwindow
      .children
      .iter()
      .find(|child| child.webview_id == id)?;
    Some((
      child,
      mouse_event(
        appwindow,
        child.offscreen.as_ref()?,
        (position.x, position.y),
      ),
    ))
  }) {
    if matches!(event, DragDropEvent::Drop { .. }) {
      child.host.drag_target_drop(Some(&mouse));
    } else {
      child.host.drag_target_drag_over(
        Some(&mouse),
        cef::sys::cef_drag_operations_mask_t::DRAG_OPERATION_COPY.into(),
      );
    }
  }
  if matches!(event, DragDropEvent::Drop { .. } | DragDropEvent::Leave) {
    appwindow.offscreen_input.drag_target = None;
    appwindow.offscreen_input.drag_paths.clear();
  }
  emit_tauri
}

fn wheel_delta(
  remainder: &mut (f64, f64),
  delta: MouseScrollDelta,
  scale: f64,
) -> Option<(i32, i32, bool)> {
  let (x, y, precise) = match delta {
    MouseScrollDelta::PixelDelta(position) => (position.x / scale, position.y / scale, true),
    MouseScrollDelta::LineDelta(x, y) => {
      // CEF's Windows path uses wheel ticks; its GTK/macOS paths consume point deltas.
      let step = if cfg!(windows) { 120.0 } else { 40.0 };
      (x as f64 * step, y as f64 * step, !cfg!(windows))
    }
    _ => return None,
  };
  if !x.is_finite() || !y.is_finite() {
    return None;
  }
  remainder.0 += x;
  remainder.1 += y;
  let (x, y) = (remainder.0.trunc() as i32, remainder.1.trunc() as i32);
  remainder.0 -= x as f64;
  remainder.1 -= y as f64;
  (x != 0 || y != 0).then_some((x, y, precise))
}

fn double_click_settings(scale: f64) -> (Duration, f64) {
  #[cfg(target_os = "macos")]
  {
    (
      Duration::from_secs_f64(objc2_app_kit::NSEvent::doubleClickInterval()),
      4.0 * scale,
    )
  }
  #[cfg(windows)]
  {
    let _ = scale;
    use windows::Win32::UI::{
      Input::KeyboardAndMouse::GetDoubleClickTime,
      WindowsAndMessaging::{GetSystemMetrics, SM_CXDOUBLECLK},
    };
    // These thread-independent system metrics are already in physical pixels.
    unsafe {
      (
        Duration::from_millis(GetDoubleClickTime() as u64),
        (GetSystemMetrics(SM_CXDOUBLECLK).max(2) / 2) as f64,
      )
    }
  }
  #[cfg(not(any(target_os = "macos", windows)))]
  {
    let (time, distance) = gtk::Settings::default()
      .map(|settings| {
        (
          settings.gtk_double_click_time(),
          settings.gtk_double_click_distance(),
        )
      })
      .unwrap_or((250, 5));
    (
      Duration::from_millis(time.max(1) as u64),
      distance.max(1) as f64 * scale,
    )
  }
}

fn contains(bounds: OffscreenBounds, x: f64, y: f64) -> bool {
  x >= f64::from(bounds.x)
    && y >= f64::from(bounds.y)
    && x < f64::from(bounds.x) + f64::from(bounds.width)
    && y < f64::from(bounds.y) + f64::from(bounds.height)
}

fn mouse_position(appwindow: &AppWindow) -> (f64, f64) {
  (
    appwindow.offscreen_input.cursor_x,
    appwindow.offscreen_input.cursor_y,
  )
}

fn mouse_event(
  appwindow: &AppWindow,
  surface: &crate::OffscreenView,
  position: (f64, f64),
) -> MouseEvent {
  let bounds = surface.physical_bounds();
  MouseEvent {
    x: ((position.0 - f64::from(bounds.x)) / bounds.scale_factor).round() as i32,
    y: ((position.1 - f64::from(bounds.y)) / bounds.scale_factor).round() as i32,
    modifiers: cef_modifiers(&appwindow.offscreen_input),
  }
}

fn mouse_button_flag(button: MouseButtonType) -> u32 {
  use cef::sys::cef_event_flags_t as Flags;
  cef_flag_bits(if button == MouseButtonType::LEFT {
    Flags::EVENTFLAG_LEFT_MOUSE_BUTTON
  } else if button == MouseButtonType::MIDDLE {
    Flags::EVENTFLAG_MIDDLE_MOUSE_BUTTON
  } else {
    Flags::EVENTFLAG_RIGHT_MOUSE_BUTTON
  })
}

fn update_pressed_buttons(
  input: &mut OffscreenInputState,
  button: MouseButtonType,
  state: ElementState,
) {
  let flag = mouse_button_flag(button);
  if state.is_pressed() {
    input.pressed_buttons |= flag;
  } else {
    input.pressed_buttons &= !flag;
  }
}

fn cef_flag_bits(flag: cef::sys::cef_event_flags_t) -> u32 {
  // The CEF bindings use signed flags on Windows and unsigned flags elsewhere.
  u32::from_ne_bytes(flag.0.to_ne_bytes())
}

fn cef_modifiers(input: &OffscreenInputState) -> u32 {
  use cef::sys::cef_event_flags_t;
  let mut flags = input.pressed_buttons;
  if input.modifiers.shift_key() {
    flags |= cef_flag_bits(cef_event_flags_t::EVENTFLAG_SHIFT_DOWN);
  }
  if input.modifiers.control_key() {
    flags |= cef_flag_bits(cef_event_flags_t::EVENTFLAG_CONTROL_DOWN);
  }
  if input.modifiers.alt_key() {
    flags |= cef_flag_bits(cef_event_flags_t::EVENTFLAG_ALT_DOWN);
  }
  if input.modifiers.meta_key() {
    flags |= cef_flag_bits(cef_event_flags_t::EVENTFLAG_COMMAND_DOWN);
  }
  flags
}

fn cef_mouse_button(button: MouseButton) -> Option<MouseButtonType> {
  match button {
    MouseButton::Left => Some(MouseButtonType::LEFT),
    MouseButton::Middle => Some(MouseButtonType::MIDDLE),
    MouseButton::Right => Some(MouseButtonType::RIGHT),
    _ => None,
  }
}

fn send_text(modifiers: ModifiersState, composing: bool, text: &str) -> bool {
  !composing
    && !modifiers.meta_key()
    && (!modifiers.control_key() || (modifiers.alt_key() && text.chars().any(|c| !c.is_control())))
}

fn event_windows_key_code(event: &KeyEvent) -> i32 {
  #[cfg(windows)]
  {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
      GetKeyboardLayout, MAPVK_VSC_TO_VK_EX, MapVirtualKeyExW,
    };
    use winit::keyboard::{KeyCode, PhysicalKey};
    use winit::platform::scancode::PhysicalKeyExtScancode;

    // Scan-code translation always maps these keypad keys to navigation keys.
    // Winit's logical key already accounts for NumLock and Shift, including the
    // layout's decimal separator, so preserve numeric input before translating.
    if matches!(event.logical_key, Key::Character(_)) {
      let code = match event.physical_key {
        PhysicalKey::Code(KeyCode::Numpad0) => Some(0x60),
        PhysicalKey::Code(KeyCode::Numpad1) => Some(0x61),
        PhysicalKey::Code(KeyCode::Numpad2) => Some(0x62),
        PhysicalKey::Code(KeyCode::Numpad3) => Some(0x63),
        PhysicalKey::Code(KeyCode::Numpad4) => Some(0x64),
        PhysicalKey::Code(KeyCode::Numpad5) => Some(0x65),
        PhysicalKey::Code(KeyCode::Numpad6) => Some(0x66),
        PhysicalKey::Code(KeyCode::Numpad7) => Some(0x67),
        PhysicalKey::Code(KeyCode::Numpad8) => Some(0x68),
        PhysicalKey::Code(KeyCode::Numpad9) => Some(0x69),
        PhysicalKey::Code(KeyCode::NumpadDecimal) => Some(0x6e),
        _ => None,
      };
      if let Some(code) = code {
        return code;
      }
    }

    if let Some(scan) = event.physical_key.to_scancode() {
      // Translate using the active keyboard layout, including OEM and international keys.
      let code = unsafe { MapVirtualKeyExW(scan, MAPVK_VSC_TO_VK_EX, Some(GetKeyboardLayout(0))) };
      if code != 0 {
        return code as i32;
      }
    }
  }
  windows_key_code(&event.logical_key)
}

fn windows_key_code(key: &Key) -> i32 {
  match key {
    Key::Character(value) => match value.as_str() {
      " " => 0x20,
      ";" | ":" => 0xba,
      "=" | "+" => 0xbb,
      "," | "<" => 0xbc,
      "-" | "_" => 0xbd,
      "." | ">" => 0xbe,
      "/" | "?" => 0xbf,
      "`" | "~" => 0xc0,
      "[" | "{" => 0xdb,
      "\\" | "|" => 0xdc,
      "]" | "}" => 0xdd,
      "'" | "\"" => 0xde,
      _ => value
        .chars()
        .next()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase() as i32)
        .unwrap_or_default(),
    },
    Key::Named(key) => match key {
      NamedKey::Backspace => 0x08,
      NamedKey::Tab => 0x09,
      NamedKey::Enter => 0x0d,
      NamedKey::Shift => 0x10,
      NamedKey::Control => 0x11,
      NamedKey::Alt => 0x12,
      NamedKey::Escape => 0x1b,
      NamedKey::PageUp => 0x21,
      NamedKey::PageDown => 0x22,
      NamedKey::End => 0x23,
      NamedKey::Home => 0x24,
      NamedKey::ArrowLeft => 0x25,
      NamedKey::ArrowUp => 0x26,
      NamedKey::ArrowRight => 0x27,
      NamedKey::ArrowDown => 0x28,
      NamedKey::Delete => 0x2e,
      NamedKey::F1 => 0x70,
      NamedKey::F2 => 0x71,
      NamedKey::F3 => 0x72,
      NamedKey::F4 => 0x73,
      NamedKey::F5 => 0x74,
      NamedKey::F6 => 0x75,
      NamedKey::F7 => 0x76,
      NamedKey::F8 => 0x77,
      NamedKey::F9 => 0x78,
      NamedKey::F10 => 0x79,
      NamedKey::F11 => 0x7a,
      NamedKey::F12 => 0x7b,
      _ => 0,
    },
    _ => 0,
  }
}

#[cfg(not(target_os = "macos"))]
fn native_key_code(event: &KeyEvent, _windows_key_code: i32) -> i32 {
  #[cfg(windows)]
  use winit::platform::scancode::PhysicalKeyExtScancode;
  #[cfg(windows)]
  let scancode = event.physical_key.to_scancode().unwrap_or_default();
  #[cfg(not(windows))]
  let scancode = winit_common::xkb::physicalkey_to_scancode(event.physical_key).unwrap_or_default();
  #[cfg(windows)]
  {
    ((scancode & 0xff) << 16
      | if scancode & 0xff00 != 0 { 1 << 24 } else { 0 }
      | if event.state == ElementState::Released {
        3 << 30
      } else if event.repeat {
        1 << 30
      } else {
        0
      }
      | 1) as i32
  }
  #[cfg(not(windows))]
  {
    // Winit exposes evdev codes; CEF's X11 keycode converter expects the XKB +8 offset.
    scancode.saturating_add(8) as i32
  }
}

#[cfg(target_os = "macos")]
fn native_key_code(event: &KeyEvent, _windows_key_code: i32) -> i32 {
  macos_native_key_code(event.physical_key)
}

#[cfg(target_os = "macos")]
fn macos_native_key_code(physical_key: PhysicalKey) -> i32 {
  // CEF rebuilds an NSEvent from this value and derives the DOM key from it.
  match physical_key {
    PhysicalKey::Unidentified(NativeKeyCode::MacOS(key_code)) => i32::from(key_code),
    physical_key => winit::platform::scancode::PhysicalKeyExtScancode::to_scancode(physical_key)
      .map(|key_code| key_code as i32)
      .unwrap_or_default(),
  }
}

#[cfg(target_os = "macos")]
fn macos_key_characters(event: &KeyEvent) -> (u16, u16) {
  // CEF identifies AppKit modifier transitions by both characters being zero.
  // Preserve NSEvent text for every other key so it remains a key-down/up event.
  (
    first_utf16(event.text_with_all_modifiers.as_deref()),
    first_utf16(event.key_without_modifiers.to_text()),
  )
}

#[cfg(target_os = "macos")]
fn first_utf16(text: Option<&str>) -> u16 {
  text
    .and_then(|text| text.encode_utf16().next())
    .unwrap_or_default()
}

pub(crate) fn flush(
  appwindow: &mut AppWindow,
  event_loop: &dyn winit::event_loop::ActiveEventLoop,
) {
  use crate::offscreen::CursorUpdate;
  sync_ime_cursor_area(appwindow);
  let Some(child) = visible_children(appwindow)
    .find(|child| Some(child.webview_id) == appwindow.offscreen_input.hovered)
  else {
    return;
  };
  let view = child.offscreen.as_ref().unwrap();
  let mut state = view.state.lock().unwrap();
  let key = (child.webview_id, state.cursor_serial);
  if appwindow.offscreen_input.applied_cursor == Some(key) {
    return;
  }
  if let CursorUpdate::Image(source) = &state.cursor {
    match event_loop.create_custom_cursor(source.clone()) {
      Ok(cursor) => state.cursor = CursorUpdate::Ready(cursor.into()),
      Err(error) => {
        log::warn!("failed to create the browser cursor: {error}");
        state.cursor = CursorUpdate::Ready(winit::cursor::CursorIcon::Default.into());
      }
    }
  }
  match &state.cursor {
    CursorUpdate::Hidden => appwindow.window.set_cursor_visible(false),
    CursorUpdate::Ready(cursor) => {
      appwindow.window.set_cursor_visible(true);
      appwindow.window.set_cursor(cursor.clone());
    }
    CursorUpdate::Image(_) => unreachable!(),
  }
  drop(state);
  appwindow.offscreen_input.applied_cursor = Some(key);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn offscreen_precise_wheel_preserves_fractional_motion() {
    let mut remainder = (0.0, 0.0);
    let delta = MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, 0.5));
    for _ in 0..3 {
      assert_eq!(wheel_delta(&mut remainder, delta, 2.0), None);
    }
    assert_eq!(wheel_delta(&mut remainder, delta, 2.0), Some((0, 1, true)));
    assert_eq!(remainder, (0.0, 0.0));
  }

  #[test]
  fn offscreen_capture_keeps_the_original_view_outside_its_bounds() {
    assert_eq!(
      pointer_target(None, [(1, true), (2, true)].into_iter()),
      Some(2)
    );
    assert_eq!(
      pointer_target(Some(1), [(1, false), (2, true)].into_iter()),
      Some(1)
    );
    assert_eq!(
      pointer_target(None, [(1, false), (2, false)].into_iter()),
      None
    );
    assert_eq!(
      pointer_target(Some(3), [(1, true), (2, false)].into_iter()),
      Some(1)
    );
  }

  #[test]
  fn offscreen_text_respects_shortcuts_altgr_and_composition() {
    assert!(!send_text(ModifiersState::CONTROL, false, "c"));
    assert!(!send_text(ModifiersState::META, false, "v"));
    assert!(send_text(
      ModifiersState::CONTROL | ModifiersState::ALT,
      false,
      "€"
    ));
    assert!(!send_text(
      ModifiersState::CONTROL | ModifiersState::ALT,
      false,
      "\u{3}"
    ));
    assert!(!send_text(ModifiersState::empty(), true, "日本語"));
    assert!(send_text(ModifiersState::empty(), false, "日本語"));
  }

  #[test]
  fn offscreen_hit_testing_uses_parent_physical_coordinates() {
    let bounds = OffscreenBounds {
      x: 40,
      y: 60,
      width: 200,
      height: 100,
      scale_factor: 2.0,
    };
    assert!(contains(bounds, 40.0, 60.0));
    assert!(contains(bounds, 239.9, 159.9));
    assert!(!contains(bounds, 240.0, 60.0));
    assert!(!contains(bounds, 40.0, 160.0));
    assert!(!contains(bounds, 39.9, 60.0));
  }

  #[test]
  fn offscreen_punctuation_does_not_become_a_navigation_key() {
    assert_eq!(windows_key_code(&Key::Character(".".into())), 0xbe);
    assert_eq!(windows_key_code(&Key::Named(NamedKey::Delete)), 0x2e);
    assert_eq!(windows_key_code(&Key::Character("+".into())), 0xbb);
  }

  #[cfg(windows)]
  #[test]
  fn offscreen_windows_keypad_preserves_numeric_and_navigation_input() {
    use winit::keyboard::{KeyCode, KeyLocation, PhysicalKey};

    // Winit resolves NumLock and Shift into the logical key before dispatch.
    // The same physical key must remain numeric or navigational accordingly.
    for (physical, logical, expected) in [
      (KeyCode::Numpad0, Key::Character("0".into()), 0x60),
      (KeyCode::Numpad1, Key::Character("1".into()), 0x61),
      (KeyCode::Numpad2, Key::Character("2".into()), 0x62),
      (KeyCode::Numpad3, Key::Character("3".into()), 0x63),
      (KeyCode::Numpad4, Key::Character("4".into()), 0x64),
      (KeyCode::Numpad5, Key::Character("5".into()), 0x65),
      (KeyCode::Numpad6, Key::Character("6".into()), 0x66),
      (KeyCode::Numpad7, Key::Character("7".into()), 0x67),
      (KeyCode::Numpad8, Key::Character("8".into()), 0x68),
      (KeyCode::Numpad9, Key::Character("9".into()), 0x69),
      (KeyCode::NumpadDecimal, Key::Character(".".into()), 0x6e),
      (KeyCode::NumpadDecimal, Key::Character(",".into()), 0x6e),
      (KeyCode::Numpad0, Key::Named(NamedKey::Insert), 0x2d),
      (KeyCode::Numpad1, Key::Named(NamedKey::End), 0x23),
      (KeyCode::Numpad2, Key::Named(NamedKey::ArrowDown), 0x28),
      (KeyCode::Numpad3, Key::Named(NamedKey::PageDown), 0x22),
      (KeyCode::Numpad4, Key::Named(NamedKey::ArrowLeft), 0x25),
      (KeyCode::Numpad5, Key::Named(NamedKey::Clear), 0x0c),
      (KeyCode::Numpad6, Key::Named(NamedKey::ArrowRight), 0x27),
      (KeyCode::Numpad7, Key::Named(NamedKey::Home), 0x24),
      (KeyCode::Numpad8, Key::Named(NamedKey::ArrowUp), 0x26),
      (KeyCode::Numpad9, Key::Named(NamedKey::PageUp), 0x21),
      (KeyCode::NumpadDecimal, Key::Named(NamedKey::Delete), 0x2e),
    ] {
      for state in [ElementState::Pressed, ElementState::Released] {
        let event = KeyEvent {
          physical_key: PhysicalKey::Code(physical),
          logical_key: logical.clone(),
          text: None,
          location: KeyLocation::Numpad,
          state,
          repeat: false,
          text_with_all_modifiers: None,
          key_without_modifiers: logical.clone(),
        };
        assert_eq!(
          event_windows_key_code(&event),
          expected,
          "{physical:?} / {logical:?} / {state:?}"
        );
      }
    }
  }

  #[test]
  fn offscreen_mouse_modifiers_preserve_other_pressed_buttons() {
    let mut input = OffscreenInputState::default();
    update_pressed_buttons(&mut input, MouseButtonType::LEFT, ElementState::Pressed);
    update_pressed_buttons(&mut input, MouseButtonType::RIGHT, ElementState::Pressed);
    update_pressed_buttons(&mut input, MouseButtonType::LEFT, ElementState::Released);
    assert_eq!(
      cef_modifiers(&input),
      cef_flag_bits(cef::sys::cef_event_flags_t::EVENTFLAG_RIGHT_MOUSE_BUTTON)
    );
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn offscreen_linux_native_key_codes_include_xkb_offset() {
    use winit::keyboard::{KeyCode, KeyLocation, NativeKey, NativeKeyCode, PhysicalKey};

    for (physical_key, expected) in [
      (PhysicalKey::Code(KeyCode::KeyA), 38),
      (PhysicalKey::Code(KeyCode::ArrowLeft), 113),
      // Winit stores the evdev scancode even for unidentified XKB keys.
      (PhysicalKey::Unidentified(NativeKeyCode::Xkb(200)), 208),
    ] {
      let event = KeyEvent {
        physical_key,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        text: None,
        location: KeyLocation::Standard,
        state: ElementState::Pressed,
        repeat: false,
        text_with_all_modifiers: None,
        key_without_modifiers: Key::Unidentified(NativeKey::Unidentified),
      };
      assert_eq!(native_key_code(&event, 0), expected, "{physical_key:?}");
    }
  }

  #[cfg(target_os = "macos")]
  #[test]
  fn macos_native_key_codes_come_from_the_physical_key() {
    assert_eq!(
      macos_native_key_code(PhysicalKey::Code(winit::keyboard::KeyCode::KeyC)),
      0x08
    );
    assert_eq!(
      macos_native_key_code(PhysicalKey::Code(winit::keyboard::KeyCode::Backspace)),
      0x33
    );
  }

  #[cfg(target_os = "macos")]
  #[test]
  fn macos_character_values_distinguish_keys_from_modifier_changes() {
    assert_eq!(first_utf16(Some("c")), u16::from(b'c'));
    assert_eq!(first_utf16(Some("\x08")), u16::from(b'\x08'));
    assert_eq!(first_utf16(None), 0);
  }

  #[test]
  fn composition_uses_a_transparent_dotted_underline_by_default() {
    let underlines = composition_underlines("a😀b", None);

    assert_eq!(underlines.len(), 1);
    assert_eq!(underlines[0].range.from, 0);
    assert_eq!(underlines[0].range.to, 4);
    assert_eq!(underlines[0].background_color, 0);
    assert_eq!(underlines[0].thick, 0);
    assert_eq!(underlines[0].style, CompositionUnderlineStyle::DOT);
  }

  #[test]
  fn composition_emphasizes_only_the_ime_target_range() {
    let target = cef::Range { from: 1, to: 3 };
    let underlines = composition_underlines("a😀b", Some(&target));

    assert_eq!(underlines.len(), 3);
    assert_eq!((underlines[0].range.from, underlines[0].range.to), (0, 1));
    assert_eq!((underlines[1].range.from, underlines[1].range.to), (1, 3));
    assert_eq!(underlines[1].background_color, 0);
    assert_eq!(underlines[1].thick, 1);
    assert_eq!(underlines[1].style, CompositionUnderlineStyle::SOLID);
    assert_eq!((underlines[2].range.from, underlines[2].range.to), (3, 4));
  }
}
