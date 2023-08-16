// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { EventCallback, EventName, UnlistenFn, listen, once } from './event'
import { ContextMenu } from './menu'
import { invoke } from './tauri'

/**
 * Describes a tray event emitted when a tray icon is clicked
 *
 * #### Platform-specific:
 *
 * - **Linux**: Unsupported. The event is not emmited even though the icon is shown,
 * the icon will still show a context menu on right click.
 */
export interface TrayIconEvent {
  /** Id of the tray icon which triggered this event. */
  id: string
  /** Physical X Position of the click the triggered this event. */
  x: number
  /** Physical Y Position of the click the triggered this event. */
  y: number
  /** Position and size of the tray icon. */
  iconRect: {
    /** The x-coordinate of the upper-left corner of the rectangle. */
    left: number
    /** The y-coordinate of the upper-left corner of the rectangle. */
    top: number
    /** The x-coordinate of the lower-right corner of the rectangle. */
    right: number
    /** The y-coordinate of the lower-right corner of the rectangle. */
    bottom: number
  }
  /** The click type that triggered this event. */
  clickType: 'Left' | 'Right' | 'Double'
}

/**
 * Tray icon types and utilities.
 *
 * This package is also accessible with `window.__TAURI__.tray` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */

/** {@link TrayIcon.new|`TrayIcon`} creation options */
export interface TrayIconOptions {
  /** The tray icon id. If undefined, a random one will be assigend */
  id?: string
  /** The tray icon menu */
  menu?: ContextMenu
  /**
   * The tray icon which could be icon bytes or path to the icon file.
   *
   * Note that you need the `icon-ico` or `icon-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "icon-png"] }
   * ```
   */
  icon?: string | Uint8Array
  /** The tray icon tooltip */
  tooltip?: string
  /**
   * The tooltip text
   *
   * #### Platform-specific
   *
   * - **Linux:** The title will not be shown unless there is an icon
   * as well.  The title is useful for numerical and other frequently
   * updated information.  In general, it shouldn't be shown unless a
   * user requests it as it can take up a significant amount of space
   * on the user's panel.  This may not be shown in all visualizations.
   * - **Windows:** Unsupported.
   */
  title?: string
  /**
   * The tray icon temp dir path. **Linux only**.
   *
   * On Linux, we need to write the icon to the disk and usually it will
   * be `$XDG_RUNTIME_DIR/tray-icon` or `$TEMP/tray-icon`.
   */
  tempDirPath?: string
  /**
   * Use the icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only**.
   */
  iconAsTemplate?: boolean
  /** Whether to show the tray menu on left click or not, default is `true`. **macOS only**. */
  menuOnLeftClieck?: boolean
}

/**
 * Tray icon class and associated methods. This type constructor is private,
 * instead, you should use the static method {@linkcode TrayIcon.new}.
 *
 * #### Warning
 *
 * Unlike Rust, javascript does not have any way to run cleanup code
 * when an object is being removed by garbage collection, but this tray icon
 * will be cleaned up when the tauri app exists, however if you want to cleanup
 * this object early, you need to call {@linkcode TrayIcon.destroy}.
 *
 * @example
 * ```ts
 * import { TrayIcon } from '@tauri-apps/api/tray';
 * const tray = await TrayIcon.new({ tooltip: 'awesome tray tooltip' });
 * tray.set_tooltip('new tooltip');
 * ```
 */
export class TrayIcon {
  _rid: number
  _id: string

  private constructor(rid: number, id: string) {
    this._rid = rid
    this._id = id
  }

  /**
   * Creates a new {@linkcode TrayIcon}
   *
   * #### Platform-specific:
   *
   * - **Linux:** Sometimes the icon won't be visible unless a menu is set.
   * Setting an empty {@linkcode Menu} is enough.
   */
  static async new(options?: TrayIconOptions): Promise<TrayIcon> {
    if (options) {
      // @ts-expect-error we only need the rid
      options.menu = options.menu?.rid
    }
    return invoke<[number, string]>('plugin:tray|new', {
      options: options ?? {}
    }).then(([rid, id]) => new TrayIcon(rid, id))
  }

  /** Returns the id associated with this tray icon.   */
  id(): string {
    return this._id
  }

  /** Sets a new tray icon. If `null` is provided, it will remove the icon. */
  async setIcon(icon: string | Uint8Array | null) {
    return invoke('plugin:tray|set_icon', { rid: this._rid, icon })
  }

