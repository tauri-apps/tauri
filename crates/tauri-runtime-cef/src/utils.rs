#[cfg(windows)]
pub mod windows {
  use tauri_runtime::dpi::PhysicalSize;
  use windows::Win32::Foundation::*;
  use windows::Win32::UI::WindowsAndMessaging::*;

  pub fn inner_size(hwnd: cef::sys::HWND) -> PhysicalSize<u32> {
    let hwnd = HWND(hwnd.0 as _);
    let mut rect = RECT::default();
    let _ = unsafe { GetClientRect(hwnd, &mut rect) };

    PhysicalSize::new(
      (rect.right - rect.left) as u32,
      (rect.bottom - rect.top) as u32,
    )
  }

  pub fn adjust_size(hwnd: cef::sys::HWND, size: PhysicalSize<u32>) -> PhysicalSize<u32> {
    let hwnd = HWND(hwnd.0 as _);

    let mut client_rect = RECT::default();
    let _ = unsafe { GetClientRect(hwnd, &mut client_rect) };
    let client_width = client_rect.right - client_rect.left;
    let client_height = client_rect.bottom - client_rect.top;

    let mut window_rect = RECT::default();
    let _ = unsafe { GetWindowRect(hwnd, &mut window_rect) };
    let window_width = window_rect.right - window_rect.left;
    let window_height = window_rect.bottom - window_rect.top;

    let width_diff = window_width - client_width;
    let height_diff = window_height - client_height;

    PhysicalSize::new(
      size.width + width_diff as u32,
      size.height + height_diff as u32,
    )
  }
}
