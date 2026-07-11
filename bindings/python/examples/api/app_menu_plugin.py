# The `app-menu` plugin the shared frontend expects. The Welcome view's
# "Context menu" button invokes `plugin:app-menu|popup`, and Ctrl/Cmd+B invokes
# `plugin:app-menu|toggle`. examples/api wires these to real menus in Rust; here
# we just acknowledge them so the invoke resolves cleanly. Build out with the
# menu API (App.create_menu / App.set_app_menu / window hide/show menu) to make
# them do something.

from tauri_ffi import Plugin

app_menu = Plugin("app-menu")


@app_menu.command("toggle")
def toggle(_payload, _message):
    print("[app] app-menu toggle", flush=True)


@app_menu.command("popup")
def popup(_payload, _message):
    print("[app] app-menu popup", flush=True)
