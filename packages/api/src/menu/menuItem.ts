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
  /**
   * Specify an accelerator (keyboard shortcut) for the new menu item, for example
   * `'CmdOrCtrl+Shift+K'`.
   *
   * The string is a list of zero or more modifiers followed by exactly one key,
   * joined with `+`. Matching is case insensitive and spaces around each token are
   * ignored, so `'CmdOrCtrl+Shift+K'` and `'cmdorctrl + shift + k'` are equivalent.
   * All modifiers must come before the key.
   *
   * Accepted modifiers:
   *
   * - `Shift`
   * - `Control`, `Ctrl`
   * - `Alt`, `Option`
   * - `Command`, `Cmd`, `Super`
   * - `CmdOrCtrl`, `CmdOrControl`, `CommandOrCtrl`, `CommandOrControl` — `Command`
   *   on macOS and `Control` everywhere else, which is what you usually want for
   *   application shortcuts.
   *
   * Accepted keys:
   *
   * - Letters `A`-`Z` (also written `KeyA`-`KeyZ`) and digits `0`-`9` (also `Digit0`-`Digit9`).
   * - Punctuation, either as the character or by name: `` ` ``/`Backquote`, `\`/`Backslash`,
   *   `[`/`BracketLeft`, `]`/`BracketRight`, `,`/`Comma`, `=`/`Equal`, `-`/`Minus`,
   *   `.`/`Period`, `'`/`Quote`, `;`/`Semicolon`, `/`/`Slash`.
   * - `Backspace`, `CapsLock`, `Enter`, `Space`, `Tab`, `Delete`, `End`, `Home`,
   *   `Insert`, `PageDown`, `PageUp`, `PrintScreen`, `ScrollLock`, `NumLock`,
   *   `Escape` (also `Esc`).
   * - Arrows: `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight` (also `Up`, `Down`, `Left`, `Right`).
   * - Function keys `F1` through `F24`.
   * - Numpad keys: `Numpad0`-`Numpad9` (also `Num0`-`Num9`), `NumpadAdd`, `NumpadSubtract`,
   *   `NumpadMultiply`, `NumpadDivide`, `NumpadDecimal`, `NumpadEnter`, `NumpadEqual`
   *   (each also accepted with the `Num` prefix, e.g. `NumAdd`).
   * - Media keys: `AudioVolumeUp`, `AudioVolumeDown`, `AudioVolumeMute` (also `VolumeUp`,
   *   `VolumeDown`, `VolumeMute`).
   *
   * @example
   * ```typescript
   * import { MenuItem } from '@tauri-apps/api/menu';
   *
   * await MenuItem.new({ text: 'Find', accelerator: 'CmdOrCtrl+F' });
   * await MenuItem.new({ text: 'Command palette', accelerator: 'CmdOrCtrl+Shift+P' });
   * await MenuItem.new({ text: 'Refresh', accelerator: 'F5' });
   * ```
   *
   * An accelerator that cannot be parsed is ignored and the item is created
   * without a shortcut, so double-check the spelling of the modifiers and key.
   */
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

  /**
   * Sets the accelerator for this menu item, or removes it when given `null`.
   *
   * See {@linkcode MenuItemOptions.accelerator} for the accepted format.
   *
   * @example
   * ```typescript
   * await item.setAccelerator('CmdOrCtrl+Shift+K');
   * ```
   */
  async setAccelerator(accelerator: string | null): Promise<void> {
    return invoke('plugin:menu|set_accelerator', {
      rid: this.rid,
      kind: this.kind,
      accelerator
    })
  }
}
