// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { MenuItemBase, newMenu } from './base'
import { invoke } from '../core'
import type { JsImage } from '../image'

/** A metadata for the about predefined menu item. */
export interface AboutMetadata {
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
  shortVersion?: string
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
  websiteLabel?: string
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
  icon?: JsImage
}

/** Options for creating a new predefined menu item. */
export interface PredefinedMenuItemOptions {
  /**
   * The text of the new predefined menu item.
   *
   * Defaults to the platform's own label for the chosen item, so only set it to
   * override or localize that label.
   */
  text?: string
  /**
   * The predefined item type.
   *
   * Each variant maps to a native menu item whose behavior is implemented by the
   * operating system, so there is no `action` to handle. Variants that are
   * unsupported on a platform still create an item, but it does nothing there.
   *
   * - `Separator`: a horizontal separator line. All platforms.
   * - `Copy`, `Cut`, `Paste`, `SelectAll`: the standard clipboard and selection
   *   commands, applied to the focused text input. All platforms.
   * - `Undo`, `Redo`: undo/redo in the focused text input. **macOS only**
   *   (*Windows / Linux:* unsupported).
   * - `Minimize`: minimizes the focused window. *Linux:* unsupported.
   * - `Maximize`: maximizes the focused window. *Linux:* unsupported.
   * - `Fullscreen`: toggles fullscreen for the focused window. **macOS only**
   *   (*Windows / Linux:* unsupported).
   * - `Hide`: hides the application. *Linux:* unsupported.
   * - `HideOthers`: hides every other application. *Linux:* unsupported.
   * - `ShowAll`: shows all hidden applications. **macOS only**
   *   (*Windows / Linux:* unsupported).
   * - `CloseWindow`: closes the focused window. *Linux:* unsupported.
   * - `Quit`: quits the application. *Linux:* unsupported.
   * - `Services`: the macOS *Services* submenu. **macOS only**
   *   (*Windows / Linux:* unsupported).
   * - `BringAllToFront`: brings all of the app's windows to the front. **macOS only**
   *   (*Windows / Linux:* unsupported).
   * - `{ About: AboutMetadata | null }`: opens an about dialog. All platforms; pass
   *   `null` to use the values from your `tauri.conf.json`, or see
   *   {@linkcode AboutMetadata} for the per-field platform support.
   *
   * @example
   * ```typescript
   * import { Menu, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';
   *
   * const edit = await Submenu.new({
   *   text: 'Edit',
   *   items: [
   *     { item: 'Undo' },
   *     { item: 'Redo' },
   *     { item: 'Separator' },
   *     { item: 'Cut' },
   *     { item: 'Copy' },
   *     { item: 'Paste' },
   *     { item: 'SelectAll' }
   *   ]
   * });
   *
   * const quit = await PredefinedMenuItem.new({ item: 'Quit', text: 'Quit my app' });
   * const menu = await Menu.new({ items: [edit, quit] });
   * ```
   */
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
    | 'BringAllToFront'
    | {
        About: AboutMetadata | null
      }
}

/** A predefined (native) menu item which has a predefined behavior by the OS or by tauri.  */
export class PredefinedMenuItem extends MenuItemBase {
  /** @ignore */
  protected constructor(rid: number, id: string) {
    super(rid, id, 'Predefined')
  }

  /**
   * Create a new predefined menu item.
   *
   * @example
   * ```typescript
   * import { PredefinedMenuItem } from '@tauri-apps/api/menu';
   * const separator = await PredefinedMenuItem.new({ item: 'Separator' });
   * ```
   */
  static async new(
    opts?: PredefinedMenuItemOptions
  ): Promise<PredefinedMenuItem> {
    return newMenu('Predefined', opts).then(
      ([rid, id]) => new PredefinedMenuItem(rid, id)
    )
  }

  /** Returns the text of this predefined menu item. */
  async text(): Promise<string> {
    return invoke('plugin:menu|text', { rid: this.rid, kind: this.kind })
  }

  /** Sets the text for this predefined menu item. */
  async setText(text: string): Promise<void> {
    return invoke('plugin:menu|set_text', {
      rid: this.rid,
      kind: this.kind,
      text
    })
  }
}
