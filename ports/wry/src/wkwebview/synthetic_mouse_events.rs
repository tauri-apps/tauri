use objc2_app_kit::{
  NSAlternateKeyMask, NSCommandKeyMask, NSControlKeyMask, NSEvent, NSEventType, NSShiftKeyMask,
  NSView,
};
use objc2_foundation::NSString;

use super::WryWebView;

pub(crate) fn other_mouse_down(this: &WryWebView, event: &NSEvent) {
  unsafe {
    if event.r#type() == NSEventType::OtherMouseDown {
      let button_number = event.buttonNumber();
      match button_number {
        // back button
        3 => {
          let js = create_js_mouse_event(this, event, true, true);
          this.evaluateJavaScript_completionHandler(&NSString::from_str(&js), None);
          return;
        }
        // forward button
        4 => {
          let js = create_js_mouse_event(this, event, true, false);
          this.evaluateJavaScript_completionHandler(&NSString::from_str(&js), None);
          return;
        }
        _ => {}
      }
    }

    this.mouseDown(event);
  }
}
pub(crate) fn other_mouse_up(this: &WryWebView, event: &NSEvent) {
  unsafe {
    if event.r#type() == NSEventType::OtherMouseUp {
      let button_number = event.buttonNumber();
      match button_number {
        // back button
        3 => {
          let js = create_js_mouse_event(this, event, false, true);
          this.evaluateJavaScript_completionHandler(&NSString::from_str(&js), None);
          return;
        }
        // forward button
        4 => {
          let js = create_js_mouse_event(this, event, false, false);
          this.evaluateJavaScript_completionHandler(&NSString::from_str(&js), None);
          return;
        }
        _ => {}
      }
    }

    this.mouseUp(event);
  }
}

unsafe fn create_js_mouse_event(
  view: &NSView,
  event: &NSEvent,
  down: bool,
  back_button: bool,
) -> String {
  let event_name = if down { "mousedown" } else { "mouseup" };
  // js equivalent https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button
  let button = if back_button { 3 } else { 4 };
  let mods_flags = event.modifierFlags();
  let window_point = event.locationInWindow();
  let view_point = view.convertPoint_fromView(window_point, None);
  let x = view_point.x as u32;
  let y = view_point.y as u32;
  // js equivalent https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/buttons
  let buttons = NSEvent::pressedMouseButtons();

  format!(
    r#"(() => {{
        const el = document.elementFromPoint({x},{y});
        const ev = new MouseEvent('{event_name}', {{
          view: window,
          button: {button},
          buttons: {buttons},
          x: {x},
          y: {y},
          bubbles: true,
          detail: {detail},
          cancelBubble: false,
          cancelable: true,
          clientX: {x},
          clientY: {y},
          composed: true,
          layerX: {x},
          layerY: {y},
          pageX: {x},
          pageY: {y},
          screenX: window.screenX + {x},
          screenY: window.screenY + {y},
          ctrlKey: {ctrl_key},
          metaKey: {meta_key},
          shiftKey: {shift_key},
          altKey: {alt_key},
        }});
        el.dispatchEvent(ev)
        if (!ev.defaultPrevented && "{event_name}" === "mouseup") {{
          if (ev.button === 3) {{
            window.history.back();
          }}
          if (ev.button === 4) {{
            window.history.forward();
          }}
        }}
      }})()"#,
    event_name = event_name,
    x = x,
    y = y,
    detail = event.clickCount(),
    ctrl_key = mods_flags.contains(NSControlKeyMask),
    alt_key = mods_flags.contains(NSAlternateKeyMask),
    shift_key = mods_flags.contains(NSShiftKeyMask),
    meta_key = mods_flags.contains(NSCommandKeyMask),
    button = button,
    buttons = buttons,
  )
}
