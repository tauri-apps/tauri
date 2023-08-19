// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { Resource, applyMixins } from './internal'
import { invoke } from './tauri'

// TODO: menu builders
// TODO: events or callbacks

/**
 * Menu types and utilities.
 *
 * This package is also accessible with `window.__TAURI__.menu` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */

interface MenuEvent {
  id: string
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

  protected static async _new(
    kind: ItemKind,
    opts?: unknown
  ): Promise<[number, string]> {
    return invoke('plugin:menu|new', { kind, opts })
  }
}

class MenuBase extends MenuItemBase {
  constructor(rid: number, id: string, kind: ItemKind) {
    super(rid, id, kind)
  }

  async append<
    T extends
      | Submenu
      | MenuItem
      | PredefinedMenuItem
      | CheckMenuItem
      | IconMenuItem
  >(items: T | T[]) {
    return invoke('plugin:menu|append', {
      rid: this.rid,
      kind: this.kind,
      items: (Array.isArray(items) ? items : [items]).map((i) => [
        i.kind,
        i.rid
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
  >(items: T | T[]) {
    return invoke('plugin:menu|prepend', {
      rid: this.rid,
      kind: this.kind,
      items: (Array.isArray(items) ? items : [items]).map((i) => [
        i.kind,
        i.rid
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
  >(items: T | T[], position: number) {
    return invoke('plugin:menu|insert', {
      rid: this.rid,
      kind: this.kind,
      items: (Array.isArray(items) ? items : [items]).map((i) => [
        i.kind,
        i.rid
      ]),
      position
    })
  }

  async remove(
    item: Submenu | MenuItem | PredefinedMenuItem | CheckMenuItem | IconMenuItem
  ) {
    return invoke('plugin:menu|remove', {
      rid: this.rid,
      kind: this.kind,
      item: [item.kind, item.rid]
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
    (Submenu | MenuItem | PredefinedMenuItem | CheckMenuItem | IconMenuItem)[]
  > {
    return invoke<[number, string, ItemKind][]>('plugin:menu|append', {
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

  // TODO: change to logical position
  // TODO use window type after migrating window back to core
  async popup(window: string, position?: [number, number]) {
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
  items: (
    | Submenu
    | MenuItem
    | PredefinedMenuItem
    | CheckMenuItem
    | IconMenuItem
  )[]
}

class Menu extends MenuBase {
  constructor(rid: number, id: string) {
    super(rid, id, 'Menu')
  }

  static async new(opts: MenuOptions) {
    return super._new('Menu', opts).then(([rid, id]) => new Menu(rid, id))
  }

  static async default() {
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

  async setText(text: string) {
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

  async setEnabled(enabled: boolean) {
    return invoke('plugin:menu|set_enabled', {
      rid: this.rid,
      kind: this.kind,
      enabled
    })
  }
}

class MenuItemBase4 extends MenuItemBase3 {
  async setAccelerator(accelerator: string | null) {
    return invoke('plugin:menu|set_accelerator', {
      rid: this.rid,
      kind: this.kind,
      accelerator
    })
  }
}

interface MenuItemOptions {
  id?: string
  text?: string
  enabled?: boolean
  accelerator?: string
}

class MenuItem extends MenuItemBase4 {
  constructor(rid: number, id: string) {
    super(rid, id, 'MenuItem')
  }

  static async new(opts?: MenuItemOptions) {
    return super
      ._new('MenuItem', opts)
      .then(([rid, id]) => new MenuItem(rid, id))
  }
}

type SubmenuOptions = Omit<MenuItemOptions, 'accelerator'> & MenuOptions

interface Submenu extends MenuItemBase3 {}
class Submenu extends MenuBase {
  constructor(rid: number, id: string) {
    super(rid, id, 'Submenu')
  }

  static async new(opts?: SubmenuOptions) {
    return super._new('Submenu', opts).then(([rid, id]) => new Submenu(rid, id))
  }

  async setAsWindowsMenuForNSApp() {
    return invoke('plugin:menu|set_as_windows_menu_for_nsapp', {
      rid: this.rid
    })
  }

  async setAsHelpMenuForNSApp() {
    return invoke('plugin:menu|set_as_help_menu_for_nsapp', {
      rid: this.rid
    })
  }
}

applyMixins(Submenu, MenuItemBase3)

type PredefinedMenuItemOptions = Omit<
  MenuItemOptions,
  'enabled' | 'accelerator' | 'id'
> & {
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

  static async new(opts?: PredefinedMenuItemOptions) {
    return super
      ._new('MenuItem', opts)
      .then(([rid, id]) => new MenuItem(rid, id))
  }
}

interface CheckMenuItemOptions extends MenuItemOptions {
  checked?: boolean
}

class CheckMenuItem extends MenuItemBase4 {
  constructor(rid: number, id: string) {
    super(rid, id, 'Check')
  }

  static async new(opts?: MenuItemOptions) {
    return super
      ._new('Check', opts)
      .then(([rid, id]) => new CheckMenuItem(rid, id))
  }

  async isChecked(): Promise<boolean> {
    return invoke('plugin:menu|is_checked', { rid: this.rid })
  }

  async setChecked(checked: boolean) {
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

  static async new(opts?: IconMenuItemOptions) {
    return super
      ._new('Icon', opts)
      .then(([rid, id]) => new IconMenuItem(rid, id))
  }

  async setIcon(icon: string | Uint8Array | null) {
    return invoke('plugin:menu|set_icon', { rid: this.rid, icon })
  }

  async setNativeIcon(icon: NativeIcon | null) {
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
export { NativeIcon, MenuItem, CheckMenuItem, IconMenuItem, Submenu, Menu }
