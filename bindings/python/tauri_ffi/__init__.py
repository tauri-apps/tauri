# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

"""Python bindings to Tauri over the tauri-ffi C ABI (cffi ABI mode — no
compiler needed). Runtime layer over the generated declarations in
tauri_ffi_cdef.py; only hand-written sugar lives here.

Threading model: ``App.run()`` blocks the calling thread — which must be the
process main thread (hard requirement on macOS) — exactly like every Python
GUI toolkit. A daemon thread pumps the serialized event queue
(``tauri_events_next``) and dispatches to your handlers, so handlers run off
the main thread; cffi releases the GIL during C calls, so both threads make
progress.
"""

from __future__ import annotations

import json
import os
import sys
import threading
from pathlib import Path
from typing import Any, Callable, Optional

from cffi import FFI

from ._cdef import ABI_VERSION, CDEF, CODES

_ffi = FFI()
_ffi.cdef(CDEF)


class TauriError(RuntimeError):
    def __init__(self, code: int, message: str):
        super().__init__(message)
        self.code = code


def library_path() -> Path:
    env = os.environ.get("TAURI_FFI_LIB")
    if env:
        return Path(env)
    name = {
        "darwin": "libtauri_ffi.dylib",
        "linux": "libtauri_ffi.so",
        "win32": "tauri_ffi.dll",
    }.get(sys.platform)
    if name is None:
        raise TauriError(-1, f"unsupported platform: {sys.platform}")
    # Installed wheels bundle the library next to the package.
    bundled = Path(__file__).resolve().parent / "_native" / name
    if bundled.exists():
        return bundled
    # Development fallback: cargo build output in the repo.
    repo_root = Path(__file__).resolve().parents[3]
    for profile in ("debug", "release"):
        candidate = repo_root / "target" / profile / name
        if candidate.exists():
            return candidate
    raise TauriError(
        -1, "tauri_ffi library not found — run `cargo build -p tauri-ffi` or set TAURI_FFI_LIB"
    )


def _open_lib(path: Optional[Path] = None):
    lib = _ffi.dlopen(str(path or library_path()))
    abi = lib.tauri_ffi_abi_version()
    if abi != ABI_VERSION:
        raise TauriError(-1, f"ABI mismatch: library has v{abi}, bindings expect v{ABI_VERSION}")
    return lib


def _s(text: str) -> bytes:
    return text.encode("utf-8")


def _check(lib, code: int, what: str) -> None:
    if code != CODES["OK"]:
        message_ptr = lib.tauri_last_error_message()
        message = _ffi.string(message_ptr).decode() if message_ptr != _ffi.NULL else "unknown error"
        raise TauriError(code, f"{what} failed ({code}): {message}")


def _take_string(lib, out) -> Optional[str]:
    if out[0] == _ffi.NULL:
        return None
    value = _ffi.string(out[0]).decode()
    lib.tauri_string_free(out[0])
    return value


