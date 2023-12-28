// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import type { Menu, Submenu } from './menu'
import { Channel, invoke, Resource } from './core'

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
  menu?: Menu | Submenu
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
  icon?: string | Uint8Array | number[]
  /** The tray icon tooltip */
  tooltip?: string
  /**
   * The tray title
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
  menuOnLeftClick?: boolean
  /** A handler for an event on the tray icon. */
  action?: (event: TrayIconEvent) => void
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
 * this object early, you need to call {@linkcode TrayIcon.close}.
 *
 * @example
 * ```ts
 * import { TrayIcon } from '@tauri-apps/api/tray';
 * const tray = await TrayIcon.new({ tooltip: 'awesome tray tooltip' });
 * tray.set_tooltip('new tooltip');
 * ```
 */
export class TrayIcon extends Resource {
  /** The id associated with this tray icon.   */
  public id: string

  private constructor(rid: number, id: string) {
    super(rid)
    this.id = id
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
    if (options?.menu) {
      // @ts-expect-error we only need the rid and kind
      options.menu = [options.menu.rid, options.menu.kind]
    }
    if (options?.icon) {
      options.icon =
        typeof options.icon === 'string'
          ? options.icon
          : Array.from(options.icon)
    }

    const handler = new Channel<TrayIconEvent>()
    if (options?.action) {
      handler.onmessage = options.action
      delete options.action
    }

    return invoke<[number, string]>('plugin:tray|new', {
      options: options ?? {},
      handler
    }).then(([rid, id]) => new TrayIcon(rid, id))
  }

  /** Sets a new tray icon. If `null` is provided, it will remove the icon. */
  async setIcon(icon: string | Uint8Array | null): Promise<void> {
    let trayIcon = null
    if (icon) {
      trayIcon = typeof icon === 'string' ? icon : Array.from(icon)
    }
    return invoke('plugin:tray|set_icon', { rid: this.rid, icon: trayIcon })
  }

  /**
   * Sets a new tray menu.
   *
   * #### Platform-specific:
   *
   * - **Linux**: once a menu is set it cannot be removed so `null` has no effect
   */
  async setMenu(menu: Menu | Submenu | null): Promise<void> {
    if (menu) {
      // @ts-expect-error we only need the rid and kind
      menu = [menu.rid, menu.kind]
    }
    return invoke('plugin:tray|set_menu', { rid: this.rid, menu })
  }

  /**
   * Sets the tooltip for this tray icon.
   *
   * ## Platform-specific:
   *
   * - **Linux:** Unsupported
   */
  async setTooltip(tooltip: string | null): Promise<void> {
    return invoke('plugin:tray|set_tooltip', { rid: this.rid, tooltip })
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
  async setTitle(title: string | null): Promise<void> {
    return invoke('plugin:tray|set_title', { rid: this.rid, title })
  }

  /** Show or hide this tray icon. */
  async setVisible(visible: boolean): Promise<void> {
    return invoke('plugin:tray|set_visible', { rid: this.rid, visible })
  }

  /**
   * Sets the tray icon temp dir path. **Linux only**.
   *
   * On Linux, we need to write the icon to the disk and usually it will
   * be `$XDG_RUNTIME_DIR/tray-icon` or `$TEMP/tray-icon`.
   */
  async setTempDirPath(path: string | null): Promise<void> {
    return invoke('plugin:tray|set_temp_dir_path', { rid: this.rid, path })
  }

  /** Sets the current icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only** */
  async setIconAsTemplate(asTemplate: boolean): Promise<void> {
    return invoke('plugin:tray|set_icon_as_template', {
      rid: this.rid,
      asTemplate
    })
  }

  /** Disable or enable showing the tray menu on left click. **macOS only**. */
  async setMenuOnLeftClick(onLeft: boolean): Promise<void> {
    return invoke('plugin:tray|set_show_menu_on_left_click', {
      rid: this.rid,
      onLeft
    })
  }
}
