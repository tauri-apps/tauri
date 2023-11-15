import { Resource } from '../internal'
import { Channel, invoke } from '../primitives'
import { CheckMenuItem } from './checkMenuItem'
import { IconMenuItem, type IconMenuItemOptions } from './iconMenuItem'
import { type MenuOptions } from './menu'
import { MenuItem, type MenuItemOptions } from './menuItem'
import {
  PredefinedMenuItem,
  type PredefinedMenuItemOptions
} from './predefinedMenuItem'
import { Submenu, type SubmenuOptions } from './submenu'

export type ItemKind =
  | 'MenuItem'
  | 'Predefined'
  | 'Check'
  | 'Icon'
  | 'Submenu'
  | 'Menu'

export function itemFromKind([rid, id, kind]: [number, string, ItemKind]):
  | Submenu
  | MenuItem
  | PredefinedMenuItem
  | CheckMenuItem
  | IconMenuItem {
  /* eslint-disable @typescript-eslint/no-unsafe-return */
  switch (kind) {
    case 'Submenu':
      // @ts-expect-error constructor is protected for external usage only
      return new Submenu(rid, id)
    case 'Predefined':
      // @ts-expect-error constructor is protected for external usage only
      return new PredefinedMenuItem(rid, id)
    case 'Check':
      // @ts-expect-error constructor is protected for external usage only
      return new CheckMenuItem(rid, id)
    case 'Icon':
      // @ts-expect-error constructor is protected for external usage only
      return new IconMenuItem(rid, id)
    case 'MenuItem':
    default:
      // @ts-expect-error constructor is protected for external usage only
      return new MenuItem(rid, id)
  }
  /* eslint-enable @typescript-eslint/no-unsafe-return */
}

export async function newMenu(
  kind: ItemKind,
  opts?:
    | MenuOptions
    | MenuItemOptions
    | SubmenuOptions
    | PredefinedMenuItemOptions
    | IconMenuItemOptions
): Promise<[number, string]> {
  const handler = new Channel<string>()
  let items: null | Array<[number, string]> = null
  if (opts) {
    if ('action' in opts && opts.action) {
      handler.onmessage = opts.action
      delete opts.action
    }

    if ('items' in opts && opts.items) {
      items = opts.items.map((i) => [i.rid, i.kind])
    }
  }

  return invoke('plugin:menu|new', {
    kind,
    options: opts ? { ...opts, items } : undefined,
    handler
  })
}

export class MenuItemBase extends Resource {
  /** @ignore */
  readonly #id: string
  /** @ignore */
  readonly #kind: ItemKind

  /** The id of this item. */
  get id(): string {
    return this.#id
  }

  /** @ignore */
  get kind(): string {
    return this.#kind
  }

  /** @ignore */
  protected constructor(rid: number, id: string, kind: ItemKind) {
    super(rid)
    this.#id = id
    this.#kind = kind
  }
}