class WebviewWindow:
    """Mirrors ``tauri::WebviewWindow`` — one method per Rust method. Obtain
    instances via ``App.create_window()`` / ``App.get_window()``; call
    ``free()`` when done with the handle (the window itself is unaffected)."""

    def __init__(self, lib, handle: int):
        self._lib = lib
        self._handle = handle

    # -- getters -----------------------------------------------------------

    def _string(self, fn, what: str) -> str:
        out = _ffi.new("char **")
        _check(self._lib, fn(self._handle, out), what)
        return _take_string(self._lib, out) or ""

    def label(self) -> str:
        return self._string(self._lib.tauri_window_label, "window.label")

    def title(self) -> str:
        return self._string(self._lib.tauri_window_title, "window.title")

    def url(self) -> str:
        return self._string(self._lib.tauri_window_url, "window.url")

    def scale_factor(self) -> float:
        out = _ffi.new("double *")
        _check(self._lib, self._lib.tauri_window_scale_factor(self._handle, out), "window.scale_factor")
        return out[0]

    def _pair(self, fn, ctype: str, what: str) -> tuple:
        a = _ffi.new(ctype)
        b = _ffi.new(ctype)
        _check(self._lib, fn(self._handle, a, b), what)
        return (a[0], b[0])

    def inner_size(self) -> tuple:
        """(width, height) in physical pixels."""
        return self._pair(self._lib.tauri_window_inner_size, "uint32_t *", "window.inner_size")

    def outer_size(self) -> tuple:
        return self._pair(self._lib.tauri_window_outer_size, "uint32_t *", "window.outer_size")

    def inner_position(self) -> tuple:
        return self._pair(self._lib.tauri_window_inner_position, "int32_t *", "window.inner_position")

    def outer_position(self) -> tuple:
        return self._pair(self._lib.tauri_window_outer_position, "int32_t *", "window.outer_position")

    def _bool(self, fn, what: str) -> bool:
        out = _ffi.new("bool *")
        _check(self._lib, fn(self._handle, out), what)
        return bool(out[0])

    def is_visible(self) -> bool:
        return self._bool(self._lib.tauri_window_is_visible, "window.is_visible")

    def is_focused(self) -> bool:
        return self._bool(self._lib.tauri_window_is_focused, "window.is_focused")

    def is_fullscreen(self) -> bool:
        return self._bool(self._lib.tauri_window_is_fullscreen, "window.is_fullscreen")

    def is_maximized(self) -> bool:
        return self._bool(self._lib.tauri_window_is_maximized, "window.is_maximized")

    def is_minimized(self) -> bool:
        return self._bool(self._lib.tauri_window_is_minimized, "window.is_minimized")

    def is_resizable(self) -> bool:
        return self._bool(self._lib.tauri_window_is_resizable, "window.is_resizable")

    # -- setters & actions ---------------------------------------------------

    def set_title(self, title: str) -> None:
        _check(self._lib, self._lib.tauri_window_set_title(self._handle, _s(title)), "window.set_title")

    def set_size(self, width: float, height: float, physical: bool = False) -> None:
        """Logical (DPI-scaled) pixels unless physical=True."""
        _check(self._lib, self._lib.tauri_window_set_size(self._handle, width, height, physical), "window.set_size")

    def set_position(self, x: float, y: float, physical: bool = False) -> None:
        _check(self._lib, self._lib.tauri_window_set_position(self._handle, x, y, physical), "window.set_position")

    def _unit(self, fn, what: str) -> None:
        _check(self._lib, fn(self._handle), what)

    def set_fullscreen(self, fullscreen: bool) -> None:
        _check(self._lib, self._lib.tauri_window_set_fullscreen(self._handle, fullscreen), "window.set_fullscreen")

    def set_resizable(self, resizable: bool) -> None:
        _check(self._lib, self._lib.tauri_window_set_resizable(self._handle, resizable), "window.set_resizable")

    def set_always_on_top(self, always_on_top: bool) -> None:
        _check(self._lib, self._lib.tauri_window_set_always_on_top(self._handle, always_on_top), "window.set_always_on_top")

    def set_decorations(self, decorations: bool) -> None:
        _check(self._lib, self._lib.tauri_window_set_decorations(self._handle, decorations), "window.set_decorations")

    def set_focus(self) -> None:
        self._unit(self._lib.tauri_window_set_focus, "window.set_focus")

    def set_zoom(self, scale: float) -> None:
        _check(self._lib, self._lib.tauri_window_set_zoom(self._handle, scale), "window.set_zoom")

    def show(self) -> None:
        self._unit(self._lib.tauri_window_show, "window.show")

    def hide(self) -> None:
        self._unit(self._lib.tauri_window_hide, "window.hide")

    def center(self) -> None:
        self._unit(self._lib.tauri_window_center, "window.center")

    def maximize(self) -> None:
        self._unit(self._lib.tauri_window_maximize, "window.maximize")

    def unmaximize(self) -> None:
        self._unit(self._lib.tauri_window_unmaximize, "window.unmaximize")

    def minimize(self) -> None:
        self._unit(self._lib.tauri_window_minimize, "window.minimize")

    def unminimize(self) -> None:
        self._unit(self._lib.tauri_window_unminimize, "window.unminimize")

    def close(self) -> None:
        self._unit(self._lib.tauri_window_close, "window.close")

    def destroy(self) -> None:
        self._unit(self._lib.tauri_window_destroy, "window.destroy")

    def eval(self, js: str) -> None:
        _check(self._lib, self._lib.tauri_window_eval(self._handle, _s(js)), "window.eval")

    def navigate(self, url: str) -> None:
        _check(self._lib, self._lib.tauri_window_navigate(self._handle, _s(url)), "window.navigate")

    def reload(self) -> None:
        self._unit(self._lib.tauri_window_reload, "window.reload")

    def free(self) -> None:
        """Releases the handle; the window itself is unaffected."""
        _check(self._lib, self._lib.tauri_handle_close(self._handle), "window.free")


