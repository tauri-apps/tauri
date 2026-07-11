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

__all__ = ["App", "WebviewWindow", "Window", "Webview", "Menu", "MenuItem", "Tray", "Plugin", "TauriError"]

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


def _embedded_dir() -> Optional[Path]:
    """The directory of the payload embedded in a frozen (PyInstaller) binary —
    ``app.assets``, ``config.json`` and ``capabilities.json`` extracted to
    ``sys._MEIPASS`` at startup — or None in dev. This is how a shipped bundle
    carries its assets/config/ACL *inside* the executable rather than as sibling
    files. Unlike Node/Deno, PyInstaller extracts to a real filesystem path, so
    the assets archive is read back by path (the cdylib reads it directly)."""
    meipass = getattr(sys, "_MEIPASS", None)
    return Path(meipass) if meipass else None


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


# Capability files carry these extensions; a ``schemas/`` subfolder holds JSON
# schema files, not capabilities, so it is skipped (mirrors tauri-build).
_CAPABILITY_EXTENSIONS = (".json", ".json5", ".toml")


def _resolve_capabilities(dirs) -> list:
    """Reads capability files from a ``capabilities/`` directory next to the
    config, mirroring the compile-time discovery a Rust app gets from
    tauri-build (``capabilities/**`` glob). Each file's raw content is a
    capability-file string (JSON or TOML; a single capability or a list) handed
    to ``add_capability``.

    The directory is looked up next to the resolved config (then the main
    module, then the bundled resource dir). Returns ``[]`` when there is none —
    the app then falls back to the built-in ``core:default`` capability."""
    for base in dirs:
        if base is None:
            continue
        cap_dir = Path(base) / "capabilities"
        if not cap_dir.is_dir():
            continue
        files = [
            p
            for p in sorted(cap_dir.rglob("*"))
            if p.is_file()
            and p.suffix.lower() in _CAPABILITY_EXTENSIONS
            and "schemas" not in p.relative_to(cap_dir).parts
        ]
        return [p.read_text() for p in files]
    return []


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
        return self._string(self._lib.tauri_webview_window_label, "window.label")

    def title(self) -> str:
        return self._string(self._lib.tauri_webview_window_title, "window.title")

    def url(self) -> str:
        return self._string(self._lib.tauri_webview_window_url, "window.url")

    def scale_factor(self) -> float:
        out = _ffi.new("double *")
        _check(self._lib, self._lib.tauri_webview_window_scale_factor(self._handle, out), "window.scale_factor")
        return out[0]

    def _pair(self, fn, ctype: str, what: str) -> tuple:
        a = _ffi.new(ctype)
        b = _ffi.new(ctype)
        _check(self._lib, fn(self._handle, a, b), what)
        return (a[0], b[0])

    def inner_size(self) -> tuple:
        """(width, height) in physical pixels."""
        return self._pair(self._lib.tauri_webview_window_inner_size, "uint32_t *", "window.inner_size")

    def outer_size(self) -> tuple:
        return self._pair(self._lib.tauri_webview_window_outer_size, "uint32_t *", "window.outer_size")

    def inner_position(self) -> tuple:
        return self._pair(self._lib.tauri_webview_window_inner_position, "int32_t *", "window.inner_position")

    def outer_position(self) -> tuple:
        return self._pair(self._lib.tauri_webview_window_outer_position, "int32_t *", "window.outer_position")

    def _bool(self, fn, what: str) -> bool:
        out = _ffi.new("bool *")
        _check(self._lib, fn(self._handle, out), what)
        return bool(out[0])

    def is_visible(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_visible, "window.is_visible")

    def is_focused(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_focused, "window.is_focused")

    def is_fullscreen(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_fullscreen, "window.is_fullscreen")

    def is_maximized(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_maximized, "window.is_maximized")

    def is_minimized(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_minimized, "window.is_minimized")

    def is_resizable(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_resizable, "window.is_resizable")

    def is_decorated(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_decorated, "window.is_decorated")

    def is_closable(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_closable, "window.is_closable")

    def is_maximizable(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_maximizable, "window.is_maximizable")

    def is_minimizable(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_minimizable, "window.is_minimizable")

    def is_always_on_top(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_always_on_top, "window.is_always_on_top")

    def is_enabled(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_enabled, "window.is_enabled")

    def is_menu_visible(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_menu_visible, "window.is_menu_visible")

    def is_devtools_open(self) -> bool:
        return self._bool(self._lib.tauri_webview_window_is_devtools_open, "window.is_devtools_open")

    def theme(self) -> str:
        """The window's current theme ("light" or "dark")."""
        return self._string(self._lib.tauri_webview_window_theme, "window.theme")

    def available_monitors(self) -> list:
        """All available monitors, as Monitor dicts."""
        return json.loads(
            self._string(self._lib.tauri_webview_window_available_monitors, "window.available_monitors") or "[]"
        )

    def current_monitor(self):
        """The monitor the window is on, or None."""
        return json.loads(
            self._string(self._lib.tauri_webview_window_current_monitor, "window.current_monitor") or "null"
        )

    def primary_monitor(self):
        """The primary monitor, or None."""
        return json.loads(
            self._string(self._lib.tauri_webview_window_primary_monitor, "window.primary_monitor") or "null"
        )

    def monitor_from_point(self, x: float, y: float):
        """The monitor containing the given physical point, or None."""
        out = _ffi.new("char **")
        _check(
            self._lib,
            self._lib.tauri_webview_window_monitor_from_point(self._handle, x, y, out),
            "window.monitor_from_point",
        )
        return json.loads(_take_string(self._lib, out) or "null")

    def cursor_position(self) -> tuple:
        """(x, y) cursor position in physical pixels."""
        return self._pair(self._lib.tauri_webview_window_cursor_position, "double *", "window.cursor_position")

    # -- setters & actions ---------------------------------------------------

    def set_title(self, title: str) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_title(self._handle, _s(title)), "window.set_title")

    def set_size(self, width: float, height: float, physical: bool = False) -> None:
        """Logical (DPI-scaled) pixels unless physical=True."""
        _check(self._lib, self._lib.tauri_webview_window_set_size(self._handle, width, height, physical), "window.set_size")

    def set_position(self, x: float, y: float, physical: bool = False) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_position(self._handle, x, y, physical), "window.set_position")

    def _unit(self, fn, what: str) -> None:
        _check(self._lib, fn(self._handle), what)

    def set_fullscreen(self, fullscreen: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_fullscreen(self._handle, fullscreen), "window.set_fullscreen")

    def set_resizable(self, resizable: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_resizable(self._handle, resizable), "window.set_resizable")

    def set_always_on_top(self, always_on_top: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_always_on_top(self._handle, always_on_top), "window.set_always_on_top")

    def set_decorations(self, decorations: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_decorations(self._handle, decorations), "window.set_decorations")

    def set_focus(self) -> None:
        self._unit(self._lib.tauri_webview_window_set_focus, "window.set_focus")

    def set_zoom(self, scale: float) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_zoom(self._handle, scale), "window.set_zoom")

    def set_closable(self, closable: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_closable(self._handle, closable), "window.set_closable")

    def set_maximizable(self, maximizable: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_maximizable(self._handle, maximizable), "window.set_maximizable")

    def set_minimizable(self, minimizable: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_minimizable(self._handle, minimizable), "window.set_minimizable")

    def set_always_on_bottom(self, always_on_bottom: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_always_on_bottom(self._handle, always_on_bottom), "window.set_always_on_bottom")

    def set_content_protected(self, protected: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_content_protected(self._handle, protected), "window.set_content_protected")

    def set_skip_taskbar(self, skip: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_skip_taskbar(self._handle, skip), "window.set_skip_taskbar")

    def set_shadow(self, enable: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_shadow(self._handle, enable), "window.set_shadow")

    def set_visible_on_all_workspaces(self, visible: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_visible_on_all_workspaces(self._handle, visible), "window.set_visible_on_all_workspaces")

    def set_ignore_cursor_events(self, ignore: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_ignore_cursor_events(self._handle, ignore), "window.set_ignore_cursor_events")

    def set_cursor_visible(self, visible: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_cursor_visible(self._handle, visible), "window.set_cursor_visible")

    def set_cursor_grab(self, grab: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_cursor_grab(self._handle, grab), "window.set_cursor_grab")

    def set_enabled(self, enabled: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_enabled(self._handle, enabled), "window.set_enabled")

    def set_focusable(self, focusable: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_focusable(self._handle, focusable), "window.set_focusable")

    def set_simple_fullscreen(self, enable: bool) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_simple_fullscreen(self._handle, enable), "window.set_simple_fullscreen")

    def set_min_size(self, width: float = 0.0, height: float = 0.0, physical: bool = False) -> None:
        """Logical pixels unless physical=True; a non-positive size clears the constraint."""
        _check(self._lib, self._lib.tauri_webview_window_set_min_size(self._handle, width, height, physical), "window.set_min_size")

    def set_max_size(self, width: float = 0.0, height: float = 0.0, physical: bool = False) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_max_size(self._handle, width, height, physical), "window.set_max_size")

    def set_cursor_position(self, x: float, y: float, physical: bool = False) -> None:
        _check(self._lib, self._lib.tauri_webview_window_set_cursor_position(self._handle, x, y, physical), "window.set_cursor_position")

    def set_theme(self, theme: Optional[str]) -> None:
        """Pass "light", "dark", or None to follow the system theme."""
        _check(self._lib, self._lib.tauri_webview_window_set_theme(self._handle, _s(theme or "")), "window.set_theme")

    def set_cursor_icon(self, icon: str) -> None:
        """e.g. "default", "pointer", "crosshair", "grab", "wait"."""
        _check(self._lib, self._lib.tauri_webview_window_set_cursor_icon(self._handle, _s(icon)), "window.set_cursor_icon")

    def request_user_attention(self, kind: Optional[str]) -> None:
        """Pass "critical", "informational", or None to cancel."""
        _check(self._lib, self._lib.tauri_webview_window_request_user_attention(self._handle, _s(kind or "")), "window.request_user_attention")

    def set_progress_bar(self, state: dict) -> None:
        """{"status": ..., "progress": ...} — see Tauri's ProgressBarState."""
        _check(self._lib, self._lib.tauri_webview_window_set_progress_bar(self._handle, _s(json.dumps(state or {}))), "window.set_progress_bar")

    def set_effects(self, effects: Optional[dict]) -> None:
        """A WindowEffectsConfig dict, or None to clear effects."""
        _check(self._lib, self._lib.tauri_webview_window_set_effects(self._handle, _s(json.dumps(effects))), "window.set_effects")

    def set_size_constraints(self, constraints: dict) -> None:
        """A WindowSizeConstraints dict, e.g. {"minWidth": {"Logical": 400}}."""
        _check(self._lib, self._lib.tauri_webview_window_set_size_constraints(self._handle, _s(json.dumps(constraints or {}))), "window.set_size_constraints")

    def set_background_color(self, r: int, g: int, b: int, a: int = 255) -> None:
        """RGBA channels 0-255."""
        _check(self._lib, self._lib.tauri_webview_window_set_background_color(self._handle, r, g, b, a), "window.set_background_color")

    def set_badge_count(self, count: Optional[int]) -> None:
        """Pass None or a negative count to clear the badge."""
        _check(self._lib, self._lib.tauri_webview_window_set_badge_count(self._handle, -1 if count is None else count), "window.set_badge_count")

    def set_badge_label(self, label: Optional[str]) -> None:
        """macOS only: the dock badge label; None clears it."""
        _check(self._lib, self._lib.tauri_webview_window_set_badge_label(self._handle, _s(label or "")), "window.set_badge_label")

    def set_title_bar_style(self, style: str) -> None:
        """macOS only: "visible", "transparent" or "overlay"."""
        _check(self._lib, self._lib.tauri_webview_window_set_title_bar_style(self._handle, _s(style)), "window.set_title_bar_style")

    def set_overlay_icon(self, rgba: Optional[bytes], width: int = 0, height: int = 0) -> None:
        """Windows only: RGBA pixels (width*height*4 bytes), or None to clear."""
        buffer = _ffi.NULL if rgba is None else rgba
        _check(self._lib, self._lib.tauri_webview_window_set_overlay_icon(self._handle, buffer, width, height), "window.set_overlay_icon")

    def ns_window(self) -> int:
        """macOS only: the NSWindow pointer, as an integer."""
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_webview_window_ns_window(self._handle, out), "window.ns_window")
        return out[0]

    def ns_view(self) -> int:
        """macOS only: the NSView pointer, as an integer."""
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_webview_window_ns_view(self._handle, out), "window.ns_view")
        return out[0]

    def hwnd(self) -> int:
        """Windows only: the window's HWND, as an integer."""
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_webview_window_hwnd(self._handle, out), "window.hwnd")
        return out[0]

    def start_dragging(self) -> None:
        self._unit(self._lib.tauri_webview_window_start_dragging, "window.start_dragging")

    def print(self) -> None:
        self._unit(self._lib.tauri_webview_window_print, "window.print")

    def clear_all_browsing_data(self) -> None:
        self._unit(self._lib.tauri_webview_window_clear_all_browsing_data, "window.clear_all_browsing_data")

    def hide_menu(self) -> None:
        self._unit(self._lib.tauri_webview_window_hide_menu, "window.hide_menu")

    def show_menu(self) -> None:
        self._unit(self._lib.tauri_webview_window_show_menu, "window.show_menu")

    def open_devtools(self) -> None:
        """Requires a debug build or the tauri-ffi ``devtools`` feature."""
        self._unit(self._lib.tauri_webview_window_open_devtools, "window.open_devtools")

    def close_devtools(self) -> None:
        self._unit(self._lib.tauri_webview_window_close_devtools, "window.close_devtools")

    def show(self) -> None:
        self._unit(self._lib.tauri_webview_window_show, "window.show")

    def hide(self) -> None:
        self._unit(self._lib.tauri_webview_window_hide, "window.hide")

    def center(self) -> None:
        self._unit(self._lib.tauri_webview_window_center, "window.center")

    def maximize(self) -> None:
        self._unit(self._lib.tauri_webview_window_maximize, "window.maximize")

    def unmaximize(self) -> None:
        self._unit(self._lib.tauri_webview_window_unmaximize, "window.unmaximize")

    def minimize(self) -> None:
        self._unit(self._lib.tauri_webview_window_minimize, "window.minimize")

    def unminimize(self) -> None:
        self._unit(self._lib.tauri_webview_window_unminimize, "window.unminimize")

    def close(self) -> None:
        self._unit(self._lib.tauri_webview_window_close, "window.close")

    def destroy(self) -> None:
        self._unit(self._lib.tauri_webview_window_destroy, "window.destroy")

    def eval(self, js: str) -> None:
        _check(self._lib, self._lib.tauri_webview_window_eval(self._handle, _s(js)), "window.eval")

    def navigate(self, url: str) -> None:
        _check(self._lib, self._lib.tauri_webview_window_navigate(self._handle, _s(url)), "window.navigate")

    def reload(self) -> None:
        self._unit(self._lib.tauri_webview_window_reload, "window.reload")

    def free(self) -> None:
        """Releases the handle; the window itself is unaffected."""
        _check(self._lib, self._lib.tauri_handle_close(self._handle), "window.free")


