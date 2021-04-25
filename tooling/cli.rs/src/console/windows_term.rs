// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::cmp;
use std::env;
use std::ffi::OsStr;
use std::fmt::Display;
use std::io;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::slice;
use std::{char, mem::MaybeUninit};

use encode_unicode::error::InvalidUtf16Tuple;
use encode_unicode::CharExt;
#[cfg(feature = "windows-console-colors")]
use regex::Regex;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::MAX_PATH;
use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
use winapi::um::consoleapi::{GetNumberOfConsoleInputEvents, ReadConsoleInputW};
use winapi::um::fileapi::FILE_NAME_INFO;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::minwinbase::FileNameInfo;
use winapi::um::processenv::GetStdHandle;
use winapi::um::winbase::GetFileInformationByHandleEx;
use winapi::um::winbase::{STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
use winapi::um::wincon::{
  FillConsoleOutputAttribute, FillConsoleOutputCharacterA, GetConsoleCursorInfo,
  GetConsoleScreenBufferInfo, SetConsoleCursorInfo, SetConsoleCursorPosition, SetConsoleTitleW,
  CONSOLE_CURSOR_INFO, CONSOLE_SCREEN_BUFFER_INFO, COORD, INPUT_RECORD, KEY_EVENT,
  KEY_EVENT_RECORD,
};
use winapi::um::winnt::{CHAR, HANDLE, INT, WCHAR};
#[cfg(feature = "windows-console-colors")]
use winapi_util::console::{Color, Console, Intense};

use super::common_term;
use super::kb::Key;
use super::term::{Term, TermTarget};

#[cfg(feature = "windows-console-colors")]
lazy_static::lazy_static! {
    static ref INTENSE_COLOR_RE: Regex = Regex::new(r"\x1b\[(3|4)8;5;(8|9|1[0-5])m").unwrap();
    static ref NORMAL_COLOR_RE: Regex = Regex::new(r"\x1b\[(3|4)([0-7])m").unwrap();
    static ref ATTR_RE: Regex = Regex::new(r"\x1b\[([1-8])m").unwrap();
}

const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x4;
pub const DEFAULT_WIDTH: u16 = 79;

pub fn as_handle(term: &Term) -> HANDLE {
  // convert between winapi::um::winnt::HANDLE and std::os::windows::raw::HANDLE
  // which are both c_void. would be nice to find a better way to do this
  term.as_raw_handle() as HANDLE
}

pub fn is_a_terminal(out: &Term) -> bool {
  let (fd, others) = match out.target() {
    TermTarget::Stdout => (STD_OUTPUT_HANDLE, [STD_INPUT_HANDLE, STD_ERROR_HANDLE]),
    TermTarget::Stderr => (STD_ERROR_HANDLE, [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE]),
  };

  if unsafe { console_on_any(&[fd]) } {
    // False positives aren't possible. If we got a console then
    // we definitely have a tty on stdin.
    return true;
  }

  // At this point, we *could* have a false negative. We can determine that
  // this is true negative if we can detect the presence of a console on
  // any of the other streams. If another stream has a console, then we know
  // we're in a Windows console and can therefore trust the negative.
  if unsafe { console_on_any(&others) } {
    return false;
  }

  msys_tty_on(out)
}

pub fn is_a_color_terminal(out: &Term) -> bool {
  if !is_a_terminal(out) {
    return false;
  }
  if msys_tty_on(out) {
    return match env::var("TERM") {
      Ok(term) => term != "dumb",
      Err(_) => true,
    };
  }
  enable_ansi_on(out)
}

fn enable_ansi_on(out: &Term) -> bool {
  unsafe {
    let handle = as_handle(out);

    let mut dw_mode = 0;
    if GetConsoleMode(handle, &mut dw_mode) == 0 {
      return false;
    }

    dw_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    if SetConsoleMode(handle, dw_mode) == 0 {
      return false;
    }

    true
  }
}

unsafe fn console_on_any(fds: &[DWORD]) -> bool {
  for &fd in fds {
    let mut out = 0;
    let handle = GetStdHandle(fd);
    if GetConsoleMode(handle, &mut out) != 0 {
      return true;
    }
  }
  false
}

#[inline]
pub fn terminal_size(out: &Term) -> Option<(u16, u16)> {
  terminal_size::terminal_size_using_handle(out.as_raw_handle()).map(|x| ((x.1).0, (x.0).0))
}

pub fn move_cursor_to(out: &Term, x: usize, y: usize) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::move_cursor_to(out, x, y);
  }
  if let Some((hand, _)) = get_console_screen_buffer_info(as_handle(out)) {
    unsafe {
      SetConsoleCursorPosition(
        hand,
        COORD {
          X: x as i16,
          Y: y as i16,
        },
      );
    }
  }
  Ok(())
}

