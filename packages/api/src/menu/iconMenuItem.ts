// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { MenuItemBase, newMenu } from './base'
import { type MenuItemOptions } from '../menu'
import { invoke } from '../core'
import { Image, transformImage } from '../image'

/**
 * A native Icon to be used for the menu item
 *
 * #### Platform-specific:
 *
 * - **Windows / Linux**: Unsupported.
 */
export enum NativeIcon {
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
  /** Small green indicator, similar to iChat's available image. */
  StatusAvailable = 'StatusAvailable',
  /** Small clear indicator. */
  StatusNone = 'StatusNone',
  /** Small yellow indicator, similar to iChat's idle image. */
  StatusPartiallyAvailable = 'StatusPartiallyAvailable',
  /** Small red indicator, similar to iChat's unavailable image. */
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

/** Options for creating a new icon menu item. */
export interface IconMenuItemOptions extends MenuItemOptions {
  /**
   * Icon to be used for the new icon menu item.
   *
   * Note that you may need the `image-ico` or `image-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-png"] }
   * ```
   */
  icon?: NativeIcon | string | Image | Uint8Array | ArrayBuffer | number[]
}

/**
 * An icon menu item inside a {@linkcode Menu} or {@linkcode Submenu}
 * and usually contains an icon and a text.
 */
export class IconMenuItem extends MenuItemBase {
  /** @ignore */
  protected constructor(rid: number, id: string) {
    super(rid, id, 'Icon')
  }

  /** Create a new icon menu item. */
  static async new(opts: IconMenuItemOptions): Promise<IconMenuItem> {
    return newMenu('Icon', opts).then(([rid, id]) => new IconMenuItem(rid, id))
  }

  /** Returns the text of this icon menu item. */
  async text(): Promise<string> {
    return invoke('plugin:menu|text', { rid: this.rid, kind: this.kind })
  }

  /** Sets the text for this icon menu item. */
  async setText(text: string): Promise<void> {
    return invoke('plugin:menu|set_text', {
      rid: this.rid,
      kind: this.kind,
      text
    })
  }

  /** Returns whether this icon menu item is enabled or not. */
  async isEnabled(): Promise<boolean> {
    return invoke('plugin:menu|is_enabled', { rid: this.rid, kind: this.kind })
  }

  /** Sets whether this icon menu item is enabled or not. */
  async setEnabled(enabled: boolean): Promise<void> {
    return invoke('plugin:menu|set_enabled', {
      rid: this.rid,
      kind: this.kind,
      enabled
    })
  }

  /** Sets the accelerator for this icon menu item. */
  async setAccelerator(accelerator: string | null): Promise<void> {
    return invoke('plugin:menu|set_accelerator', {
      rid: this.rid,
      kind: this.kind,
      accelerator
    })
  }

  /** Sets an icon for this icon menu item */
  async setIcon(
    icon:
      | NativeIcon
      | string
      | Image
      | Uint8Array
      | ArrayBuffer
      | number[]
      | null
  ): Promise<void> {
    return invoke('plugin:menu|set_icon', {
      rid: this.rid,
      icon: transformImage(icon)
    })
  }
}