# (snake_case method name, PathResolver kind) for Manager::path base directories.
_PATH_KINDS = [
    ("app_config_dir", "appConfig"), ("app_data_dir", "appData"),
    ("app_local_data_dir", "appLocalData"), ("app_cache_dir", "appCache"),
    ("app_log_dir", "appLog"), ("audio_dir", "audio"), ("cache_dir", "cache"),
    ("config_dir", "config"), ("data_dir", "data"), ("local_data_dir", "localData"),
    ("desktop_dir", "desktop"), ("document_dir", "document"), ("download_dir", "download"),
    ("executable_dir", "executable"), ("font_dir", "font"), ("home_dir", "home"),
    ("picture_dir", "picture"), ("public_dir", "public"), ("resource_dir", "resource"),
    ("runtime_dir", "runtime"), ("template_dir", "template"), ("video_dir", "video"),
    ("temp_dir", "temp"),
]


class _PathResolver:
    """Mirrors ``Manager::path`` — one method per platform base directory
    (e.g. ``app.path.app_data_dir()``, ``app.path.home_dir()``)."""

    def __init__(self, app: "App"):
        self._app = app

    def _dir(self, kind: str, what: str) -> str:
        out = _ffi.new("char **")
        _check(self._app._lib, self._app._lib.tauri_app_path(self._app._app, _s(kind), out), what)
        return _take_string(self._app._lib, out) or ""


