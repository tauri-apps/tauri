# Python host for Tauri's `examples/api` validation app.
#
# The frontend is NOT copied here — it is the shared Svelte app in
# /examples/api, referenced through tauri.conf.json (`beforeDevCommand` +
# `devUrl` in dev, `frontendDist` -> ../../../../examples/api/dist in a build).
# This file only provides the host (Rust-equivalent) side: the commands the
# frontend invokes and the events it exchanges. app.run() blocks the main
# thread (like every Python GUI toolkit); handlers run on the event-pump thread.
#
# Run from this directory (starts the shared Vite dev server automatically):
#   ../../../../target/debug/cargo-tauri dev

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
sys.path.insert(0, str(Path(__file__).resolve().parent))
from tauri_ffi import App  # noqa: E402
from app_menu_plugin import app_menu  # noqa: E402

# Configuration comes from ./tauri.conf.json; assets are referenced from
# examples/api via `build > frontendDist` / `build > devUrl`.
app = App()

# plugin:app-menu|toggle / |popup — native side + ACL set up at run().
app.plugin(app_menu)


# Communication view: "Call Log API". Rust logs it through an ACL-scoped
# command; here we just print it (the frontend ignores the return value).
@app.command("log_operation")
def log_operation(payload, _message):
    print("[app] log_operation:", payload.get("event"), payload.get("payload") or "", flush=True)


# Communication view: "Call Request (async) API".
@app.command("perform_request")
def perform_request(payload, _message):
    print("[app] perform_request:", payload.get("endpoint"), json.dumps(payload.get("body")), flush=True)
    return {"message": "message response"}


# Communication view: "Echo". The Rust command echoes the raw request body (an
# object or a bare array), so we return the payload untouched.
@app.command("echo")
def echo(payload, _message):
    return payload


# Communication view: "Spam". The Rust command streams 1..=1000 over an
# ipc::Channel; channels are not yet bridged across the FFI boundary, so this
# resolves without streaming. TODO(tauri-ffi): channel support.
@app.command("spam")
def spam(_payload, _message):
    return None


# Communication view: "Send event to Rust" emits `js-event`; reply with
# `rust-event`, mirroring the listener in examples/api/src-tauri/src/lib.rs.
def on_js_event(payload, _message):
    print("[app] js-event:", json.dumps(payload), flush=True)
    app.emit("rust-event", {"data": "something else"})


app.listen("js-event", on_js_event)


@app.on("ready")
def ready(_message):
    print("[app] ready — serving examples/api frontend", flush=True)


sys.exit(app.run())