pub fn move_cursor_up(out: &Term, n: usize) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::move_cursor_up(out, n);
  }

  if let Some((_, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
    move_cursor_to(out, 0, csbi.dwCursorPosition.Y as usize - n)?;
  }
  Ok(())
}

pub fn move_cursor_down(out: &Term, n: usize) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::move_cursor_down(out, n);
  }

  if let Some((_, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
    move_cursor_to(out, 0, csbi.dwCursorPosition.Y as usize + n)?;
  }
  Ok(())
}

pub fn move_cursor_left(out: &Term, n: usize) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::move_cursor_left(out, n);
  }

  if let Some((_, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
    move_cursor_to(
      out,
      csbi.dwCursorPosition.X as usize - n,
      csbi.dwCursorPosition.Y as usize,
    )?;
  }
  Ok(())
}

pub fn move_cursor_right(out: &Term, n: usize) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::move_cursor_right(out, n);
  }

  if let Some((_, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
    move_cursor_to(
      out,
      csbi.dwCursorPosition.X as usize + n,
      csbi.dwCursorPosition.Y as usize,
    )?;
  }
  Ok(())
}

pub fn clear_line(out: &Term) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::clear_line(out);
  }
  if let Some((hand, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
    unsafe {
      let width = csbi.srWindow.Right - csbi.srWindow.Left;
      let pos = COORD {
        X: 0,
        Y: csbi.dwCursorPosition.Y,
      };
      let mut written = 0;
      FillConsoleOutputCharacterA(hand, b' ' as CHAR, width as DWORD, pos, &mut written);
      FillConsoleOutputAttribute(hand, csbi.wAttributes, width as DWORD, pos, &mut written);
      SetConsoleCursorPosition(hand, pos);
    }
  }
  Ok(())
}

pub fn clear_chars(out: &Term, n: usize) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::clear_chars(out, n);
  }
  if let Some((hand, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
    unsafe {
      let width = cmp::min(csbi.dwCursorPosition.X, n as i16);
      let pos = COORD {
        X: csbi.dwCursorPosition.X - width,
        Y: csbi.dwCursorPosition.Y,
      };
      let mut written = 0;
      FillConsoleOutputCharacterA(hand, b' ' as CHAR, width as DWORD, pos, &mut written);
      FillConsoleOutputAttribute(hand, csbi.wAttributes, width as DWORD, pos, &mut written);
      SetConsoleCursorPosition(hand, pos);
    }
  }
  Ok(())
}

pub fn clear_screen(out: &Term) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::clear_screen(out);
  }
  if let Some((hand, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
    unsafe {
      let cells = csbi.dwSize.X as DWORD * csbi.dwSize.Y as DWORD; // as DWORD, or else this causes stack overflows.
      let pos = COORD { X: 0, Y: 0 };
      let mut written = 0;
      FillConsoleOutputCharacterA(hand, b' ' as CHAR, cells, pos, &mut written); // cells as DWORD no longer needed.
      FillConsoleOutputAttribute(hand, csbi.wAttributes, cells, pos, &mut written);
      SetConsoleCursorPosition(hand, pos);
    }
  }
  Ok(())
}