def _install_path_methods():
    for method_name, kind in _PATH_KINDS:
        def make(kind, method_name):
            def dir_method(self) -> str:
                return self._dir(kind, f"path.{method_name}")
            dir_method.__name__ = method_name
            dir_method.__qualname__ = f"_PathResolver.{method_name}"
            return dir_method

        setattr(_PathResolver, method_name, make(kind, method_name))


_install_path_methods()


class Tray:
    """Mirrors ``tauri::tray::TrayIcon``. Obtain via ``App.create_tray()``; call
    ``free()`` when done (dropping the last handle removes the icon). Menus are
    not yet exposed — set an icon, tooltip, title and listen for ``tray-event``
    via ``@app.on("tray-event")``."""

    def __init__(self, lib, handle: int):
        self._lib = lib
        self._handle = handle

    def id(self) -> str:
        out = _ffi.new("char **")
        _check(self._lib, self._lib.tauri_tray_id(self._handle, out), "tray.id")
        return _take_string(self._lib, out) or ""

    def set_icon(self, path: Optional[str]) -> None:
        """Icon from a PNG/ICO file path; pass None/'' to clear."""
        _check(self._lib, self._lib.tauri_tray_set_icon(self._handle, _s(path or "")), "tray.set_icon")

    def set_icon_as_template(self, is_template: bool) -> None:
        _check(self._lib, self._lib.tauri_tray_set_icon_as_template(self._handle, is_template), "tray.set_icon_as_template")

    def set_tooltip(self, tooltip: Optional[str]) -> None:
        _check(self._lib, self._lib.tauri_tray_set_tooltip(self._handle, _s(tooltip or "")), "tray.set_tooltip")

    def set_title(self, title: Optional[str]) -> None:
        _check(self._lib, self._lib.tauri_tray_set_title(self._handle, _s(title or "")), "tray.set_title")

    def set_visible(self, visible: bool) -> None:
        _check(self._lib, self._lib.tauri_tray_set_visible(self._handle, visible), "tray.set_visible")

    def set_show_menu_on_left_click(self, enable: bool) -> None:
        _check(self._lib, self._lib.tauri_tray_set_show_menu_on_left_click(self._handle, enable), "tray.set_show_menu_on_left_click")

    def set_menu(self, menu: Optional["Menu"]) -> None:
        """Sets (or clears, when menu is None) the tray context menu."""
        _check(self._lib, self._lib.tauri_tray_set_menu(self._handle, menu.handle if menu else 0), "tray.set_menu")

    def free(self) -> None:
        """Releases the handle; dropping the last handle removes the icon."""
        _check(self._lib, self._lib.tauri_handle_close(self._handle), "tray.free")


