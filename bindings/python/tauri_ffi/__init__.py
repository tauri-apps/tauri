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
from .plugin import Plugin

__all__ = ["App", "WebviewWindow", "Plugin", "TauriError"]

_ffi = FFI()
_ffi.cdef(CDEF)


class TauriError(RuntimeError):
    def __init__(self, code: int, message: str):
        super().__init__(message)
        self.code = code


def library_path() -> Path:
    # a frozen bundle ignores the TAURI_FFI_LIB env override — it must load only
    # its own bundled cdylib, never an arbitrary library named by the env
    env = None if _is_bundled() else os.environ.get("TAURI_FFI_LIB")
    if env:
        return Path(env)
    name = {
        "darwin": "libtauri_ffi.dylib",
        "linux": "libtauri_ffi.so",
        "win32": "tauri_ffi.dll",
    }.get(sys.platform)
    if name is None:
        raise TauriError(-1, f"unsupported platform: {sys.platform}")
    # Bundled next to the executable (frozen binary in a Tauri bundle).
    resource_dir = _bundled_resource_dir()
    if resource_dir is not None and (resource_dir / name).exists():
        return resource_dir / name
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


def _is_bundled() -> bool:
    """Whether this is a frozen, distributable binary (e.g. PyInstaller) rather
    than a plain ``python main.py`` run. Such a binary must be hermetic: the
    environment-variable config/dev overrides the Tauri CLI sets in dev
    (``TAURI_CONFIG``, ``TAURI_DEV``) are ignored so a shipped bundle can't be
    repointed at attacker-controlled config or a dev URL through its env."""
    return bool(getattr(sys, "frozen", False))


def _bundled_resource_dir() -> Optional[Path]:
    """The directory holding this app's bundled resources (cdylib, packed
    assets, config) when running as a frozen PyInstaller binary inside a Tauri
    bundle, or None in dev. Resolved from ``sys.executable``:
    ``.app/Contents/Resources`` on macOS, the executable's directory
    elsewhere."""
    if not getattr(sys, "frozen", False):
        return None
    exe = Path(sys.executable).resolve()
    for parent in exe.parents:
        if parent.name == "MacOS" and parent.parent.name == "Contents":
            return parent.parent / "Resources"
    return exe.parent


def _merge_config(target: dict, source: dict) -> dict:
    """JSON merge patch (like the CLI's config merging): objects merge
    recursively, None removes the key, everything else replaces."""
    for key, value in source.items():
        if value is None:
            target.pop(key, None)
        elif isinstance(value, dict) and isinstance(target.get(key), dict):
            _merge_config(target[key], value)
        else:
            target[key] = _merge_config({}, value) if isinstance(value, dict) else value
    return target


def _resolve_config(explicit: Optional[dict]) -> tuple:
    """Resolves the app configuration: the explicit ``config`` argument, or a
    ``tauri.conf.json`` found next to the main module (then in the working
    directory) — deep-merged with the ``TAURI_CONFIG`` environment variable
    (the Tauri CLI passes the fully merged config through it).

    Returns ``(config, config_dir)``; ``config_dir`` anchors relative
    ``build.frontendDist`` paths."""
    import copy

    config = None
    config_dir = None
    if explicit is not None:
        config = copy.deepcopy(explicit)
    else:
        candidates = []
        # resource dir first so a bundled app uses its packed config
        resource_dir = _bundled_resource_dir()
        if resource_dir is not None:
            candidates.append(resource_dir)
        if sys.argv and sys.argv[0]:
            candidates.append(Path(sys.argv[0]).resolve().parent)
        candidates.append(Path.cwd())
        for candidate in candidates:
            file = candidate / "tauri.conf.json"
            if file.exists():
                config = json.loads(file.read_text())
                config_dir = candidate
                break

    # a frozen bundle ignores the TAURI_CONFIG env override (hermetic in production)
    env = None if _is_bundled() else os.environ.get("TAURI_CONFIG")
    if env:
        config = _merge_config(config or {}, json.loads(env))
    if config is None:
        raise TauriError(
            -1,
            "no configuration found — pass `config` to App() or add a tauri.conf.json next to your app entry",
        )
    return config, config_dir