pub fn clear_to_end_of_screen(out: &Term) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::clear_to_end_of_screen(out);
  }
  if let Some((hand, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
    unsafe {
      let bottom = csbi.srWindow.Right as DWORD * csbi.srWindow.Bottom as DWORD;
      let cells = bottom - (csbi.dwCursorPosition.X as DWORD * csbi.dwCursorPosition.Y as DWORD); // as DWORD, or else this causes stack overflows.
      let pos = COORD {
        X: 0,
        Y: csbi.dwCursorPosition.Y,
      };
      let mut written = 0;
      FillConsoleOutputCharacterA(hand, b' ' as CHAR, cells, pos, &mut written); // cells as DWORD no longer needed.
      FillConsoleOutputAttribute(hand, csbi.wAttributes, cells, pos, &mut written);
      SetConsoleCursorPosition(hand, pos);
    }
  }
  Ok(())
}

pub fn show_cursor(out: &Term) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::show_cursor(out);
  }
  if let Some((hand, mut cci)) = get_console_cursor_info(as_handle(out)) {
    unsafe {
      cci.bVisible = 1;
      SetConsoleCursorInfo(hand, &cci);
    }
  }
  Ok(())
}

pub fn hide_cursor(out: &Term) -> io::Result<()> {
  if out.is_msys_tty {
    return common_term::hide_cursor(out);
  }
  if let Some((hand, mut cci)) = get_console_cursor_info(as_handle(out)) {
    unsafe {
      cci.bVisible = 0;
      SetConsoleCursorInfo(hand, &cci);
    }
  }
  Ok(())
}

fn get_console_screen_buffer_info(hand: HANDLE) -> Option<(HANDLE, CONSOLE_SCREEN_BUFFER_INFO)> {
  let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = unsafe { mem::zeroed() };
  match unsafe { GetConsoleScreenBufferInfo(hand, &mut csbi) } {
    0 => None,
    _ => Some((hand, csbi)),
  }
}

fn get_console_cursor_info(hand: HANDLE) -> Option<(HANDLE, CONSOLE_CURSOR_INFO)> {
  let mut cci: CONSOLE_CURSOR_INFO = unsafe { mem::zeroed() };
  match unsafe { GetConsoleCursorInfo(hand, &mut cci) } {
    0 => None,
    _ => Some((hand, cci)),
  }
}

pub fn key_from_key_code(code: INT) -> Key {
  match code {
    winapi::um::winuser::VK_LEFT => Key::ArrowLeft,
    winapi::um::winuser::VK_RIGHT => Key::ArrowRight,
    winapi::um::winuser::VK_UP => Key::ArrowUp,
    winapi::um::winuser::VK_DOWN => Key::ArrowDown,
    winapi::um::winuser::VK_RETURN => Key::Enter,
    winapi::um::winuser::VK_ESCAPE => Key::Escape,
    winapi::um::winuser::VK_BACK => Key::Backspace,
    winapi::um::winuser::VK_TAB => Key::Tab,
    winapi::um::winuser::VK_HOME => Key::Home,
    winapi::um::winuser::VK_END => Key::End,
    winapi::um::winuser::VK_DELETE => Key::Del,
    winapi::um::winuser::VK_SHIFT => Key::Shift,
    _ => Key::Unknown,
  }
}

pub fn read_secure() -> io::Result<String> {
  let mut rv = String::new();
  loop {
    match read_single_key()? {
      Key::Enter => {
        break;
      }
      Key::Char('\x08') => {
        if !rv.is_empty() {
          let new_len = rv.len() - 1;
          rv.truncate(new_len);
        }
      }
      Key::Char(c) => {
        rv.push(c);
      }
      _ => {}
    }
  }
  Ok(rv)
}

