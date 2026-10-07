# CEF off-screen rendering

This example renders a native Vello scene and composites a transparent CEF UI
over it using wgpu. It uses software CEF paint buffers; the runtime does not
depend on Vello or wgpu.

Use the v3 Tauri CLI, then run `cargo tauri dev` from this directory. The CLI
sets up the CEF framework and helper applications required on macOS. To check
the Rust source without launching a window, run `cargo check -p cef-offscreen`
from the repository root.

The panel exercises a Rust command, text input and IME, popup widgets, pointer
capture, and scrolling. Resize the window or move it between displays to check
the native surface and browser scale together. The orange circle is native
Vello content, visible through the transparent browser background.

The application owns the wgpu device, textures, and presentation surface. CEF
paints are copied into `OffscreenSurface`; its callback schedules a native
redraw with `WindowCefExt::request_redraw`. `on_redraw` presents both layers.
GPU resources are released on `WindowEvent::Destroyed`.

For shared textures, configure `OffscreenSurface::with_accelerated_paint` before
cloning it or creating the webview. Open and copy each platform resource during
the callback, synchronizing GPU access before returning. The handle cannot be
cached for later presentation. The application chooses its own graphics API
and importer; enabling `cef/accelerated_osr` is not required for the callback.

CEF controls browser frame scheduling by default. Applications that enable
`with_external_begin_frame(true)` must send `BrowserHost::send_external_begin_frame`
through `WebviewCefExt::with_cef_webview` to advance Chromium frames.

Windowless webviews retain Tauri IPC and custom protocols. They cannot be
reparented because CEF fixes the native dialog parent at browser creation.
`window.open` creates an ordinary native browser; select-menu popup widgets are
part of the offscreen surface. Linux continues to use the runtime's X11 backend.
