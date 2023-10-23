// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { TauriEvent, listen } from './event'
import { Resource, applyMixins } from './internal'
import { invoke } from './primitives'
import { getCurrent } from './window'

/**
 * Menu types and utilities.
 *
 * This package is also accessible with `window.__TAURI__.menu` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */

/** Describes a menu event emitted when a menu item is activated */
interface MenuEvent {
  /** Id of the menu item that triggered this event */
  id: string
}

declare global {
  interface Window {
    __TAURI_MENU__?: {
      handlers: Record<string, Array<() => void>>
    }
  }
}

async function addEventListener(
  id: string,
  handler: () => void
): Promise<void> {
  if (!window.__TAURI_MENU__) {
    window.__TAURI_MENU__ = { handlers: {} }
    const unlisten = await listen<MenuEvent>(
      TauriEvent.MENU,
      (e) => {
        const handlers = window.__TAURI_MENU__?.handlers[e.payload.id]
        if (handlers) {
          for (handler of handlers) {
            handler()
          }
        }
      },
      {
        target: getCurrent().label
      }
    )

    window.addEventListener('unload', unlisten)
  }

  // eslint-disable-next-line
  if (!window.__TAURI_MENU__.handlers[id]) {
    // eslint-disable-next-line
    window.__TAURI_MENU__.handlers[id] = []
  }

  // eslint-disable-next-line
  window.__TAURI_MENU__.handlers[id].push(handler)
}

/**
 * A native Icon to be used for the menu item
 *
 * #### Platform-specific:
 *
 * - **Windows / Linux**: Unsupported.
 */
enum NativeIcon {
  /** An add item template image. */
  Add = 'Add',
  /** Advanced preferences toolbar icon for the preferences window. */
  Advanced = 'Advanced',
  /** A Bluetooth template image. */
  Bluetooth = 'Bluetooth',
  /** Bookmarks image suitable for a template. */
  Bookmarks = 'Bookmarks',
  /** A caution image. */
  Caution = 'Caution',
  /** A color panel toolbar icon. */
  ColorPanel = 'ColorPanel',
  /** A column view mode template image. */
  ColumnView = 'ColumnView',
  /** A computer icon. */
  Computer = 'Computer',
  /** An enter full-screen mode template image. */
  EnterFullScreen = 'EnterFullScreen',
  /** Permissions for all users. */
  Everyone = 'Everyone',
  /** An exit full-screen mode template image. */
  ExitFullScreen = 'ExitFullScreen',
  /** A cover flow view mode template image. */
  FlowView = 'FlowView',
  /** A folder image. */
  Folder = 'Folder',
  /** A burnable folder icon. */
  FolderBurnable = 'FolderBurnable',
  /** A smart folder icon. */
  FolderSmart = 'FolderSmart',
  /** A link template image. */
  FollowLinkFreestanding = 'FollowLinkFreestanding',
  /** A font panel toolbar icon. */
  FontPanel = 'FontPanel',
  /** A `go back` template image. */
  GoLeft = 'GoLeft',
  /** A `go forward` template image. */
  GoRight = 'GoRight',
  /** Home image suitable for a template. */
  Home = 'Home',
  /** An iChat Theater template image. */
  IChatTheater = 'IChatTheater',
  /** An icon view mode template image. */
  IconView = 'IconView',
  /** An information toolbar icon. */
  Info = 'Info',
  /** A template image used to denote invalid data. */
  InvalidDataFreestanding = 'InvalidDataFreestanding',
  /** A generic left-facing triangle template image. */
  LeftFacingTriangle = 'LeftFacingTriangle',
  /** A list view mode template image. */
  ListView = 'ListView',
  /** A locked padlock template image. */
  LockLocked = 'LockLocked',
  /** An unlocked padlock template image. */
  LockUnlocked = 'LockUnlocked',
  /** A horizontal dash, for use in menus. */
  MenuMixedState = 'MenuMixedState',
  /** A check mark template image, for use in menus. */
  MenuOnState = 'MenuOnState',
  /** A MobileMe icon. */
  MobileMe = 'MobileMe',
  /** A drag image for multiple items. */
  MultipleDocuments = 'MultipleDocuments',
  /** A network icon. */
  Network = 'Network',
  /** A path button template image. */
  Path = 'Path',
  /** General preferences toolbar icon for the preferences window. */
  PreferencesGeneral = 'PreferencesGeneral',
  /** A Quick Look template image. */
  QuickLook = 'QuickLook',
  /** A refresh template image. */
  RefreshFreestanding = 'RefreshFreestanding',
  /** A refresh template image. */
  Refresh = 'Refresh',
  /** A remove item template image. */
  Remove = 'Remove',
  /** A reveal contents template image. */
  RevealFreestanding = 'RevealFreestanding',
  /** A generic right-facing triangle template image. */
  RightFacingTriangle = 'RightFacingTriangle',
  /** A share view template image. */
  Share = 'Share',
  /** A slideshow template image. */
  Slideshow = 'Slideshow',
  /** A badge for a `smart` item. */
  SmartBadge = 'SmartBadge',
  /** Small green indicator, similar to iChat’s available image. */
  StatusAvailable = 'StatusAvailable',
  /** Small clear indicator. */
  StatusNone = 'StatusNone',
  /** Small yellow indicator, similar to iChat’s idle image. */
  StatusPartiallyAvailable = 'StatusPartiallyAvailable',
  /** Small red indicator, similar to iChat’s unavailable image. */
  StatusUnavailable = 'StatusUnavailable',
  /** A stop progress template image. */
  StopProgressFreestanding = 'StopProgressFreestanding',
  /** A stop progress button template image. */
  StopProgress = 'StopProgress',
  /** An image of the empty trash can. */
  TrashEmpty = 'TrashEmpty',
  /** An image of the full trash can. */
  TrashFull = 'TrashFull',
  /** Permissions for a single user. */
  User = 'User',
  /** User account toolbar icon for the preferences window. */
  UserAccounts = 'UserAccounts',
  /** Permissions for a group of users. */
  UserGroup = 'UserGroup',
  /** Permissions for guests. */
  UserGuest = 'UserGuest'
}

