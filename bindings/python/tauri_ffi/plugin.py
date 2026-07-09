# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

"""A packageable plugin wrapper, mirroring ``tauri::plugin::Builder``. A plugin
bundles a name, an optional JavaScript init script (injected into every webview)
and a set of commands with their handlers. Ship one from its own module/package;
this module has no dependencies, so a plugin need not import the FFI layer.

    from tauri_ffi import Plugin

    greet = Plugin("greet").init_script(
        "window.greet = { hello: (n) => window.__TAURI__.core.invoke('plugin:greet|hello', { name: n }) }"
    )

    @greet.command("hello")
    def hello(payload, _message):
        return f"Hello {payload['name']}!"

The app registers it with ``app.plugin(greet)``; the ``plugin:<name>|<command>``
wire format never leaks into app code.
"""

from __future__ import annotations

from typing import Callable, Optional


class Plugin:
    """A Tauri plugin definition. Register with ``App.plugin()``."""

    def __init__(self, name: str):
        if not name:
            raise ValueError("a plugin needs a name")
        self.name = name
        self.script: Optional[str] = None
        self.commands: dict[str, Callable] = {}

    def init_script(self, js: str) -> "Plugin":
        """JavaScript injected into every webview before page scripts run.
        Assign globals to ``window``; commonly used to install the plugin's
        frontend API. Chainable."""
        self.script = js
        return self

    def command(self, name: str):
        """Decorator registering a command: ``handler(payload, message)``; the
        return value (or raised error) resolves/rejects the frontend invoke."""

        def register(handler):
            self.commands[name] = handler
            return handler

        return register
