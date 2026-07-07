# Examples

Run the `cargo run --example <file_name>` to see how each example works.

- `control_flow`: tell event loop what to do in the next iteration, after the current one's finished.
- `custom_events`: user can create custom events and emit or listen to them through tao.
- `fullscreen`: example for configuring different screen sizes, and video modes.
- `handling_close`: close window with a warning.
- `request_redraw_threaded`: same as request_redraw but multithreaded.
- `request_redraw`: an event emitted when it's needed to redraw (when resizing window for example).
- `timer`: an example that makes a timer which suspend the thread for some time.
- `window_run_return`: similar to run function of EventLoop, but accept non-move closures and returns control flow to the caller when exit.
- `window_debug`: example that debugs with eprintln.

## Quite self-explainatory examples.

- `cursor_grab`: prevent the cursor from going outside the window.
- `cursor`: set different cursor icons.
- `drag_window`: allow dragging window when hold left mouse and move.
- `min_max_size`: set smallest/largest window size you can zoom.
- `minimize`: minimize window.
- `monitor_list`: list all available monitors.
- `mouse_wheel`: get the difference in scrolling state (MouseScrollDelta) in pixel or line.
- `multithreaded`: same as multiwindow but multithreaded.
- `multiwindow`: create multiple windows
- `parentwindow`: a window inside another window.
- `reopen_event`: handle click on dock icon on macOS
- `resizable`: allow resizing window or not.
- `set_ime_position`: set IME (input method editor) position when click.
- `transparent`: make a transparent window.
- `video_modes`: example that lists all video modes of primary monitor
- `window_icon`: add window icon.
- `window`: example that makes a window.