class Window:
    """Mirrors ``tauri::Window`` — a bare OS window (no webview) that can host
    one or more :class:`Webview` (multiwebview). For the common single-webview
    case use :class:`WebviewWindow`. Obtain via ``App.create_bare_window()``."""

    def __init__(self, lib, handle: int):
        self._lib = lib
        self._handle = handle

    @property
    def handle(self) -> int:
        return self._handle

    def _str(self, fn, what: str) -> str:
        out = _ffi.new("char **")
        _check(self._lib, fn(self._handle, out), what)
        return _take_string(self._lib, out) or ""

    def _bool(self, fn, what: str) -> bool:
        out = _ffi.new("bool *")
        _check(self._lib, fn(self._handle, out), what)
        return bool(out[0])

    def _pair(self, fn, ctype: str, what: str) -> tuple:
        a = _ffi.new(ctype)
        b = _ffi.new(ctype)
        _check(self._lib, fn(self._handle, a, b), what)
        return (a[0], b[0])

    def _act(self, fn, what: str) -> None:
        _check(self._lib, fn(self._handle), what)

    def add_webview(self, config: dict, x: float, y: float, width: float, height: float, physical: bool = False) -> "Webview":
        """Adds a child webview at the given position/size (logical unless physical=True)."""
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_window_add_webview(self._handle, _s(json.dumps(config)), x, y, width, height, physical, out), "window.add_webview")
        return Webview(self._lib, out[0])

    def webviews(self) -> list:
        return json.loads(self._str(self._lib.tauri_window_webviews, "window.webviews") or "[]")

    def label(self) -> str: return self._str(self._lib.tauri_window_label, "window.label")
    def title(self) -> str: return self._str(self._lib.tauri_window_title, "window.title")
    def theme(self) -> str: return self._str(self._lib.tauri_window_theme, "window.theme")
    def scale_factor(self) -> float:
        out = _ffi.new("double *"); _check(self._lib, self._lib.tauri_window_scale_factor(self._handle, out), "window.scale_factor"); return out[0]
    def inner_size(self) -> tuple: return self._pair(self._lib.tauri_window_inner_size, "uint32_t *", "window.inner_size")
    def outer_size(self) -> tuple: return self._pair(self._lib.tauri_window_outer_size, "uint32_t *", "window.outer_size")
    def inner_position(self) -> tuple: return self._pair(self._lib.tauri_window_inner_position, "int32_t *", "window.inner_position")
    def outer_position(self) -> tuple: return self._pair(self._lib.tauri_window_outer_position, "int32_t *", "window.outer_position")
    def cursor_position(self) -> tuple: return self._pair(self._lib.tauri_window_cursor_position, "double *", "window.cursor_position")
    def is_visible(self) -> bool: return self._bool(self._lib.tauri_window_is_visible, "window.is_visible")
    def is_focused(self) -> bool: return self._bool(self._lib.tauri_window_is_focused, "window.is_focused")
    def is_fullscreen(self) -> bool: return self._bool(self._lib.tauri_window_is_fullscreen, "window.is_fullscreen")
    def is_maximized(self) -> bool: return self._bool(self._lib.tauri_window_is_maximized, "window.is_maximized")
    def is_minimized(self) -> bool: return self._bool(self._lib.tauri_window_is_minimized, "window.is_minimized")
    def is_resizable(self) -> bool: return self._bool(self._lib.tauri_window_is_resizable, "window.is_resizable")
    def is_decorated(self) -> bool: return self._bool(self._lib.tauri_window_is_decorated, "window.is_decorated")
    def is_closable(self) -> bool: return self._bool(self._lib.tauri_window_is_closable, "window.is_closable")
    def is_maximizable(self) -> bool: return self._bool(self._lib.tauri_window_is_maximizable, "window.is_maximizable")
    def is_minimizable(self) -> bool: return self._bool(self._lib.tauri_window_is_minimizable, "window.is_minimizable")
    def is_always_on_top(self) -> bool: return self._bool(self._lib.tauri_window_is_always_on_top, "window.is_always_on_top")
    def is_enabled(self) -> bool: return self._bool(self._lib.tauri_window_is_enabled, "window.is_enabled")
    def is_menu_visible(self) -> bool: return self._bool(self._lib.tauri_window_is_menu_visible, "window.is_menu_visible")
    def available_monitors(self) -> list: return json.loads(self._str(self._lib.tauri_window_available_monitors, "window.available_monitors") or "[]")
    def current_monitor(self): return json.loads(self._str(self._lib.tauri_window_current_monitor, "window.current_monitor") or "null")
    def primary_monitor(self): return json.loads(self._str(self._lib.tauri_window_primary_monitor, "window.primary_monitor") or "null")
    def monitor_from_point(self, x: float, y: float):
        out = _ffi.new("char **"); _check(self._lib, self._lib.tauri_window_monitor_from_point(self._handle, x, y, out), "window.monitor_from_point"); return json.loads(_take_string(self._lib, out) or "null")
    def set_title(self, title: str) -> None: _check(self._lib, self._lib.tauri_window_set_title(self._handle, _s(title)), "window.set_title")
    def set_size(self, width: float, height: float, physical: bool = False) -> None: _check(self._lib, self._lib.tauri_window_set_size(self._handle, width, height, physical), "window.set_size")
    def set_position(self, x: float, y: float, physical: bool = False) -> None: _check(self._lib, self._lib.tauri_window_set_position(self._handle, x, y, physical), "window.set_position")
    def set_min_size(self, width: float = 0.0, height: float = 0.0, physical: bool = False) -> None: _check(self._lib, self._lib.tauri_window_set_min_size(self._handle, width, height, physical), "window.set_min_size")
    def set_max_size(self, width: float = 0.0, height: float = 0.0, physical: bool = False) -> None: _check(self._lib, self._lib.tauri_window_set_max_size(self._handle, width, height, physical), "window.set_max_size")
    def set_cursor_position(self, x: float, y: float, physical: bool = False) -> None: _check(self._lib, self._lib.tauri_window_set_cursor_position(self._handle, x, y, physical), "window.set_cursor_position")
    def set_fullscreen(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_fullscreen(self._handle, v), "window.set_fullscreen")
    def set_resizable(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_resizable(self._handle, v), "window.set_resizable")
    def set_always_on_top(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_always_on_top(self._handle, v), "window.set_always_on_top")
    def set_always_on_bottom(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_always_on_bottom(self._handle, v), "window.set_always_on_bottom")
    def set_decorations(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_decorations(self._handle, v), "window.set_decorations")
    def set_closable(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_closable(self._handle, v), "window.set_closable")
    def set_maximizable(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_maximizable(self._handle, v), "window.set_maximizable")
    def set_minimizable(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_minimizable(self._handle, v), "window.set_minimizable")
    def set_content_protected(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_content_protected(self._handle, v), "window.set_content_protected")
    def set_skip_taskbar(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_skip_taskbar(self._handle, v), "window.set_skip_taskbar")
    def set_shadow(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_shadow(self._handle, v), "window.set_shadow")
    def set_visible_on_all_workspaces(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_visible_on_all_workspaces(self._handle, v), "window.set_visible_on_all_workspaces")
    def set_ignore_cursor_events(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_ignore_cursor_events(self._handle, v), "window.set_ignore_cursor_events")
    def set_cursor_visible(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_cursor_visible(self._handle, v), "window.set_cursor_visible")
    def set_cursor_grab(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_cursor_grab(self._handle, v), "window.set_cursor_grab")
    def set_enabled(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_enabled(self._handle, v), "window.set_enabled")
    def set_focusable(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_focusable(self._handle, v), "window.set_focusable")
    def set_simple_fullscreen(self, v: bool) -> None: _check(self._lib, self._lib.tauri_window_set_simple_fullscreen(self._handle, v), "window.set_simple_fullscreen")
    def set_theme(self, theme: Optional[str]) -> None: _check(self._lib, self._lib.tauri_window_set_theme(self._handle, _s(theme or "")), "window.set_theme")
    def set_cursor_icon(self, icon: str) -> None: _check(self._lib, self._lib.tauri_window_set_cursor_icon(self._handle, _s(icon)), "window.set_cursor_icon")
    def request_user_attention(self, kind: Optional[str]) -> None: _check(self._lib, self._lib.tauri_window_request_user_attention(self._handle, _s(kind or "")), "window.request_user_attention")
    def set_progress_bar(self, state: dict) -> None: _check(self._lib, self._lib.tauri_window_set_progress_bar(self._handle, _s(json.dumps(state or {}))), "window.set_progress_bar")
    def set_effects(self, effects: Optional[dict]) -> None: _check(self._lib, self._lib.tauri_window_set_effects(self._handle, _s(json.dumps(effects))), "window.set_effects")
    def set_size_constraints(self, constraints: dict) -> None: _check(self._lib, self._lib.tauri_window_set_size_constraints(self._handle, _s(json.dumps(constraints or {}))), "window.set_size_constraints")
    def set_background_color(self, r: int, g: int, b: int, a: int = 255) -> None: _check(self._lib, self._lib.tauri_window_set_background_color(self._handle, r, g, b, a), "window.set_background_color")
    def set_badge_count(self, count: Optional[int]) -> None: _check(self._lib, self._lib.tauri_window_set_badge_count(self._handle, -1 if count is None else count), "window.set_badge_count")
    def set_badge_label(self, label: Optional[str]) -> None: _check(self._lib, self._lib.tauri_window_set_badge_label(self._handle, _s(label or "")), "window.set_badge_label")
    def set_title_bar_style(self, style: str) -> None: _check(self._lib, self._lib.tauri_window_set_title_bar_style(self._handle, _s(style)), "window.set_title_bar_style")
    def set_overlay_icon(self, rgba: Optional[bytes], width: int = 0, height: int = 0) -> None: _check(self._lib, self._lib.tauri_window_set_overlay_icon(self._handle, _ffi.NULL if rgba is None else rgba, width, height), "window.set_overlay_icon")
    def ns_window(self) -> int: out = _ffi.new("uint64_t *"); _check(self._lib, self._lib.tauri_window_ns_window(self._handle, out), "window.ns_window"); return out[0]
    def ns_view(self) -> int: out = _ffi.new("uint64_t *"); _check(self._lib, self._lib.tauri_window_ns_view(self._handle, out), "window.ns_view"); return out[0]
    def hwnd(self) -> int: out = _ffi.new("uint64_t *"); _check(self._lib, self._lib.tauri_window_hwnd(self._handle, out), "window.hwnd"); return out[0]
    def set_focus(self) -> None: self._act(self._lib.tauri_window_set_focus, "window.set_focus")
    def show(self) -> None: self._act(self._lib.tauri_window_show, "window.show")
    def hide(self) -> None: self._act(self._lib.tauri_window_hide, "window.hide")
    def center(self) -> None: self._act(self._lib.tauri_window_center, "window.center")
    def maximize(self) -> None: self._act(self._lib.tauri_window_maximize, "window.maximize")
    def unmaximize(self) -> None: self._act(self._lib.tauri_window_unmaximize, "window.unmaximize")
    def minimize(self) -> None: self._act(self._lib.tauri_window_minimize, "window.minimize")
    def unminimize(self) -> None: self._act(self._lib.tauri_window_unminimize, "window.unminimize")
    def close(self) -> None: self._act(self._lib.tauri_window_close, "window.close")
    def destroy(self) -> None: self._act(self._lib.tauri_window_destroy, "window.destroy")
    def start_dragging(self) -> None: self._act(self._lib.tauri_window_start_dragging, "window.start_dragging")
    def hide_menu(self) -> None: self._act(self._lib.tauri_window_hide_menu, "window.hide_menu")
    def show_menu(self) -> None: self._act(self._lib.tauri_window_show_menu, "window.show_menu")
    def set_menu(self, menu: "Menu") -> None: _check(self._lib, self._lib.tauri_menu_set_as_window_menu(self._handle, menu.handle), "window.set_menu")
    def free(self) -> None: _check(self._lib, self._lib.tauri_handle_close(self._handle), "window.free")


