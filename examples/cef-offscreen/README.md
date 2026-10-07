# CEF offscreen rendering

Renders a native Vello scene and composites a transparent CEF webview over it
with wgpu. The example uses software paint buffers and keeps its own frame
copies and GPU resources. The runtime has no graphics dependency.

## Run

From the repository root:

```sh
cargo build -p tauri-cli
cd examples/cef-offscreen
../../target/debug/cargo-tauri dev
```

Use the workspace CLI so macOS gets the CEF framework and helper applications
inside an app bundle. The first build downloads CEF if it is not cached. This
example has no frontend build step or JavaScript dependencies.

## Try it

- Type in the input, select a popup option, and scroll the text area.
- Click **Call Rust** to invoke a Tauri command.
- Click the native circle. Its color and the window title change while the
  browser click count stays unchanged.
- Drag from the blue panel onto the circle. The browser retains the gesture
  and displays **Drag finished** when released.
- Resize the window or move it between displays to exercise surface sizing
  and scale changes.

[`main.rs`](src-tauri/src/main.rs) enables windowless rendering, receives
borrowed paints, and handles native events. Tauri forwards browser input and
manages focus, IME and popup geometry. The example copies each complete paint
because it presents later on a native redraw; popup bounds remain in logical
pixels until presentation.

[`renderer.rs`](src-tauri/src/renderer.rs) owns the wgpu surface, textures and
Vello renderer. It uploads changed frames, composites premultiplied BGRA pixels,
and releases the surface on `Destroyed`. Accelerated CEF texture import is
outside this example.
