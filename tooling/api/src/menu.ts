// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { TauriEvent, listen } from './event'
import { Resource, applyMixins } from './internal'
import { invoke } from './tauri'

// TODO menu builders

/**
 * Menu types and utilities.
 *
 * This package is also accessible with `window.__TAURI__.menu` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */

interface MenuEvent {
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
        target: window.__TAURI_METADATA__.__currentWindow.label
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

async function newMenu(kind: ItemKind, opts?: any): Promise<[number, string]> {
  let handler: null | (() => void) = null
  let items: null | Array<[number, string]> = null
  if (opts) {
    if ('action' in opts) {
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

  get id(): string {
    return this.#id
  }

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

  async remove(
    item: Submenu | MenuItem | PredefinedMenuItem | CheckMenuItem | IconMenuItem
  ): Promise<void> {
    return invoke('plugin:menu|remove', {
      rid: this.rid,
      kind: this.kind,
      item: [item.rid, item.kind]
    })
  }

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
  async popup(window: string, position?: [number, number]): Promise<void> {
    return invoke('plugin:menu|popup', {
      rid: this.rid,
      kind: this.kind,
      window,
      position
    })
  }
}

interface MenuOptions {
  id?: string
  items?: Array<
    Submenu | MenuItem | PredefinedMenuItem | CheckMenuItem | IconMenuItem
  >
}

class Menu extends MenuBase {
  constructor(rid: number, id: string) {
    super(rid, id, 'Menu')
  }

  static async new(opts?: MenuOptions): Promise<Menu> {
    return newMenu('Menu', opts).then(([rid, id]) => new Menu(rid, id))
  }

  static async default(): Promise<Menu> {
    return invoke<[number, string]>('plugin:menu|default').then(
      ([rid, id]) => new Menu(rid, id)
    )
  }

  async setAsAppMenu(): Promise<Menu | null> {
    return invoke<[number, string] | null>('plugin:menu|set_as_app_menu', {
      rid: this.rid
    }).then((r) => (r ? new Menu(r[0], r[1]) : null))
  }

  // TODO use window type after migrating window back to core
  async setAsWindowMenu(window: string): Promise<Menu | null> {
    return invoke<[number, string] | null>('plugin:menu|set_as_window_menu', {
      rid: this.rid,
      window
    }).then((r) => (r ? new Menu(r[0], r[1]) : null))
  }
}

class MenuItemBase2 extends MenuItemBase {
  async text(): Promise<string> {
    return invoke('plugin:menu|text', { rid: this.rid, kind: this.kind })
  }

  async setText(text: string): Promise<void> {
    return invoke('plugin:menu|set_text', {
      rid: this.rid,
      kind: this.kind,
      text
    })
  }
}

class MenuItemBase3 extends MenuItemBase2 {
  async isEnabled(): Promise<boolean> {
    return invoke('plugin:menu|is_enabled', { rid: this.rid, kind: this.kind })
  }

  async setEnabled(enabled: boolean): Promise<void> {
    return invoke('plugin:menu|set_enabled', {
      rid: this.rid,
      kind: this.kind,
      enabled
    })
  }
}

class MenuItemBase4 extends MenuItemBase3 {
  async setAccelerator(accelerator: string | null): Promise<void> {
    return invoke('plugin:menu|set_accelerator', {
      rid: this.rid,
      kind: this.kind,
      accelerator
    })
  }
}

interface MenuItemOptions {
  id?: string
  text: string
  enabled?: boolean
  accelerator?: string
  action?: () => void
}

class MenuItem extends MenuItemBase4 {
  constructor(rid: number, id: string) {
    super(rid, id, 'MenuItem')
  }

  static async new(opts: MenuItemOptions): Promise<MenuItem> {
    return newMenu('MenuItem', opts).then(([rid, id]) => new MenuItem(rid, id))
  }
}

type SubmenuOptions = Omit<MenuItemOptions, 'accelerator' | 'action'> &
  MenuOptions

interface Submenu extends MenuItemBase3 {}
class Submenu extends MenuBase {
  constructor(rid: number, id: string) {
    super(rid, id, 'Submenu')
  }

  static async new(opts: SubmenuOptions): Promise<Submenu> {
    return newMenu('Submenu', opts).then(([rid, id]) => new Submenu(rid, id))
  }

  async setAsWindowsMenuForNSApp(): Promise<void> {
    return invoke('plugin:menu|set_as_windows_menu_for_nsapp', {
      rid: this.rid
    })
  }

  async setAsHelpMenuForNSApp(): Promise<void> {
    return invoke('plugin:menu|set_as_help_menu_for_nsapp', {
      rid: this.rid
    })
  }
}

applyMixins(Submenu, MenuItemBase3)

interface PredefinedMenuItemOptions {
  text?: string
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
        About: {
          name?: string
          version?: string
          short_version?: string
          authors?: string[]
          comments?: string
          copyright?: string
          license?: string
          website?: string
          website_label?: string
          credits?: string
          icon?: string | Uint8Array
        }
      }
}

class PredefinedMenuItem extends MenuItemBase2 {
  constructor(rid: number, id: string) {
    super(rid, id, 'MenuItem')
  }

  static async new(
    opts?: PredefinedMenuItemOptions
  ): Promise<PredefinedMenuItem> {
    return newMenu('MenuItem', opts).then(
      ([rid, id]) => new PredefinedMenuItem(rid, id)
    )
  }
}

interface CheckMenuItemOptions extends MenuItemOptions {
  checked?: boolean
}

class CheckMenuItem extends MenuItemBase4 {
  constructor(rid: number, id: string) {
    super(rid, id, 'Check')
  }

  static async new(opts: MenuItemOptions): Promise<CheckMenuItem> {
    return newMenu('Check', opts).then(
      ([rid, id]) => new CheckMenuItem(rid, id)
    )
  }

  async isChecked(): Promise<boolean> {
    return invoke('plugin:menu|is_checked', { rid: this.rid })
  }

  async setChecked(checked: boolean): Promise<void> {
    return invoke('plugin:menu|set_checked', {
      rid: this.rid,
      checked
    })
  }
}

interface IconMenuItemOptions extends MenuItemOptions {
  icon?: string | Uint8Array
  nativeIcon?: NativeIcon
}
class IconMenuItem extends MenuItemBase4 {
  constructor(rid: number, id: string) {
    super(rid, id, 'Icon')
  }

  static async new(opts: IconMenuItemOptions): Promise<IconMenuItem> {
    return newMenu('Icon', opts).then(([rid, id]) => new IconMenuItem(rid, id))
  }

  async setIcon(icon: string | Uint8Array | null): Promise<void> {
    return invoke('plugin:menu|set_icon', { rid: this.rid, icon })
  }

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
  IconMenuItemOptions
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