class Webview:
    """Mirrors ``tauri::Webview`` — a webview hosted inside a :class:`Window`."""

    def __init__(self, lib, handle: int):
        self._lib = lib
        self._handle = handle

    @property
    def handle(self) -> int:
        return self._handle

    def _str(self, fn, what: str) -> str:
        out = _ffi.new("char **")
        _check(self._lib, fn(self._handle, out), what)
        return _take_string(self._lib, out) or ""

    def _pair(self, fn, ctype: str, what: str) -> tuple:
        a = _ffi.new(ctype); b = _ffi.new(ctype)
        _check(self._lib, fn(self._handle, a, b), what)
        return (a[0], b[0])

    def label(self) -> str: return self._str(self._lib.tauri_webview_label, "webview.label")
    def url(self) -> str: return self._str(self._lib.tauri_webview_url, "webview.url")
    def position(self) -> tuple: return self._pair(self._lib.tauri_webview_position, "int32_t *", "webview.position")
    def size(self) -> tuple: return self._pair(self._lib.tauri_webview_size, "uint32_t *", "webview.size")
    def window(self) -> Window:
        out = _ffi.new("uint64_t *"); _check(self._lib, self._lib.tauri_webview_get_window(self._handle, out), "webview.window"); return Window(self._lib, out[0])
    def eval(self, js: str) -> None: _check(self._lib, self._lib.tauri_webview_eval(self._handle, _s(js)), "webview.eval")
    def navigate(self, url: str) -> None: _check(self._lib, self._lib.tauri_webview_navigate(self._handle, _s(url)), "webview.navigate")
    def reload(self) -> None: _check(self._lib, self._lib.tauri_webview_reload(self._handle), "webview.reload")
    def print(self) -> None: _check(self._lib, self._lib.tauri_webview_print(self._handle), "webview.print")
    def set_zoom(self, scale: float) -> None: _check(self._lib, self._lib.tauri_webview_set_zoom(self._handle, scale), "webview.set_zoom")
    def set_auto_resize(self, v: bool) -> None: _check(self._lib, self._lib.tauri_webview_set_auto_resize(self._handle, v), "webview.set_auto_resize")
    def set_size(self, width: float, height: float, physical: bool = False) -> None: _check(self._lib, self._lib.tauri_webview_set_size(self._handle, width, height, physical), "webview.set_size")
    def set_position(self, x: float, y: float, physical: bool = False) -> None: _check(self._lib, self._lib.tauri_webview_set_position(self._handle, x, y, physical), "webview.set_position")
    def set_background_color(self, r: int, g: int, b: int, a: int = 255) -> None: _check(self._lib, self._lib.tauri_webview_set_background_color(self._handle, r, g, b, a), "webview.set_background_color")
    def set_focus(self) -> None: _check(self._lib, self._lib.tauri_webview_set_focus(self._handle), "webview.set_focus")
    def show(self) -> None: _check(self._lib, self._lib.tauri_webview_show(self._handle), "webview.show")
    def hide(self) -> None: _check(self._lib, self._lib.tauri_webview_hide(self._handle), "webview.hide")
    def close(self) -> None: _check(self._lib, self._lib.tauri_webview_close(self._handle), "webview.close")
    def clear_all_browsing_data(self) -> None: _check(self._lib, self._lib.tauri_webview_clear_all_browsing_data(self._handle), "webview.clear_all_browsing_data")
    def reparent(self, window: Window) -> None: _check(self._lib, self._lib.tauri_webview_reparent(self._handle, window.handle), "webview.reparent")
    def open_devtools(self) -> None: _check(self._lib, self._lib.tauri_webview_open_devtools(self._handle), "webview.open_devtools")
    def close_devtools(self) -> None: _check(self._lib, self._lib.tauri_webview_close_devtools(self._handle), "webview.close_devtools")
    def is_devtools_open(self) -> bool:
        out = _ffi.new("bool *"); _check(self._lib, self._lib.tauri_webview_is_devtools_open(self._handle, out), "webview.is_devtools_open"); return bool(out[0])
    def free(self) -> None: _check(self._lib, self._lib.tauri_handle_close(self._handle), "webview.free")


