// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import {
  MenuItemOptions,
  SubmenuOptions,
  IconMenuItemOptions,
  PredefinedMenuItemOptions,
  CheckMenuItemOptions
} from '../menu'
import { MenuItem } from './menuItem'
import { CheckMenuItem } from './checkMenuItem'
import { IconMenuItem } from './iconMenuItem'
import { PredefinedMenuItem } from './predefinedMenuItem'
import { itemFromKind, Submenu } from './submenu'
import { type LogicalPosition, PhysicalPosition, Position } from '../dpi'
import { type Window } from '../window'
import { invoke } from '../core'
import { type ItemKind, MenuItemBase, newMenu } from './base'

/** Options for creating a new menu. */
export interface MenuOptions {
  /** Specify an id to use for the new menu. */
  id?: string
  /** List of items to add to the new menu. */
  items?: Array<
    | Submenu
    | MenuItem
    | PredefinedMenuItem
    | CheckMenuItem
    | IconMenuItem
    | MenuItemOptions
    | SubmenuOptions
    | IconMenuItemOptions
    | PredefinedMenuItemOptions
    | CheckMenuItemOptions
  >
}

/** A type that is either a menu bar on the window
 * on Windows and Linux or as a global menu in the menubar on macOS.
 *
 * #### Platform-specific:
 *
 * - **macOS**: if using {@linkcode Menu} for the global menubar, it can only contain {@linkcode Submenu}s.
 */
export class Menu extends MenuItemBase {
  /** @ignore */
  protected constructor(rid: number, id: string) {
    super(rid, id, 'Menu')
  }

  /**
   * Create a new menu.
   *
   * @example
   * ```typescript
   * import { Menu, Submenu } from '@tauri-apps/api/menu';
   *
   * const menu = await Menu.new({
   *   items: [
   *     await Submenu.new({
   *       text: 'File',
   *       items: [
   *         { id: 'open', text: 'Open', accelerator: 'CmdOrCtrl+O', action: () => console.log('open') },
   *         { item: 'Separator' },
   *         { item: 'Quit' }
   *       ]
   *     })
   *   ]
   * });
   * ```
   */
  static async new(opts?: MenuOptions): Promise<Menu> {
    return newMenu('Menu', opts).then(([rid, id]) => new Menu(rid, id))
  }

  /**
   * Create the default application menu, the same one Tauri installs when no menu
   * is configured.
   *
   * It contains an `Edit` submenu (undo, redo, cut, copy, paste, select all), a
   * `Window` submenu (minimize, maximize, close window) and a `Help` submenu, plus:
   *
   * - **macOS:** an application submenu named after your app (about, services,
   *   hide, hide others, quit) and a `View` submenu with the fullscreen item.
   * - **Windows:** a `File` submenu with close window and quit; the about item
   *   lives in `Help`.
   * - **Linux:** no `File` submenu; the about item lives in `Help`.
   *
   * Useful as a starting point you then extend with {@linkcode Menu.append},
   * {@linkcode Menu.insert} or {@linkcode Menu.prepend}.
   *
   * @example
   * ```typescript
   * import { Menu, Submenu } from '@tauri-apps/api/menu';
   *
   * const menu = await Menu.default();
   * await menu.append(await Submenu.new({ text: 'Tools', items: [{ id: 'fmt', text: 'Format' }] }));
   * await menu.setAsAppMenu();
   * ```
   */
  static async default(): Promise<Menu> {
    return invoke<[number, string]>('plugin:menu|create_default').then(
      ([rid, id]) => new Menu(rid, id)
    )
  }

  /**
   * Add a menu item to the end of this menu.
   *
   * #### Platform-specific:
   *
   * - **macOS:** Only {@linkcode Submenu}s can be added to a {@linkcode Menu}.
   */
  async append<
    T extends
      | Submenu
      | MenuItem
      | PredefinedMenuItem
      | CheckMenuItem
      | IconMenuItem
      | MenuItemOptions
      | SubmenuOptions
      | IconMenuItemOptions
      | PredefinedMenuItemOptions
      | CheckMenuItemOptions
  >(items: T | T[]): Promise<void> {
    return invoke('plugin:menu|append', {
      rid: this.rid,
      kind: this.kind,
      items: (Array.isArray(items) ? items : [items]).map((i) =>
        'rid' in i ? [i.rid, i.kind] : i
      )
    })
  }

  /**
   * Add a menu item to the beginning of this menu.
   *
   * #### Platform-specific:
   *
   * - **macOS:** Only {@linkcode Submenu}s can be added to a {@linkcode Menu}.
   */
  async prepend<
    T extends
      | Submenu
      | MenuItem
      | PredefinedMenuItem
      | CheckMenuItem
      | IconMenuItem
      | MenuItemOptions
      | SubmenuOptions
      | IconMenuItemOptions
      | PredefinedMenuItemOptions
      | CheckMenuItemOptions
  >(items: T | T[]): Promise<void> {
    return invoke('plugin:menu|prepend', {
      rid: this.rid,
      kind: this.kind,
      items: (Array.isArray(items) ? items : [items]).map((i) =>
        'rid' in i ? [i.rid, i.kind] : i
      )
    })
  }