class App:
    """Register commands/handlers, then call ``run()`` from the main thread."""

    def __init__(
        self,
        config: dict,
        *,
        assets_dir: Optional[os.PathLike] = None,
        capabilities: Optional[list] = None,
        library: Optional[Path] = None,
    ):
        self._config = config
        self._assets_dir = assets_dir
        self._capabilities = capabilities or []
        self._library = library
        self._commands: dict[str, Callable] = {}
        self._lifecycle: dict[str, list[Callable]] = {}
        self._listeners: dict[int, Callable] = {}
        self._pending_listens: list[tuple] = []
        self._lib = None
        self._app = 0

    # -- registration ---------------------------------------------------------

    def command(self, name: str):
        """Decorator: ``handler(payload, message)`` resolves/rejects the invoke."""

        def register(handler):
            self._commands[name] = handler
            return handler

        return register

    def on(self, event_type: str):
        """Decorator for lifecycle events: 'ready' | 'exit' | 'exit-requested' | 'window-event'."""

        def register(handler):
            self._lifecycle.setdefault(event_type, []).append(handler)
            return handler

        return register

    def listen(self, event: str, handler: Callable) -> Optional[int]:
        """Listens to a Tauri event; returns the listener id (None pre-run)."""
        if self._app == 0:
            self._pending_listens.append((event, handler))
            return None
        return self._listen_now(event, handler)

    def _listen_now(self, event: str, handler: Callable) -> int:
        out = _ffi.new("uint32_t *")
        _check(self._lib, self._lib.tauri_app_listen(self._app, _s(event), out), f"listen({event})")
        self._listeners[out[0]] = handler
        return out[0]

    def unlisten(self, listener: int) -> None:
        self._listeners.pop(listener, None)
        _check(self._lib, self._lib.tauri_app_unlisten(self._app, listener), "unlisten")

    # -- runtime ---------------------------------------------------------------

    def emit(self, event: str, payload: Any = None) -> None:
        _check(
            self._lib,
            self._lib.tauri_app_emit(self._app, _s(event), _s(json.dumps(payload))),
            f"emit({event})",
        )

    def emit_to(self, label: str, event: str, payload: Any = None) -> None:
        _check(
            self._lib,
            self._lib.tauri_app_emit_to(self._app, _s(label), _s(event), _s(json.dumps(payload))),
            f"emit_to({event})",
        )

    def create_window(self, config: dict) -> WebviewWindow:
        """Creates a window from a WindowConfig dict (same shape as entries of
        app.windows in tauri.conf.json). Call only while the app is running."""
        out = _ffi.new("uint64_t *")
        _check(
            self._lib,
            self._lib.tauri_window_create(self._app, _s(json.dumps(config)), out),
            f"create_window({config.get('label')})",
        )
        return WebviewWindow(self._lib, out[0])

    def get_window(self, label: str) -> Optional[WebviewWindow]:
        out = _ffi.new("uint64_t *")
        code = self._lib.tauri_app_get_window(self._app, _s(label), out)
        if code == CODES["NOT_FOUND"]:
            return None
        _check(self._lib, code, f"get_window({label})")
        return WebviewWindow(self._lib, out[0])

    def window_labels(self) -> list:
        out = _ffi.new("char **")
        _check(self._lib, self._lib.tauri_app_window_labels(self._app, out), "window_labels")
        return json.loads(_take_string(self._lib, out) or "[]")

    def exit(self, code: int = 0) -> None:
        _check(self._lib, self._lib.tauri_app_exit(self._app, code), "exit")

    def run(self) -> int:
        """Builds the app and runs the event loop on the calling thread (must
        be the process main thread). Blocks until exit; returns the exit code."""
        lib = self._lib = _open_lib(self._library)

        out_builder = _ffi.new("uint64_t *")
        _check(lib, lib.tauri_app_builder_new(_s(json.dumps(self._config)), out_builder), "builder_new")
        builder = out_builder[0]

        if self._assets_dir is not None:
            _check(lib, lib.tauri_app_builder_set_assets_dir(builder, _s(str(self._assets_dir))), "set_assets_dir")
        for name in self._commands:
            _check(lib, lib.tauri_app_builder_register_command(builder, _s(name)), f"register_command({name})")
        for capability in self._capabilities:
            value = capability if isinstance(capability, str) else json.dumps(capability)
            _check(lib, lib.tauri_app_builder_add_capability(builder, _s(value)), "add_capability")

        out_app = _ffi.new("uint64_t *")
        _check(lib, lib.tauri_app_build(builder, out_app), "app_build")
        self._app = out_app[0]

        for event, handler in self._pending_listens:
            self._listen_now(event, handler)
        self._pending_listens.clear()

        pump = threading.Thread(target=self._pump, name="tauri-ffi-events", daemon=True)
        pump.start()

        out_code = _ffi.new("int32_t *")
        _check(lib, lib.tauri_app_run(self._app, out_code), "app_run")
        return out_code[0]

    # -- event pump (daemon thread) ---------------------------------------------

    def _pump(self) -> None:
        while True:
            out = _ffi.new("char **")
            code = self._lib.tauri_events_next(self._app, 1000, out)
            if code == CODES["TIMEOUT"]:
                continue
            if code == CODES["CLOSED"]:
                return
            if code != CODES["OK"]:
                print(f"[tauri-ffi] event pump failed ({code})", file=sys.stderr)
                return
            message = json.loads(_take_string(self._lib, out) or "null")
            try:
                self._dispatch(message)
            except Exception as error:  # user handler errors must not kill the pump
                print(f"[tauri-ffi] handler error: {error!r}", file=sys.stderr)
            if message.get("type") == "exit":
                return

    def _dispatch(self, message: dict) -> None:
        kind = message.get("type")
        if kind == "invoke":
            self._handle_invoke(message)
        elif kind == "event":
            handler = self._listeners.get(message.get("id"))
            if handler:
                handler(message.get("payload"), message)
        else:
            for handler in self._lifecycle.get(kind, []):
                handler(message)

    def _handle_invoke(self, message: dict) -> None:
        handler = self._commands.get(message["command"])
        if handler is None:
            self._lib.tauri_invoke_reject(
                message["id"], _s(json.dumps(f"command {message['command']} not found"))
            )
            return
        try:
            result = handler(message.get("payload"), message)
            _check(
                self._lib,
                self._lib.tauri_invoke_resolve(message["id"], _s(json.dumps(result))),
                "invoke_resolve",
            )
        except Exception as error:
            self._lib.tauri_invoke_reject(message["id"], _s(json.dumps(str(error))))
