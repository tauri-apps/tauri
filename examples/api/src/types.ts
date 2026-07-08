import type { Component } from 'svelte'
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

export type MessageHandler = (value: unknown, ...extra: unknown[]) => void

export interface View {
  label: string
  component: Component<{ onMessage: MessageHandler }>
  icon: string
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