def _resolve_assets(
    assets_dir: Optional[os.PathLike],
    assets_archive: Optional[os.PathLike],
    config: dict,
    config_dir: Optional[Path],
) -> tuple:
    """Resolves where frontend assets come from, in precedence order: explicit
    arguments, then the config's ``build.frontendDist`` (a ``.assets`` archive
    — e.g. one packed by ``tauri build``, resolved next to the config — or a
    directory; URLs are handled by the runtime).

    The asset source is deliberately not overridable by an environment
    variable: the frontend is trusted content (it can invoke commands), so its
    origin must come from code or the bundled config, never the ambient env.

    Returns ``(dir, archive)`` (one of them, or both, is None)."""
    if assets_dir is not None:
        return assets_dir, None
    if assets_archive is not None:
        return None, assets_archive

    dist = (config.get("build") or {}).get("frontendDist")
    if not isinstance(dist, str) or dist.startswith(("http:", "https:")):
        return None, None
    resolved = (config_dir or Path.cwd()) / dist
    if dist.endswith(".assets"):
        return None, resolved
    return resolved, None


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
    """Register commands/handlers, then call ``run()`` from the main thread.

    ``config`` is optional: when omitted, a ``tauri.conf.json`` next to the
    main module (or in the working directory) is used. The ``TAURI_CONFIG``
    environment variable (set by the Tauri CLI) is deep-merged on top in
    either case, and ``TAURI_DEV`` enables dev mode."""

    def __init__(
        self,
        config: Optional[dict] = None,
        *,
        assets_dir: Optional[os.PathLike] = None,
        assets_archive: Optional[os.PathLike] = None,
        dev: Optional[bool] = None,
        capabilities: Optional[list] = None,
        library: Optional[Path] = None,
    ):
        self._config, config_dir = _resolve_config(config)
        self._assets_dir, self._assets_archive = _resolve_assets(
            assets_dir, assets_archive, self._config, config_dir
        )
        # a frozen bundle ignores the TAURI_DEV env override (hermetic in production)
        if dev is not None:
            self._dev = dev
        else:
            self._dev = not _is_bundled() and os.environ.get("TAURI_DEV") == "true"
        self._capabilities = capabilities or []
        self._library = library
        self._commands: dict[str, Callable] = {}
        self._lifecycle: dict[str, list[Callable]] = {}
        self._listeners: dict[int, Callable] = {}
        self._pending_listens: list[tuple] = []
        self._plugins: list[Plugin] = []
        self._plugin_handlers: dict[tuple, Callable] = {}
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

    def plugin(self, plugin: Plugin) -> None:
        """Registers a plugin (from ``Plugin``) before ``run()`` — its native
        side and ACL are set up at build, and its command handlers are wired up.
        The ``plugin:<name>|<command>`` wire format never appears in app code."""
        self._plugins.append(plugin)
        for name, handler in plugin.commands.items():
            self._plugin_handlers[(plugin.name, name)] = handler

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

        if self._dev:
            _check(lib, lib.tauri_app_builder_set_dev(builder, True), "set_dev")
        if self._assets_archive is not None:
            _check(
                lib,
                lib.tauri_app_builder_set_assets_archive(builder, _s(str(self._assets_archive))),
                "set_assets_archive",
            )
        elif self._assets_dir is not None:
            _check(lib, lib.tauri_app_builder_set_assets_dir(builder, _s(str(self._assets_dir))), "set_assets_dir")
        for name in self._commands:
            _check(lib, lib.tauri_app_builder_register_command(builder, _s(name)), f"register_command({name})")
        for capability in self._capabilities:
            value = capability if isinstance(capability, str) else json.dumps(capability)
            _check(lib, lib.tauri_app_builder_add_capability(builder, _s(value)), "add_capability")
        for plugin in self._plugins:
            out_plugin = _ffi.new("uint64_t *")
            _check(lib, lib.tauri_plugin_new(_s(plugin.name), out_plugin), f"plugin_new({plugin.name})")
            handle = out_plugin[0]
            if plugin.script:
                _check(
                    lib,
                    lib.tauri_plugin_set_init_script(handle, _s(plugin.script)),
                    f"plugin_set_init_script({plugin.name})",
                )
            for command in plugin.commands:
                _check(
                    lib,
                    lib.tauri_plugin_register_command(handle, _s(command)),
                    f"plugin_register_command({plugin.name}|{command})",
                )
            _check(lib, lib.tauri_app_builder_add_plugin(builder, handle), f"add_plugin({plugin.name})")

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
        plugin = message.get("plugin")
        if plugin is not None:
            handler = self._plugin_handlers.get((plugin, message["command"]))
            label = f"plugin:{plugin}|{message['command']}"
        else:
            handler = self._commands.get(message["command"])
            label = message["command"]
        if handler is None:
            self._lib.tauri_invoke_reject(
                message["id"], _s(json.dumps(f"command {label} not found"))
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