type ItemKind =
  | 'MenuItem'
  | 'Predefined'
  | 'Check'
  | 'Icon'
  | 'Submenu'
  | 'Menu'

function itemFromKind([rid, id, kind]: [number, string, ItemKind]):
  | Submenu
  | MenuItem
  | PredefinedMenuItem
  | CheckMenuItem
  | IconMenuItem {
  switch (kind) {
    case 'Submenu':
      return new Submenu(rid, id)
    case 'Predefined':
      return new PredefinedMenuItem(rid, id)
    case 'Check':
      return new CheckMenuItem(rid, id)
    case 'Icon':
      return new IconMenuItem(rid, id)
    case 'MenuItem':
    default:
      return new MenuItem(rid, id)
  }
}

async function newMenu(
  kind: ItemKind,
  opts?:
    | MenuOptions
    | MenuItemOptions
    | SubmenuOptions
    | PredefinedMenuItemOptions
    | IconMenuItemOptions
): Promise<[number, string]> {
  let handler: null | (() => void) = null
  let items: null | Array<[number, string]> = null
  if (opts) {
    if ('action' in opts && opts.action !== void 0) {
      // eslint-disable-next-line
      handler = opts.action
    }

    if ('items' in opts) {
      // @ts-expect-error items should all have rid and kind accessors
      // eslint-disable-next-line
      items = opts.items.map((i) => [i.rid, i.kind])
    }
  }

  return invoke<[number, string]>('plugin:menu|new', {
    kind,
    options: opts ? { ...opts, items } : undefined
  }).then((ret) => {
    if (handler) {
      void addEventListener(ret[1], handler)
    }
    return ret
  })
}

class MenuItemBase extends Resource {
  #id: string
  #kind: ItemKind

  /// The id of this item.
  get id(): string {
    return this.#id
  }

  /** @ignore */
  get kind(): string {
    return this.#kind
  }

  constructor(rid: number, id: string, kind: ItemKind) {
    super(rid)
    this.#id = id
    this.#kind = kind
  }
}

