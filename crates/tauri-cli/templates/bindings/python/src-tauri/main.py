# Process entry point. `app.run()` blocks the main thread (like every Python
# GUI toolkit); command and event handlers run on the event-pump thread.
# Started by `tauri dev` / `tauri build` through the `build > runner` command
# in tauri.conf.json. Configuration comes from ./tauri.conf.json.
import sys

from tauri_ffi import App

app = App()


# Frontend calls this with `invoke('greet', { name })`.
@app.command("greet")
def greet(payload, _message):
    return {"message": f"Hello, {payload['name']}! You've been greeted from Python."}


@app.on("ready")
def ready(_message):
    print("Tauri app ready", flush=True)


sys.exit(app.run())
