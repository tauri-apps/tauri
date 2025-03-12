use cef::{CefStringUtf16, CefStringUtf8};

pub fn utf16_string_to_utf8(s: CefStringUtf16) -> CefStringUtf8 {
  let value: *const cef_dll_sys::_cef_string_utf16_t = (&s).into();

  unsafe {
    let mut cef_string = std::mem::zeroed();

    if let Some((str_, length)) = value.as_ref().map(|value| (value.str_, value.length)) {
      cef_dll_sys::cef_string_utf16_to_utf8(str_, length, &mut cef_string);
    }

    cef_string
  }
  .into()
}
