# Example tauri-ffi app in Python. Run from anywhere:
#   pip install cffi && python3 bindings/python/examples/hello/main.py
# app.run() blocks the main thread (like every Python GUI toolkit); handlers
# run on the event-pump thread.

import itertools
import json
import os
import sys
import threading
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
sys.path.insert(0, str(Path(__file__).resolve().parent))
from tauri_ffi import App, TauriError  # noqa: E402
from demo_plugin import demo  # noqa: E402

STATUS = os.environ.get("FIXTURE_STATUS")


def log(*args):
    line = " ".join(str(a) for a in args)
    print("[app]", line, flush=True)
    if STATUS:
        with open(STATUS, "a") as f:
            f.write(line + "\n")


# Configuration comes from ./tauri.conf.json; assets from `build > frontendDist`.
app = App()

# Register the plugin — its native side + ACL are set up at run(), and its
# command handlers are wired up. No `plugin:demo|echo` strings in app code.
app.plugin(demo)

child_counter = itertools.count(1)


@app.command("greet")
def greet(payload, _message):
    log("greet:", payload["name"])
    return {"message": f"Hello {payload['name']}! Greetings from Python {sys.version.split()[0]}."}


@app.command("open-window")
def open_window(_payload, _message):
    label = f"child-{next(child_counter)}"
    win = app.create_window(
        {"label": label, "url": "index.html", "title": f"Child ({label})", "width": 420, "height": 320}
    )
    win.free()
    return {"label": label}


def tick():
    for count in itertools.count(1):
        time.sleep(1)
        try:
            app.emit("tick", {"count": count})
        except Exception:
            return


@app.on("ready")
def ready(_message):
    log("ready")

    # Exercise the window API against the config-declared main window.
    main = app.get_window("main")
    log("main-title:", main.title())
    log("main-url:", main.url())  # dev server URL under `tauri dev`, tauri://localhost in production
    width, height = main.inner_size()
    log("main-size:", "ok" if width > 0 and height > 0 else f"bad {width}x{height}")
    log("labels:", json.dumps(app.window_labels()))
    main.set_title("Tauri FFI — Python (ready)")

    # Platform-gated APIs: same symbols on every platform, ERR_UNSUPPORTED (-6)
    # where the feature does not exist.
    if sys.platform == "darwin":
        app.set_activation_policy("regular")
        main.set_title_bar_style("visible")
        log("platform-api:", "ok" if main.ns_window() > 0 else "bad ns_window")
    try:
        main.hwnd()
        log("platform-gate:", "ok" if sys.platform == "win32" else "hwnd unexpectedly succeeded")
    except TauriError as e:
        log("platform-gate:", "ok" if "only supported on Windows" in str(e) else f"bad {e}")

    # Runtime window creation: hidden child, checked and torn down again.
    child = app.create_window(
        {"label": "child", "url": "index.html", "title": "child", "width": 320, "height": 240, "visible": False}
    )
    log("child-created:", child.label(), "visible" if child.is_visible() else "hidden")
    child.destroy()
    child.free()
    main.free()
    log("child-destroyed")

    threading.Thread(target=tick, daemon=True).start()


@app.on("window-event")
def window_event(message):
    log("window-event:", message["label"], message["event"]["kind"])


app.listen("frontend-ping", lambda payload, _message: log("frontend-ping:", json.dumps(payload)))
app.listen(
    "plugin-init-ran",
    lambda payload, _message: log("plugin-init:", "ran" if payload.get("ran") else "missing"),
)
app.listen("plugin-echo-ok", lambda payload, _message: log("plugin-echo:", json.dumps(payload)))

sys.exit(app.run())
