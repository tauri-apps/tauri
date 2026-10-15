#!/usr/bin/env python3
# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

"""A minimal StatusNotifierWatcher for headless test sessions.

Tauri's Linux tray icon is a StatusNotifierItem: creating one registers it with
whatever owns `org.kde.StatusNotifierWatcher` on the session bus, and fails when
nothing does, which is the case under Xvfb with a bare window manager. This
service accepts registrations so an app that creates its tray icon at startup
can run there. It renders nothing.

Needs PyGObject (`python3-gi` on Debian/Ubuntu). Run it in the background on
the session bus the app will use, e.g.:

    dbus-run-session -- bash -c 'python3 .scripts/ci/sni-watcher.py & ...'

It exits with status 1 if the name is already owned (a desktop tray is running).
"""

import sys

import gi

gi.require_version("Gio", "2.0")
from gi.repository import Gio, GLib  # noqa: E402

BUS_NAME = "org.kde.StatusNotifierWatcher"
OBJECT_PATH = "/StatusNotifierWatcher"

INTROSPECTION_XML = """
<node>
  <interface name="org.kde.StatusNotifierWatcher">
    <method name="RegisterStatusNotifierItem">
      <arg name="service" type="s" direction="in"/>
    </method>
    <method name="RegisterStatusNotifierHost">
      <arg name="service" type="s" direction="in"/>
    </method>
    <property name="RegisteredStatusNotifierItems" type="as" access="read"/>
    <property name="IsStatusNotifierHostRegistered" type="b" access="read"/>
    <property name="ProtocolVersion" type="i" access="read"/>
    <signal name="StatusNotifierItemRegistered">
      <arg name="service" type="s"/>
    </signal>
    <signal name="StatusNotifierItemUnregistered">
      <arg name="service" type="s"/>
    </signal>
    <signal name="StatusNotifierHostRegistered"/>
  </interface>
</node>
"""

# Registered items ("<bus name><object path>") mapped to the id of the name
# watch that drops them when their owner leaves the bus.
items = {}
exit_code = 0
loop = GLib.MainLoop()


def emit(connection, signal_name, service):
    connection.emit_signal(
        None, OBJECT_PATH, BUS_NAME, signal_name, GLib.Variant("(s)", (service,))
    )


def unregister(connection, item):
    watch = items.pop(item, None)
    if watch is None:
        return
    Gio.bus_unwatch_name(watch)
    emit(connection, "StatusNotifierItemUnregistered", item)


def register(connection, sender, service):
    # An item passes either its bus name (its object is /StatusNotifierItem) or
    # an object path (its bus name is the caller's).
    if service.startswith("/"):
        owner, item = sender, f"{sender}{service}"
    else:
        owner, item = service, f"{service}/StatusNotifierItem"
    if item in items:
        return
    items[item] = Gio.bus_watch_name_on_connection(
        connection,
        owner,
        Gio.BusNameWatcherFlags.NONE,
        None,
        lambda conn, _name: unregister(conn, item),
    )
    emit(connection, "StatusNotifierItemRegistered", item)


def on_method_call(
    connection, sender, _path, _interface, method, parameters, invocation
):
    if method == "RegisterStatusNotifierItem":
        (service,) = parameters.unpack()
        register(connection, sender, service)
        invocation.return_value(None)
    elif method == "RegisterStatusNotifierHost":
        # Hosts are what would render the items; there is none here, but the
        # spec's reference implementations report one regardless (see
        # IsStatusNotifierHostRegistered), so accept the call.
        invocation.return_value(None)
    else:
        invocation.return_dbus_error(
            "org.freedesktop.DBus.Error.UnknownMethod", f"unknown method {method}"
        )


def on_get_property(_connection, _sender, _path, _interface, name):
    if name == "RegisteredStatusNotifierItems":
        return GLib.Variant("as", list(items))
    if name == "IsStatusNotifierHostRegistered":
        # Items refuse to register (ksni: "no StatusNotifierHost exists") when
        # this is false; KDE and GNOME hardcode it to true as well.
        return GLib.Variant("b", True)
    if name == "ProtocolVersion":
        return GLib.Variant("i", 0)
    return None


def on_bus_acquired(connection, _name):
    node = Gio.DBusNodeInfo.new_for_xml(INTROSPECTION_XML)
    connection.register_object(
        OBJECT_PATH, node.interfaces[0], on_method_call, on_get_property, None
    )


def on_name_lost(_connection, name):
    global exit_code
    print(f"sni-watcher: could not own {name} on the session bus", file=sys.stderr)
    exit_code = 1
    loop.quit()


def main():
    Gio.bus_own_name(
        Gio.BusType.SESSION,
        BUS_NAME,
        Gio.BusNameOwnerFlags.NONE,
        on_bus_acquired,
        None,
        on_name_lost,
    )
    loop.run()
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