pub fn read_single_key() -> io::Result<Key> {
  let key_event = read_key_event()?;

  let unicode_char = unsafe { *key_event.uChar.UnicodeChar() };
  if unicode_char == 0 {
    Ok(key_from_key_code(key_event.wVirtualKeyCode as INT))
  } else {
    // This is a unicode character, in utf-16. Try to decode it by itself.
    match char::from_utf16_tuple((unicode_char, None)) {
      Ok(c) => {
        // Maintain backward compatibility. The previous implementation (_getwch()) would return
        // a special keycode for `Enter`, while ReadConsoleInputW() prefers to use '\r'.
        if c == '\r' {
          Ok(Key::Enter)
        } else if c == '\x08' {
          Ok(Key::Backspace)
        } else if c == '\x1B' {
          Ok(Key::Escape)
        } else {
          Ok(Key::Char(c))
        }
      }
      // This is part of a surrogate pair. Try to read the second half.
      Err(InvalidUtf16Tuple::MissingSecond) => {
        // Confirm that there is a next character to read.
        if get_key_event_count()? == 0 {
          let message = format!(
            "Read invlid utf16 {}: {}",
            unicode_char,
            InvalidUtf16Tuple::MissingSecond
          );
          return Err(io::Error::new(io::ErrorKind::InvalidData, message));
        }

        // Read the next character.
        let next_event = read_key_event()?;
        let next_surrogate = unsafe { *next_event.uChar.UnicodeChar() };

        // Attempt to decode it.
        match char::from_utf16_tuple((unicode_char, Some(next_surrogate))) {
          Ok(c) => Ok(Key::Char(c)),

          // Return an InvalidData error. This is the recommended value for UTF-related I/O errors.
          // (This error is given when reading a non-UTF8 file into a String, for example.)
          Err(e) => {
            let message = format!(
              "Read invalid surrogate pair ({}, {}): {}",
              unicode_char, next_surrogate, e
            );
            Err(io::Error::new(io::ErrorKind::InvalidData, message))
          }
        }
      }

      // Return an InvalidData error. This is the recommended value for UTF-related I/O errors.
      // (This error is given when reading a non-UTF8 file into a String, for example.)
      Err(e) => {
        let message = format!("Read invalid utf16 {}: {}", unicode_char, e);
        Err(io::Error::new(io::ErrorKind::InvalidData, message))
      }
    }
  }
}

fn get_stdin_handle() -> io::Result<HANDLE> {
  let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
  if handle == INVALID_HANDLE_VALUE {
    Err(io::Error::last_os_error())
  } else {
    Ok(handle)
  }
}

/// Get the number of pending events in the ReadConsoleInput queue. Note that while
/// these aren't necessarily key events, the only way that multiple events can be
/// put into the queue simultaneously is if a unicode character spanning multiple u16's
/// is read.
///
/// Therefore, this is accurate as long as at least one KEY_EVENT has already been read.
fn get_key_event_count() -> io::Result<DWORD> {
  let handle = get_stdin_handle()?;
  let mut event_count: DWORD = unsafe { mem::zeroed() };

  let success = unsafe { GetNumberOfConsoleInputEvents(handle, &mut event_count) };
  if success == 0 {
    Err(io::Error::last_os_error())
  } else {
    Ok(event_count)
  }
}

fn read_key_event() -> io::Result<KEY_EVENT_RECORD> {
  let handle = get_stdin_handle()?;
  let mut buffer: INPUT_RECORD = unsafe { mem::zeroed() };

  let mut events_read: DWORD = unsafe { mem::zeroed() };

  let mut key_event: KEY_EVENT_RECORD;
  loop {
    let success = unsafe { ReadConsoleInputW(handle, &mut buffer, 1, &mut events_read) };
    if success == 0 {
      return Err(io::Error::last_os_error());
    }
    if events_read == 0 {
      return Err(io::Error::new(
        io::ErrorKind::Other,
        "ReadConsoleInput returned no events, instead of waiting for an event",
      ));
    }

    if events_read == 1 && buffer.EventType != KEY_EVENT {
      // This isn't a key event; ignore it.
      continue;
    }

    key_event = unsafe { mem::transmute(buffer.Event) };

    if key_event.bKeyDown == 0 {
      // This is a key being released; ignore it.
      continue;
    }

    return Ok(key_event);
  }
}

pub fn wants_emoji() -> bool {
  // If WT_SESSION is set, we can assume we're running in the nne
  // Windows Terminal.  The correct way to detect this is not available
  // yet.  See https://github.com/microsoft/terminal/issues/1040
  env::var("WT_SESSION").is_ok()
}

