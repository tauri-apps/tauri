import {
  type CheckMenuItem,
  type IconMenuItem,
  type MenuItem,
  type MenuItemOptions,
  type PredefinedMenuItem
} from '../menu'
import { invoke } from '../primitives'
import { type LogicalPosition, PhysicalPosition, type Window } from '../window'
import { type ItemKind, MenuItemBase, itemFromKind, newMenu } from './base'
import { type MenuOptions } from './menu'

export type SubmenuOptions = Omit<MenuItemOptions, 'accelerator' | 'action'> &
  MenuOptions

/** A type that is a submenu inside a {@linkcode Menu} or {@linkcode Submenu}. */
export class Submenu extends MenuItemBase {
  /** @ignore */
  protected constructor(rid: number, id: string) {
    super(rid, id, 'Submenu')
  }

  /** Create a new submenu. */
  static async new(opts: SubmenuOptions): Promise<Submenu> {
    return newMenu('Submenu', opts).then(([rid, id]) => new Submenu(rid, id))
  }

  /** Returns the text of this submenu. */
  async text(): Promise<string> {
    return invoke('plugin:menu|text', { rid: this.rid, kind: this.kind })
  }

  /** Sets the text for this submenu. */
  async setText(text: string): Promise<void> {
    return invoke('plugin:menu|set_text', {
      rid: this.rid,
      kind: this.kind,
      text
    })
  }

  /** Returns whether this submenu is enabled or not. */
  async isEnabled(): Promise<boolean> {
    return invoke('plugin:menu|is_enabled', { rid: this.rid, kind: this.kind })
  }

  /** Sets whether this submenu is enabled or not. */
  async setEnabled(enabled: boolean): Promise<void> {
    return invoke('plugin:menu|set_enabled', {
      rid: this.rid,
      kind: this.kind,
      enabled
    })
  }

  /**
   * Add a menu item to the end of this submenu.
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
   * Add a menu item to the beginning of this submenu.
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
   * Add a menu item to the specified position in this submenu.
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

  /** Remove a menu item from this submenu. */
  async remove(
    item: Submenu | MenuItem | PredefinedMenuItem | CheckMenuItem | IconMenuItem
  ): Promise<void> {
    return invoke('plugin:menu|remove', {
      rid: this.rid,
      kind: this.kind,
      item: [item.rid, item.kind]
    })
  }

  /** Remove a menu item from this submenu at the specified position. */
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

  /** Returns a list of menu items that has been added to this submenu. */
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

  /**
   * Popup this submenu as a context menu on the specified window.
   *
   * If the position, is provided, it is relative to the window's top-left corner.
   */
  async popup(
    at?: PhysicalPosition | LogicalPosition,
    window?: Window
  ): Promise<void> {
    let atValue = null
    if (at) {
      atValue = {
        type: at instanceof PhysicalPosition ? 'Physical' : 'Logical',
        data: at
      }
    }
    return invoke('plugin:menu|popup', {
      rid: this.rid,
      kind: this.kind,
      window: window?.label ?? null,
      at: atValue
    })
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
