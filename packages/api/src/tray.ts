// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import type { Menu, Submenu } from './menu'
import { Channel, invoke, Resource } from './core'
import { Image, transformImage } from './image'
import { PhysicalPosition, PhysicalSize } from './dpi'

export type MouseButtonState = 'Up' | 'Down'
export type MouseButton = 'Left' | 'Right' | 'Middle'
export type TrayIconEventType =
  | 'Click'
  | 'DoubleClick'
  | 'Enter'
  | 'Move'
  | 'Leave'

export type TrayIconEventBase<T extends TrayIconEventType> = {
  /** The tray icon event type */
  type: T
  /** Id of the tray icon which triggered this event. */
  id: string
  /** Physical position of the click the triggered this event. */
  position: PhysicalPosition
  /** Position and size of the tray icon. */
  rect: {
    position: PhysicalPosition
    size: PhysicalSize
  }
}

export type TrayIconClickEvent = {
  /** Mouse button that triggered this event. */
  button: MouseButton
  /** Mouse button state when this event was triggered. */
  buttonState: MouseButtonState
}

/**
 * Describes a tray icon event.
 *
 * #### Platform-specific:
 *
 * - **Linux**: Unsupported. The event is not emitted even though the icon is shown,
 * the icon will still show a context menu on right click.
 */
export type TrayIconEvent =
  | (TrayIconEventBase<'Click'> & TrayIconClickEvent)
  | (TrayIconEventBase<'DoubleClick'> & Omit<TrayIconClickEvent, 'buttonState'>)
  | TrayIconEventBase<'Enter'>
  | TrayIconEventBase<'Move'>
  | TrayIconEventBase<'Leave'>

type RustTrayIconEvent = Omit<TrayIconEvent, 'rect'> & {
  rect: {
    position: {
      Physical: { x: number; y: number }
    }
    size: {
      Physical: { width: number; height: number }
    }
  }
}

/**
 * Tray icon types and utilities.
 *
 * This package is also accessible with `window.__TAURI__.tray` when [`app.withGlobalTauri`](https://v2.tauri.app/reference/config/#withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */

/** {@link TrayIcon.new|`TrayIcon`} creation options */
export interface TrayIconOptions {
  /** The tray icon id. If undefined, a random one will be assigned */
  id?: string
  /** The tray icon menu */
  menu?: Menu | Submenu
  /**
   * The tray icon which could be icon bytes or path to the icon file.
   *
   * Note that you may need the `image-ico` or `image-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-png"] }
   * ```
   */
  icon?: string | Uint8Array | ArrayBuffer | number[] | Image
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
  /**
   * Whether to show the tray menu on left click or not, default is `true`.
   *
   * #### Platform-specific:
   *
   * - **Linux**: Unsupported.
   *
   * @deprecated use {@linkcode TrayIconOptions.showMenuOnLeftClick} instead.
   */
  menuOnLeftClick?: boolean
  /**
   * Whether to show the tray menu on left click or not, default is `true`.
   *
   * #### Platform-specific:
   *
   * - **Linux**: Unsupported.
   *
   * @since 2.2.0
   */
  showMenuOnLeftClick?: boolean
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

  /** Gets a tray icon using the provided id. */
  static async getById(id: string): Promise<TrayIcon | null> {
    return invoke<number>('plugin:tray|get_by_id', { id }).then((rid) =>
      rid ? new TrayIcon(rid, id) : null
    )
  }

  /**
   * Removes a tray icon using the provided id from tauri's internal state.
   *
   * Note that this may cause the tray icon to disappear
   * if it wasn't cloned somewhere else or referenced by JS.
   */
  static async removeById(id: string): Promise<void> {
    return invoke('plugin:tray|remove_by_id', { id })
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
      options.icon = transformImage(options.icon)
    }

    const handler = new Channel<RustTrayIconEvent>()
    if (options?.action) {
      const action = options.action
      handler.onmessage = (e) => action(mapEvent(e))
      delete options.action
    }

    return invoke<[number, string]>('plugin:tray|new', {
      options: options ?? {},
      handler
    }).then(([rid, id]) => new TrayIcon(rid, id))
  }

  /**
   *  Sets a new tray icon. If `null` is provided, it will remove the icon.
   *
   * Note that you may need the `image-ico` or `image-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-png"] }
   * ```
   */
  async setIcon(
    icon: string | Image | Uint8Array | ArrayBuffer | number[] | null
  ): Promise<void> {
    let trayIcon = null
    if (icon) {
      trayIcon = transformImage(icon)
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
   * #### Platform-specific:
   *
   * - **Linux:** Unsupported
   */
  async setTooltip(tooltip: string | null): Promise<void> {
    return invoke('plugin:tray|set_tooltip', { rid: this.rid, tooltip })
  }

  /**
   * Sets the tooltip for this tray icon.
   *
   * #### Platform-specific:
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

  /**
   *  Disable or enable showing the tray menu on left click.
   *
   * #### Platform-specific:
   *
   * - **Linux**: Unsupported.
   *
   * @deprecated use {@linkcode TrayIcon.setShowMenuOnLeftClick} instead.
   */
  async setMenuOnLeftClick(onLeft: boolean): Promise<void> {
    return invoke('plugin:tray|set_show_menu_on_left_click', {
      rid: this.rid,
      onLeft
    })
  }

  /**
   *  Disable or enable showing the tray menu on left click.
   *
   * #### Platform-specific:
   *
   * - **Linux**: Unsupported.
   *
   * @since 2.2.0
   */
  async setShowMenuOnLeftClick(onLeft: boolean): Promise<void> {
    return invoke('plugin:tray|set_show_menu_on_left_click', {
      rid: this.rid,
      onLeft
    })
  }
}

function mapEvent(e: RustTrayIconEvent): TrayIconEvent {
  const out = e as unknown as TrayIconEvent

  out.position = new PhysicalPosition(e.position)
  out.rect.position = new PhysicalPosition(e.rect.position)
  out.rect.size = new PhysicalSize(e.rect.size)

  return out
}
