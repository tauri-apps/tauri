// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use windows::core::{w, HSTRING};

enum Level {
  Error,
  #[allow(unused)]
  Warning,
  #[allow(unused)]
  Info,
}

pub fn error<S: AsRef<str>>(err: S) {
  dialog_inner(err.as_ref(), Level::Error);
}

fn dialog_inner(err: &str, level: Level) {
  let title = match level {
    Level::Warning => w!("Warning"),
    Level::Error => w!("Error"),
    Level::Info => w!("Info"),
  };

  #[cfg(not(feature = "common-controls-v6"))]
  {
    use windows::Win32::UI::WindowsAndMessaging::*;

    let err = remove_hyperlink(err);
    let err = HSTRING::from(err);

    unsafe {
      MessageBoxW(
        None,
        &err,
        title,
        match level {
          Level::Warning => MB_ICONWARNING,
          Level::Error => MB_ICONERROR,
          Level::Info => MB_ICONINFORMATION,
        },
      )
    };
  }

  #[cfg(feature = "common-controls-v6")]
  {
    use windows::core::{HRESULT, PCWSTR};
    use windows::Win32::Foundation::*;
    use windows::Win32::UI::Controls::*;
    use windows::Win32::UI::Shell::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    extern "system" fn task_dialog_callback(
      _hwnd: HWND,
      msg: TASKDIALOG_NOTIFICATIONS,
      _wparam: WPARAM,
      lparam: LPARAM,
      _data: isize,
    ) -> HRESULT {
      if msg == TDN_HYPERLINK_CLICKED {
        let link = PCWSTR(lparam.0 as _);
        let _ = unsafe { ShellExecuteW(None, None, link, None, None, SW_SHOWNORMAL) };
      }

      S_OK
    }

    let err = HSTRING::from(err);
    let err = PCWSTR(err.as_ptr());

    let task_dialog_config = TASKDIALOGCONFIG {
      cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
      dwFlags: TDF_ALLOW_DIALOG_CANCELLATION | TDF_ENABLE_HYPERLINKS,
      pszWindowTitle: title,
      pszContent: err,
      Anonymous1: TASKDIALOGCONFIG_0 {
        pszMainIcon: match level {
          Level::Warning => TD_WARNING_ICON,
          Level::Error => TD_ERROR_ICON,
          Level::Info => TD_INFORMATION_ICON,
        },
      },
      dwCommonButtons: TDCBF_OK_BUTTON,
      pfCallback: Some(task_dialog_callback),
      ..Default::default()
    };

    let _ = unsafe { TaskDialogIndirect(&task_dialog_config, None, None, None) };
  }
}

#[cfg(not(feature = "common-controls-v6"))]
fn remove_hyperlink(str: &str) -> String {
  let mut result = String::new();
  let mut in_hyperlink = false;

  for c in str.chars() {
    if c == '<' {
      in_hyperlink = true;
    } else if c == '>' {
      in_hyperlink = false;
    } else if !in_hyperlink {
      result.push(c);
    }
  }

  result
}

#[cfg(test)]
#[cfg(not(feature = "common-controls-v6"))]
mod tests {
  use super::*;

  #[test]
  fn test_remove_hyperlink() {
    let input = "This is a <A href=\"some link\">test</A> string.";
    let expected = "This is a test string.";
    let result = remove_hyperlink(input);
    assert_eq!(result, expected);
  }
}