class MenuBase extends MenuItemBase {
  /**
   * Add a menu item to the end of this menu.
   *
   * ## Platform-spcific:
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
  >(items: T | T[]): Promise<void> {
    return invoke('plugin:menu|append', {
      rid: this.rid,
      kind: this.kind,
      items: (Array.isArray(items) ? items : [items]).map((i) => [
        i.rid,
        i.kind
      ])
    })
  }

  /**
   * Add a menu item to the beginning of this menu.
   *
   * ## Platform-spcific:
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
  >(items: T | T[]): Promise<void> {
    return invoke('plugin:menu|prepend', {
      rid: this.rid,
      kind: this.kind,
      items: (Array.isArray(items) ? items : [items]).map((i) => [
        i.rid,
        i.kind
      ])
    })
  }

  /**
   * Add a menu item to the specified position in this menu.
   *
   * ## Platform-spcific:
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
  >(items: T | T[], position: number): Promise<void> {
    return invoke('plugin:menu|insert', {
      rid: this.rid,
      kind: this.kind,
      items: (Array.isArray(items) ? items : [items]).map((i) => [
        i.rid,
        i.kind
      ]),
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
    return invoke<Array<[number, string, ItemKind]>>('plugin:menu|append', {
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
    return invoke<[number, string, ItemKind] | null>('plugin:menu|append', {
      rid: this.rid,
      kind: this.kind,
      id
    }).then((r) => (r ? itemFromKind(r) : null))
  }

  // TODO change to logical position
  // TODO use window type after migrating window back to core
  /**
   * Popup this menu as a context menu on the specified window.
   *
   * If the position, is provided, it is relative to the window's top-left corner.
   */
  async popup(window: string, position?: [number, number]): Promise<void> {
    return invoke('plugin:menu|popup', {
      rid: this.rid,
      kind: this.kind,
      window,
      position
    })
  }
}

/** Options for creating a new menu. */
interface MenuOptions {
  /** Specify an id to use for the new menu. */
  id?: string
  /** List of items to add to the new menu. */
  items?: Array<
    Submenu | MenuItem | PredefinedMenuItem | CheckMenuItem | IconMenuItem
  >
}

/** A type that is either a menu bar on the window
 * on Windows and Linux or as a global menu in the menubar on macOS.
 */
class Menu extends MenuBase {
  constructor(rid: number, id: string) {
    super(rid, id, 'Menu')
  }

  /** Create a new menu. */
  static async new(opts?: MenuOptions): Promise<Menu> {
    return newMenu('Menu', opts).then(([rid, id]) => new Menu(rid, id))
  }

  /** Create a default menu. */
  static async default(): Promise<Menu> {
    return invoke<[number, string]>('plugin:menu|default').then(
      ([rid, id]) => new Menu(rid, id)
    )
  }

  /**
   * Sets the app-wide menu and returns the previous one.
   *
   * If a window was not created with an explicit menu or had one set explicitly,
   * this menu will be assigned to it.
   */
  async setAsAppMenu(): Promise<Menu | null> {
    return invoke<[number, string] | null>('plugin:menu|set_as_app_menu', {
      rid: this.rid
    }).then((r) => (r ? new Menu(r[0], r[1]) : null))
  }

  // TODO use window type after migrating window back to core
  /**
   * Sets the window menu and returns the previous one.
   *
   * #### Platform-specific:
   *
   * - **macOS:** Unsupported. The menu on macOS is app-wide and not specific to one
   * window, if you need to set it, use {@linkcode Menu.setAsAppMenu} instead.
   */
  async setAsWindowMenu(window: string): Promise<Menu | null> {
    return invoke<[number, string] | null>('plugin:menu|set_as_window_menu', {
      rid: this.rid,
      window
    }).then((r) => (r ? new Menu(r[0], r[1]) : null))
  }
}

class MenuItemBase2 extends MenuItemBase {
  /** Returns the text of this item. */
  async text(): Promise<string> {
    return invoke('plugin:menu|text', { rid: this.rid, kind: this.kind })
  }

  /** Sets the text for this item. */
  async setText(text: string): Promise<void> {
    return invoke('plugin:menu|set_text', {
      rid: this.rid,
      kind: this.kind,
      text
    })
  }
}

class MenuItemBase3 extends MenuItemBase2 {
  /** Returns whether this item is enabled or not. */
  async isEnabled(): Promise<boolean> {
    return invoke('plugin:menu|is_enabled', { rid: this.rid, kind: this.kind })
  }

  /** Sets whether this item is enabled or not. */
  async setEnabled(enabled: boolean): Promise<void> {
    return invoke('plugin:menu|set_enabled', {
      rid: this.rid,
      kind: this.kind,
      enabled
    })
  }
}