/// Returns true if there is an MSYS tty on the given handle.
pub fn msys_tty_on(term: &Term) -> bool {
  let handle = term.as_raw_handle();
  unsafe {
    // Check whether the Windows 10 native pty is enabled
    {
      let mut out = MaybeUninit::uninit();
      let res = GetConsoleMode(handle as *mut _, out.as_mut_ptr());
      if res != 0 // If res is true then out was initialized.
                && (out.assume_init() & ENABLE_VIRTUAL_TERMINAL_PROCESSING)
                    == ENABLE_VIRTUAL_TERMINAL_PROCESSING
      {
        return true;
      }
    }

    let size = mem::size_of::<FILE_NAME_INFO>();
    let mut name_info_bytes = vec![0u8; size + MAX_PATH * mem::size_of::<WCHAR>()];
    let res = GetFileInformationByHandleEx(
      handle as *mut _,
      FileNameInfo,
      &mut *name_info_bytes as *mut _ as *mut c_void,
      name_info_bytes.len() as u32,
    );
    if res == 0 {
      return false;
    }
    let name_info: &FILE_NAME_INFO = &*(name_info_bytes.as_ptr() as *const FILE_NAME_INFO);
    let s = slice::from_raw_parts(
      name_info.FileName.as_ptr(),
      name_info.FileNameLength as usize / 2,
    );
    let name = String::from_utf16_lossy(s);
    // This checks whether 'pty' exists in the file name, which indicates that
    // a pseudo-terminal is attached. To mitigate against false positives
    // (e.g., an actual file name that contains 'pty'), we also require that
    // either the strings 'msys-' or 'cygwin-' are in the file name as well.)
    let is_msys = name.contains("msys-") || name.contains("cygwin-");
    let is_pty = name.contains("-pty");
    is_msys && is_pty
  }
}

pub fn set_title<T: Display>(title: T) {
  let buffer: Vec<u16> = OsStr::new(&format!("{}", title))
    .encode_wide()
    .chain(once(0))
    .collect();
  unsafe {
    SetConsoleTitleW(buffer.as_ptr());
  }
}

#[cfg(feature = "windows-console-colors")]
pub fn console_colors(out: &Term, mut con: Console, bytes: &[u8]) -> io::Result<()> {
  use super::ansi::AnsiCodeIterator;
  use std::str::from_utf8;

  let s = from_utf8(bytes).expect("data to be printed is not an ansi string");
  let mut iter = AnsiCodeIterator::new(s);

  while !iter.rest_slice().is_empty() {
    if let Some((part, is_esc)) = iter.next() {
      if !is_esc {
        out.write_through_common(part.as_bytes())?;
      } else if part == "\x1b[0m" {
        con.reset()?;
      } else if let Some(cap) = INTENSE_COLOR_RE.captures(part) {
        let color = get_color_from_ansi(cap.get(2).unwrap().as_str());

        match cap.get(1).unwrap().as_str() {
          "3" => con.fg(Intense::Yes, color)?,
          "4" => con.bg(Intense::Yes, color)?,
          _ => unreachable!(),
        };
      } else if let Some(cap) = NORMAL_COLOR_RE.captures(part) {
        let color = get_color_from_ansi(cap.get(2).unwrap().as_str());

        match cap.get(1).unwrap().as_str() {
          "3" => con.fg(Intense::No, color)?,
          "4" => con.bg(Intense::No, color)?,
          _ => unreachable!(),
        };
      } else if !ATTR_RE.is_match(part) {
        out.write_through_common(part.as_bytes())?;
      }
    }
  }

  Ok(())
}

#[cfg(feature = "windows-console-colors")]
fn get_color_from_ansi(ansi: &str) -> Color {
  match ansi {
    "0" | "8" => Color::Black,
    "1" | "9" => Color::Red,
    "2" | "10" => Color::Green,
    "3" | "11" => Color::Yellow,
    "4" | "12" => Color::Blue,
    "5" | "13" => Color::Magenta,
    "6" | "14" => Color::Cyan,
    "7" | "15" => Color::White,
    _ => unreachable!(),
  }
}
