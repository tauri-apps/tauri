# Tauri Event System Specification

## Emitters

Emitters can emit to any and all listeners.

- `App` and `AppHandle`
- `Window`
- `Webview`
- `WebviewWindow`
- Any type that implements `Manager` trait.

## Emit functions

- `emit`: emits an event to all listeners.
- `emit_to`: emits an event to a specified target.
- `emit_filter`: emits an event to targets based on a filtering callback.

## Listeners

Emitters can emit to any and all listeners.

- `App` and `AppHandle`
- `Window`
- `Webview`
- `WebviewWindow`
- Any type that implements `Manager` trait but is limited to only using `listen_any/once_any`.

## Listen functions

- `listen`: Listens to all events targeting this listener type only.
- `once`: Listens to a single event targeting this listener type only.
- `listen_any` (available only through `Manager` trait): Listens to all events to any target (aka event sniffer).
- `once_any` (available only through `Manager` trait): Listens to a single event to any target (aka event sniffer).