class MenuItemBase4 extends MenuItemBase3 {
  /** Sets the accelerator for this menu item. */
  async setAccelerator(accelerator: string | null): Promise<void> {
    return invoke('plugin:menu|set_accelerator', {
      rid: this.rid,
      kind: this.kind,
      accelerator
    })
  }
}

/** Options for creating a new menu item. */
interface MenuItemOptions {
  /** Specify an id to use for the new menu item. */
  id?: string
  /** The text of the new menu item. */
  text: string
  /** Whether the new menu item is enabled or not. */
  enabled?: boolean
  /** Specify an accelerator for the new menu item. */
  accelerator?: string
  /** Specify a handler to be called when this menu item is activated. */
  action?: () => void
}

/** A menu item inside a {@linkcode Menu} or {@linkcode Submenu} and contains only text. */
class MenuItem extends MenuItemBase4 {
  constructor(rid: number, id: string) {
    super(rid, id, 'MenuItem')
  }

  /** Create a new menu item. */
  static async new(opts: MenuItemOptions): Promise<MenuItem> {
    return newMenu('MenuItem', opts).then(([rid, id]) => new MenuItem(rid, id))
  }
}

type SubmenuOptions = Omit<MenuItemOptions, 'accelerator' | 'action'> &
  MenuOptions

interface Submenu extends MenuItemBase3 {}

/** A type that is a submenu inside a {@linkcode Menu} or {@linkcode Submenu}. */
class Submenu extends MenuBase {
  constructor(rid: number, id: string) {
    super(rid, id, 'Submenu')
  }

  /** Create a new submenu. */
  static async new(opts: SubmenuOptions): Promise<Submenu> {
    return newMenu('Submenu', opts).then(([rid, id]) => new Submenu(rid, id))
  }

  /**
   * Set this submenu as the Window menu for the application on macOS.
   *
   * This will cause macOS to automatically add window-switching items and
   * certain other items to the menu.
   *
   * #### Platform-specific:
   *
   * - **Windows / Linux**: Unsupported.
   */
  async setAsWindowsMenuForNSApp(): Promise<void> {
    return invoke('plugin:menu|set_as_windows_menu_for_nsapp', {
      rid: this.rid
    })
  }

  /**
   * Set this submenu as the Help menu for the application on macOS.
   *
   * This will cause macOS to automatically add a search box to the menu.
   *
   * If no menu is set as the Help menu, macOS will automatically use any menu
   * which has a title matching the localized word "Help".
   *
   * #### Platform-specific:
   *
   * - **Windows / Linux**: Unsupported.
   */
  async setAsHelpMenuForNSApp(): Promise<void> {
    return invoke('plugin:menu|set_as_help_menu_for_nsapp', {
      rid: this.rid
    })
  }
}

applyMixins(Submenu, MenuItemBase3)

/** A metadata for the about predefined menu item. */
interface AboutMetadata {
  /** Sets the application name. */
  name?: string
  /** The application version. */
  version?: string
  /**
   * The short version, e.g. "1.0".
   *
   * #### Platform-specific
   *
   * - **Windows / Linux:** Appended to the end of `version` in parentheses.
   */
  short_version?: string
  /**
   * The authors of the application.
   *
   * #### Platform-specific
   *
   * - **macOS:** Unsupported.
   */
  authors?: string[]
  /**
   * Application comments.
   *
   * #### Platform-specific
   *
   * - **macOS:** Unsupported.
   */
  comments?: string
  /** The copyright of the application. */
  copyright?: string
  /**
   * The license of the application.
   *
   * #### Platform-specific
   *
   * - **macOS:** Unsupported.
   */
  license?: string
  /**
   * The application website.
   *
   * #### Platform-specific
   *
   * - **macOS:** Unsupported.
   */
  website?: string
  /**
   * The website label.
   *
   * #### Platform-specific
   *
   * - **macOS:** Unsupported.
   */
  website_label?: string
  /**
   * The credits.
   *
   * #### Platform-specific
   *
   * - **Windows / Linux:** Unsupported.
   */
  credits?: string
  /**
   * The application icon.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  icon?: string | Uint8Array
}

/** Options for creating a new predefined menu item. */
interface PredefinedMenuItemOptions {
  /** The text of the new predefined menu item. */
  text?: string
  /** The predefined item type */
  item:
    | 'Separator'
    | 'Copy'
    | 'Cut'
    | 'Paste'
    | 'SelectAll'
    | 'Undo'
    | 'Redo'
    | 'Minimize'
    | 'Maximize'
    | 'Fullscreen'
    | 'Hide'
    | 'HideOthers'
    | 'ShowAll'
    | 'CloseWindow'
    | 'Quit'
    | 'Services'
    | {
        About: AboutMetadata | null
      }
}

/** A predefined (native) menu item which has a predfined behavior by the OS or by tauri.  */
class PredefinedMenuItem extends MenuItemBase2 {
  constructor(rid: number, id: string) {
    super(rid, id, 'MenuItem')
  }