class MenuItem:
    """A menu item (``tauri::menu::MenuItemKind``). Menu clicks arrive via
    ``@app.on("menu-event")`` with the item id."""

    def __init__(self, lib, handle: int):
        self._lib = lib
        self._handle = handle

    @property
    def handle(self) -> int:
        return self._handle

    def id(self) -> str:
        out = _ffi.new("char **"); _check(self._lib, self._lib.tauri_menu_item_id(self._handle, out), "menu_item.id"); return _take_string(self._lib, out) or ""
    def set_text(self, text: str) -> None: _check(self._lib, self._lib.tauri_menu_item_set_text(self._handle, _s(text)), "menu_item.set_text")
    def set_enabled(self, v: bool) -> None: _check(self._lib, self._lib.tauri_menu_item_set_enabled(self._handle, v), "menu_item.set_enabled")
    def set_checked(self, v: bool) -> None: _check(self._lib, self._lib.tauri_menu_item_set_checked(self._handle, v), "menu_item.set_checked")
    def set_accelerator(self, accel: Optional[str]) -> None: _check(self._lib, self._lib.tauri_menu_item_set_accelerator(self._handle, _s(accel or "")), "menu_item.set_accelerator")
    def append(self, item: "MenuItem") -> None:
        """Appends a child (only valid on a submenu)."""
        _check(self._lib, self._lib.tauri_submenu_append(self._handle, item.handle), "submenu.append")
    def free(self) -> None: _check(self._lib, self._lib.tauri_handle_close(self._handle), "menu_item.free")