  /**
   * Sets a new tray menu.
   *
   * #### Platform-specific:
   *
   * - **Linux**: once a menu is set it cannot be removed so `null` has no effect
   */
  async setMenu(menu: ContextMenu | null) {
    return invoke('plugin:tray|set_menu', { rid: this._rid, menu: menu?.rid })
  }

  /**
   * Sets the tooltip for this tray icon.
   *
   * ## Platform-specific:
   *
   * - **Linux:** Unsupported
   */
  async setTooltip(tooltip: string | null) {
    return invoke('plugin:tray|set_tooltip', { rid: this._rid, tooltip })
  }

  /**
   * Sets the tooltip for this tray icon.
   *
   * ## Platform-specific:
   *
   * - **Linux:** The title will not be shown unless there is an icon
   * as well.  The title is useful for numerical and other frequently
   * updated information.  In general, it shouldn't be shown unless a
   * user requests it as it can take up a significant amount of space
   * on the user's panel.  This may not be shown in all visualizations.
   * - **Windows:** Unsupported
   */
  async setTitle(title: string | null) {
    return invoke('plugin:tray|set_title', { rid: this._rid, title })
  }

  /** Show or hide this tray icon. */
  async setVisible(visible: boolean) {
    return invoke('plugin:tray|set_visible', { rid: this._rid, visible })
  }

  /**
   * Sets the tray icon temp dir path. **Linux only**.
   *
   * On Linux, we need to write the icon to the disk and usually it will
   * be `$XDG_RUNTIME_DIR/tray-icon` or `$TEMP/tray-icon`.
   */
  async setTempDirPath(path: string | null) {
    return invoke('plugin:tray|set_temp_dir_path', { rid: this._rid, path })
  }

  /** Sets the current icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only** */
  async setIconAsTemplate(asTemplate: boolean) {
    return invoke('plugin:tray|set_icon_as_template', {
      rid: this._rid,
      asTemplate
    })
  }

  /** Disable or enable showing the tray menu on left click. **macOS only**. */
  async setMenuOnLeftClick(onLeft: boolean) {
    return invoke('plugin:tray|set_show_menu_on_left_click', {
      rid: this._rid,
      onLeft
    })
  }

  /**
   *  Destroys and removes this tray icon from the system tray. See {@linkcode TrayIcon} docs for more info.
   *
   * #### Warning
   *
   * You should not call any method on this object anymore and should drop any reference to it,
   * otherwise the methods will throw errors when used.
   */
  async destroy() {
    return invoke('plugin:tray|destroy', {
      rid: this._rid
    })
  }

  /**
   * Listen to an event emitted by the backend that is tied to the current webview window.
   *
   * @example
   * ```typescript
   * import { TrayIcon } from '@tauri-apps/api/tray';
   * let tray = await TrayIcon.new();
   * const unlisten = await tray.listen<string>('state-changed', (event) => {
   *   console.log(`Got error: ${payload}`);
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param handler Event handler.
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 2.0.0
   */
  async listen<T>(
    event: EventName,
    handler: EventCallback<T>
  ): Promise<UnlistenFn> {
    return listen(event, handler, {
      target: window.__TAURI_METADATA__.__currentWindow.label
    })
  }

  /**
   * Listen to an one-off event emitted by the backend that is tied to the current webview window.
   *
   * @example
   * ```typescript
   * import { TrayIcon } from '@tauri-apps/api/tray';
   * let tray = await TrayIcon.new();
   * const unlisten = await tray.once<string>('state-initalized', (event) => {
   *   console.log(`State initalized`);
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param handler Event handler.
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 2.0.0
   */
  async once<T>(
    event: EventName,
    handler: EventCallback<T>
  ): Promise<UnlistenFn> {
    return once(event, handler, {
      target: window.__TAURI_METADATA__.__currentWindow.label
    })
  }

  /**
   * Listen to this tray icon events.
   *
   * @example
   * ```typescript
   * import { TrayIcon } from '@tauri-apps/api/tray';
   * let tray = await TrayIcon.new();
   * const unlisten = await tray.onTrayIconEvent((event) => {
   *   console.log(event)
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @param handler Event handler.
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 2.0.0
   */
  async onTrayIconEvent(
    handler: EventCallback<TrayIconEvent>
  ): Promise<UnlistenFn> {
    return listen('tauri://tray', handler, {
      target: window.__TAURI_METADATA__.__currentWindow.label
    })
  }
}