  /** Create a new predefined menu item. */
  static async new(
    opts?: PredefinedMenuItemOptions
  ): Promise<PredefinedMenuItem> {
    return newMenu('MenuItem', opts).then(
      ([rid, id]) => new PredefinedMenuItem(rid, id)
    )
  }
}

/** Options for creating a new check menu item. */
interface CheckMenuItemOptions extends MenuItemOptions {
  /** Whether the new check menu item is enabled or not. */
  checked?: boolean
}

/**
 * A chcek menu item inside a {@linkcode Menu} or {@linkcode Submenu}
 * and usually contains a text and a check mark or a similar toggle
 * that corresponds to a checked and unchecked states.
 */
class CheckMenuItem extends MenuItemBase4 {
  constructor(rid: number, id: string) {
    super(rid, id, 'Check')
  }

  /** Create a new check menu item. */
  static async new(opts: MenuItemOptions): Promise<CheckMenuItem> {
    return newMenu('Check', opts).then(
      ([rid, id]) => new CheckMenuItem(rid, id)
    )
  }

  /** Returns whether this item is checked or not. */
  async isChecked(): Promise<boolean> {
    return invoke('plugin:menu|is_checked', { rid: this.rid })
  }

  /** Sets whether this item is checked or not. */
  async setChecked(checked: boolean): Promise<void> {
    return invoke('plugin:menu|set_checked', {
      rid: this.rid,
      checked
    })
  }
}

/** Options for creating a new icon menu item. */
interface IconMenuItemOptions extends MenuItemOptions {
  /**
   * Icon to be used for the new icon menu item.
   *
   * if both {@linkcode IconMenuItemOptions.icon|icon} and {@linkcode IconMenuItemOptions.nativeIcon|nativeIcon} are specified, only {@linkcode IconMenuItemOptions.icon|icon} is used.
   */
  icon?: string | Uint8Array
  /**
   * NativeIcon to be used for the new icon menu item.
   *
   * if both {@linkcode IconMenuItemOptions.icon|icon} and {@linkcode IconMenuItemOptions.nativeIcon|nativeIcon} are specified, only {@linkcode IconMenuItemOptions.icon|icon} is used.
   */
  nativeIcon?: NativeIcon
}

/**
 * An icon menu item inside a {@linkcode Menu} or {@linkcode Submenu}
 * and usually contains an icon and a text.
 */
class IconMenuItem extends MenuItemBase4 {
  constructor(rid: number, id: string) {
    super(rid, id, 'Icon')
  }

  /** Create a new icon menu item. */
  static async new(opts: IconMenuItemOptions): Promise<IconMenuItem> {
    return newMenu('Icon', opts).then(([rid, id]) => new IconMenuItem(rid, id))
  }

  /** Sets an icon for this icon menu item */
  async setIcon(icon: string | Uint8Array | null): Promise<void> {
    return invoke('plugin:menu|set_icon', { rid: this.rid, icon })
  }

  /** Sets a native icon for this icon menu item */
  async setNativeIcon(icon: NativeIcon | null): Promise<void> {
    return invoke('plugin:menu|set_native_icon', { rid: this.rid, icon })
  }
}

export type {
  MenuEvent,
  MenuOptions,
  MenuItemOptions,
  SubmenuOptions,
  PredefinedMenuItemOptions,
  CheckMenuItemOptions,
  IconMenuItemOptions,
  AboutMetadata
}
export {
  NativeIcon,
  MenuItem,
  CheckMenuItem,
  PredefinedMenuItem,
  IconMenuItem,
  Submenu,
  Menu
}