  /**
   * Add a menu item to the specified position in this menu.
   *
   * #### Platform-specific:
   *
   * - **macOS:** Only {@linkcode Submenu}s can be added to a {@linkcode Menu}.
   */
  async insert<
    T extends
      | Submenu
      | MenuItem
      | PredefinedMenuItem
      | CheckMenuItem
      | IconMenuItem
      | MenuItemOptions
      | SubmenuOptions
      | IconMenuItemOptions
      | PredefinedMenuItemOptions
      | CheckMenuItemOptions
  >(items: T | T[], position: number): Promise<void> {
    return invoke('plugin:menu|insert', {
      rid: this.rid,
      kind: this.kind,
      items: (Array.isArray(items) ? items : [items]).map((i) =>
        'rid' in i ? [i.rid, i.kind] : i
      ),
      position
    })
  }

  /** Remove a menu item from this menu. */
  async remove(
    item: Submenu | MenuItem | PredefinedMenuItem | CheckMenuItem | IconMenuItem
  ): Promise<void> {
    return invoke('plugin:menu|remove', {
      rid: this.rid,
      kind: this.kind,
      item: [item.rid, item.kind]
    })
  }

  /** Remove a menu item from this menu at the specified position. */
  async removeAt(
    position: number
  ): Promise<
    | Submenu
    | MenuItem
    | PredefinedMenuItem
    | CheckMenuItem
    | IconMenuItem
    | null
  > {
    return invoke<[number, string, ItemKind]>('plugin:menu|remove_at', {
      rid: this.rid,
      kind: this.kind,
      position
    }).then(itemFromKind)
  }

  /** Returns a list of menu items that has been added to this menu. */
  async items(): Promise<
    Array<
      Submenu | MenuItem | PredefinedMenuItem | CheckMenuItem | IconMenuItem
    >
  > {
    return invoke<Array<[number, string, ItemKind]>>('plugin:menu|items', {
      rid: this.rid,
      kind: this.kind
    }).then((i) => i.map(itemFromKind))
  }

  /** Retrieves the menu item matching the given identifier. */
  async get(
    id: string
  ): Promise<
    | Submenu
    | MenuItem
    | PredefinedMenuItem
    | CheckMenuItem
    | IconMenuItem
    | null
  > {
    return invoke<[number, string, ItemKind] | null>('plugin:menu|get', {
      rid: this.rid,
      kind: this.kind,
      id
    }).then((r) => (r ? itemFromKind(r) : null))
  }

  /**
   * Popup this menu as a context menu on the specified window.
   *
   * Call it from a `contextmenu` DOM listener (and `preventDefault()` on the event)
   * to replace the webview context menu with a native one. The promise resolves as
   * soon as the menu is shown, not when an item is picked: use each item's `action`
   * handler for that.
   *
   * @example
   * ```typescript
   * import { Menu } from '@tauri-apps/api/menu';
   * import { LogicalPosition } from '@tauri-apps/api/dpi';
   *
   * const menu = await Menu.new({
   *   items: [{ id: 'copy', text: 'Copy', action: () => console.log('copy') }]
   * });
   *
   * document.addEventListener('contextmenu', (event) => {
   *   event.preventDefault();
   *   void menu.popup(new LogicalPosition(event.clientX, event.clientY));
   * });
   * ```
   *
   * @param at If a position is provided, it is relative to the window's top-left corner.
   * If there isn't one provided, the menu will pop up at the current location of the mouse.
   * @param window The window to show the menu on. Defaults to the current window.
   */
  async popup(
    at?: PhysicalPosition | LogicalPosition | Position,
    window?: Window
  ): Promise<void> {
    return invoke('plugin:menu|popup', {
      rid: this.rid,
      kind: this.kind,
      window: window?.label ?? null,
      at: at instanceof Position ? at : at ? new Position(at) : null
    })
  }

  /**
   * Sets the app-wide menu and returns the previous one.
   *
   * If a window was not created with an explicit menu or had one set explicitly,
   * this menu will be assigned to it.
   *
   * This is the menu bar shown at the top of the screen on macOS. On Windows and
   * Linux, where menus belong to a window, use {@linkcode Menu.setAsWindowMenu}
   * to target one window instead.
   *
   * @example
   * ```typescript
   * import { Menu } from '@tauri-apps/api/menu';
   *
   * const menu = await Menu.default();
   * const previous = await menu.setAsAppMenu();
   * await previous?.close();
   * ```
   *
   * @returns The menu that was set before this call, or `null` if there was none.
   */
  async setAsAppMenu(): Promise<Menu | null> {
    return invoke<[number, string] | null>('plugin:menu|set_as_app_menu', {
      rid: this.rid
    }).then((r) => (r ? new Menu(r[0], r[1]) : null))
  }

  /**
   * Sets the window menu and returns the previous one.
   *
   * #### Platform-specific:
   *
   * - **macOS:** Unsupported. The menu on macOS is app-wide and not specific to one
   * window, if you need to set it, use {@linkcode Menu.setAsAppMenu} instead.
   *
   * @example
   * ```typescript
   * import { Menu } from '@tauri-apps/api/menu';
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   *
   * const menu = await Menu.default();
   * await menu.setAsWindowMenu(getCurrentWindow());
   * ```
   *
   * @param window The window to set the menu on. Defaults to the current window.
   * @returns The menu that was set on that window before this call, or `null` if there was none.
   */
  async setAsWindowMenu(window?: Window): Promise<Menu | null> {
    return invoke<[number, string] | null>('plugin:menu|set_as_window_menu', {
      rid: this.rid,
      window: window?.label ?? null
    }).then((r) => (r ? new Menu(r[0], r[1]) : null))
  }
}
