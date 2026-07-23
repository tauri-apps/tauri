// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { MenuItemBase, newMenu } from './base'
import { invoke } from '../core'

/** Options for creating a new menu item. */
export interface MenuItemOptions {
  /** Specify an id to use for the new menu item. */
  id?: string
  /** The text of the new menu item. */
  text: string
  /** Whether the new menu item is enabled or not. */
  enabled?: boolean
  /** Specify an accelerator for the new menu item. */
  accelerator?: string
  /** Specify a handler to be called when this menu item is activated. */
  action?: (id: string) => void
}

/** A menu item inside a {@linkcode Menu} or {@linkcode Submenu} and contains only text. */
export class MenuItem extends MenuItemBase {
  /** @ignore */
  protected constructor(rid: number, id: string) {
    super(rid, id, 'MenuItem')
  }

  /** Create a new menu item. */
  static async new(opts: MenuItemOptions): Promise<MenuItem> {
    return newMenu('MenuItem', opts).then(([rid, id]) => new MenuItem(rid, id))
  }

  /** Returns the text of this menu item. */
  async text(): Promise<string> {
    return invoke('plugin:menu|text', { rid: this.rid, kind: this.kind })
  }

  /** Sets the text for this menu item. */
  async setText(text: string): Promise<void> {
    return invoke('plugin:menu|set_text', {
      rid: this.rid,
      kind: this.kind,
      text
    })
  }

  /** Returns whether this menu item is enabled or not. */
  async isEnabled(): Promise<boolean> {
    return invoke('plugin:menu|is_enabled', { rid: this.rid, kind: this.kind })
  }

  /** Sets whether this menu item is enabled or not. */
  async setEnabled(enabled: boolean): Promise<void> {
    return invoke('plugin:menu|set_enabled', {
      rid: this.rid,
      kind: this.kind,
      enabled
    })
  }

  /** Sets the accelerator for this menu item. */
  async setAccelerator(accelerator: string | null): Promise<void> {
    return invoke('plugin:menu|set_accelerator', {
      rid: this.rid,
      kind: this.kind,
      accelerator
    })
  }
}