class Menu:
    """A menu (``tauri::menu::Menu``). Attach with ``App.set_app_menu()``,
    ``Window.set_menu()`` or ``Tray.set_menu()``."""

    def __init__(self, lib, handle: int):
        self._lib = lib
        self._handle = handle

    @property
    def handle(self) -> int:
        return self._handle

    def append(self, item: MenuItem) -> None:
        _check(self._lib, self._lib.tauri_menu_append(self._handle, item.handle), "menu.append")
    def free(self) -> None: _check(self._lib, self._lib.tauri_handle_close(self._handle), "menu.free")


class App:
    """Register commands/handlers, then call ``run()`` from the main thread.

    ``config`` is optional: when omitted, a ``tauri.conf.json`` next to the
    main module (or in the working directory) is used. The ``TAURI_CONFIG``
    environment variable (set by the Tauri CLI) is deep-merged on top in
    either case, and ``TAURI_DEV`` enables dev mode.

    ``capabilities`` are merged with any files found in a ``capabilities/``
    directory next to the config (mirroring a Rust app's compile-time
    capability discovery); when none are supplied, core:default is granted to
    all windows."""

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
        # When frozen, assets/config/capabilities are embedded *inside* the
        # binary (extracted to sys._MEIPASS); in dev they come from disk.
        # Explicit arguments always win.
        embedded = _embedded_dir()

        if config is None and embedded is not None:
            cfg_file = embedded / "config.json"
            if not cfg_file.is_file():
                raise TauriError(-1, "bundled app is missing its embedded config.json")
            self._config = json.loads(cfg_file.read_text())
            config_dir = None
        else:
            self._config, config_dir = _resolve_config(config)

        if embedded is not None and assets_dir is None and assets_archive is None:
            archive = embedded / "app.assets"
            self._assets_dir, self._assets_archive = None, (archive if archive.is_file() else None)
        else:
            self._assets_dir, self._assets_archive = _resolve_assets(
                assets_dir, assets_archive, self._config, config_dir
            )

        # Inline capabilities plus those embedded in the binary (frozen) or
        # discovered in a `capabilities/` directory next to the config (dev) —
        # compile-time capability files, read at construction.
        entry_dir = Path(sys.argv[0]).resolve().parent if sys.argv and sys.argv[0] else None
        cap_file = embedded / "capabilities.json" if embedded is not None else None
        if cap_file is not None and cap_file.is_file():
            discovered = json.loads(cap_file.read_text())
        else:
            discovered = _resolve_capabilities([config_dir, entry_dir])
        # a frozen bundle ignores the TAURI_DEV env override (hermetic in production)
        if dev is not None:
            self._dev = dev
        else:
            self._dev = not _is_bundled() and os.environ.get("TAURI_DEV") == "true"
        self._capabilities = list(capabilities or []) + discovered
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
            self._lib.tauri_webview_window_create(self._app, _s(json.dumps(config)), out),
            f"create_window({config.get('label')})",
        )
        return WebviewWindow(self._lib, out[0])

    def get_window(self, label: str) -> Optional[WebviewWindow]:
        out = _ffi.new("uint64_t *")
        code = self._lib.tauri_app_get_webview_window(self._app, _s(label), out)
        if code == CODES["NOT_FOUND"]:
            return None
        _check(self._lib, code, f"get_window({label})")
        return WebviewWindow(self._lib, out[0])

    def window_labels(self) -> list:
        out = _ffi.new("char **")
        _check(self._lib, self._lib.tauri_app_webview_window_labels(self._app, out), "window_labels")
        return json.loads(_take_string(self._lib, out) or "[]")

    def config(self) -> dict:
        """The app's resolved configuration (tauri.conf.json shape)."""
        out = _ffi.new("char **")
        _check(self._lib, self._lib.tauri_app_config(self._app, out), "config")
        return json.loads(_take_string(self._lib, out) or "null")

    def package_info(self) -> dict:
        """The app package info: name, version, authors, description, crateName."""
        out = _ffi.new("char **")
        _check(self._lib, self._lib.tauri_app_package_info(self._app, out), "package_info")
        return json.loads(_take_string(self._lib, out) or "null")

    @property
    def path(self) -> _PathResolver:
        """Platform base directories (``Manager::path``), e.g.
        ``app.path.app_data_dir()``."""
        return _PathResolver(self)

    def get_focused_window(self) -> Optional[WebviewWindow]:
        """The focused webview window, or None."""
        out = _ffi.new("uint64_t *")
        code = self._lib.tauri_app_get_focused_window(self._app, out)
        if code == CODES["NOT_FOUND"]:
            return None
        _check(self._lib, code, "get_focused_window")
        return WebviewWindow(self._lib, out[0])

    def add_capability(self, capability) -> None:
        """Adds a capability (JSON/TOML string or dict) to the running app."""
        text = capability if isinstance(capability, str) else json.dumps(capability)
        _check(self._lib, self._lib.tauri_app_add_capability(self._app, _s(text)), "add_capability")

    def once(self, event: str, handler: Callable) -> int:
        """Like listen(), but auto-removed after the first delivery."""
        out = _ffi.new("uint32_t *")
        _check(self._lib, self._lib.tauri_app_once(self._app, _s(event), out), f"once({event})")
        listener_id = out[0]

        def wrapper(payload, message):
            self._listeners.pop(listener_id, None)
            handler(payload, message)

        self._listeners[listener_id] = wrapper
        return listener_id

    def set_theme(self, theme: Optional[str]) -> None:
        """Sets the app-wide theme: "light", "dark", or None to follow the system."""
        _check(self._lib, self._lib.tauri_app_set_theme(self._app, _s(theme or "")), "set_theme")

    def cursor_position(self) -> tuple:
        """(x, y) cursor position in physical screen pixels."""
        x = _ffi.new("double *")
        y = _ffi.new("double *")
        _check(self._lib, self._lib.tauri_app_cursor_position(self._app, x, y), "cursor_position")
        return (x[0], y[0])

    def request_restart(self) -> None:
        """Requests an app restart (exits and relaunches)."""
        _check(self._lib, self._lib.tauri_app_request_restart(self._app), "request_restart")

    def set_activation_policy(self, policy: str) -> None:
        """macOS only: "regular", "accessory" (no Dock icon) or "prohibited"."""
        _check(self._lib, self._lib.tauri_app_set_activation_policy(self._app, _s(policy)), "set_activation_policy")

    def set_dock_visibility(self, visible: bool) -> None:
        """macOS only: shows or hides the app's Dock icon."""
        _check(self._lib, self._lib.tauri_app_set_dock_visibility(self._app, visible), "set_dock_visibility")

    def show(self) -> None:
        """macOS only: shows the application without focusing it."""
        _check(self._lib, self._lib.tauri_app_show(self._app), "show")

    def hide(self) -> None:
        """macOS only: hides the application, like Cmd+H."""
        _check(self._lib, self._lib.tauri_app_hide(self._app), "hide")

    def create_tray(self, id: str = "") -> Tray:
        """Creates a system tray icon. Listen via ``@app.on("tray-event")``."""
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_tray_new(self._app, _s(id), out), f"create_tray({id})")
        return Tray(self._lib, out[0])

    def remove_tray_by_id(self, id: str) -> None:
        _check(self._lib, self._lib.tauri_app_remove_tray_by_id(self._app, _s(id)), "remove_tray_by_id")

    def create_bare_window(self, config: dict) -> Window:
        """Creates a bare OS window (no webview) that can host webviews."""
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_window_create(self._app, _s(json.dumps(config)), out), f"create_bare_window({config.get('label')})")
        return Window(self._lib, out[0])

    def get_bare_window(self, label: str) -> Optional[Window]:
        out = _ffi.new("uint64_t *")
        code = self._lib.tauri_app_get_window(self._app, _s(label), out)
        if code == CODES["NOT_FOUND"]:
            return None
        _check(self._lib, code, f"get_bare_window({label})")
        return Window(self._lib, out[0])

    def bare_window_labels(self) -> list:
        out = _ffi.new("char **")
        _check(self._lib, self._lib.tauri_app_window_labels(self._app, out), "bare_window_labels")
        return json.loads(_take_string(self._lib, out) or "[]")

    def create_menu(self) -> Menu:
        """Creates an empty menu."""
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_menu_new(self._app, out), "create_menu")
        return Menu(self._lib, out[0])

    def menu_item(self, text: str, *, id: str = "", enabled: bool = True, accelerator: str = "") -> MenuItem:
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_menu_item_new(self._app, _s(id), _s(text), enabled, _s(accelerator), out), "menu_item")
        return MenuItem(self._lib, out[0])

    def check_menu_item(self, text: str, *, id: str = "", enabled: bool = True, checked: bool = False, accelerator: str = "") -> MenuItem:
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_menu_check_item_new(self._app, _s(id), _s(text), enabled, checked, _s(accelerator), out), "check_menu_item")
        return MenuItem(self._lib, out[0])

    def predefined_menu_item(self, kind: str, text: str = "") -> MenuItem:
        """A predefined item ('separator', 'copy', 'quit', …)."""
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_menu_predefined_item_new(self._app, _s(kind), _s(text), out), "predefined_menu_item")
        return MenuItem(self._lib, out[0])

    def submenu(self, text: str, *, id: str = "", enabled: bool = True) -> MenuItem:
        out = _ffi.new("uint64_t *")
        _check(self._lib, self._lib.tauri_submenu_new(self._app, _s(id), _s(text), enabled, out), "submenu")
        return MenuItem(self._lib, out[0])

    def set_app_menu(self, menu: Menu) -> None:
        """Sets a menu as the app-wide menu (macOS menu bar)."""
        _check(self._lib, self._lib.tauri_menu_set_as_app_menu(menu.handle), "set_app_menu")

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
