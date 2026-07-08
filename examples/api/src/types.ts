// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import type {
  CheckMenuItem,
  CheckMenuItemOptions,
  IconMenuItem,
  IconMenuItemOptions,
  MenuItem,
  MenuItemOptions,
  PredefinedMenuItem,
  PredefinedMenuItemOptions
} from '@tauri-apps/api/menu'

export type MessageHandler = (value: unknown) => void

export type ViewProps = {
  onMessage: MessageHandler
}

export interface MenuItemClickDetail {
  id: string
  text: string
}

export type MenuItemClickHandler = (detail: MenuItemClickDetail) => void

export type BuiltMenuItemInstance =
  | MenuItem
  | IconMenuItem
  | CheckMenuItem
  | PredefinedMenuItem

export type BuiltMenuItemOptions =
  | MenuItemOptions
  | IconMenuItemOptions
  | CheckMenuItemOptions
  | PredefinedMenuItemOptions

export interface BuiltMenuItem {
  item: BuiltMenuItemInstance
  options: BuiltMenuItemOptions
}
