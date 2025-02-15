// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { Channel, invoke, Resource } from '../core'
import { transformImage } from '../image'
import { CheckMenuItemOptions } from './checkMenuItem'
import { IconMenuItemOptions } from './iconMenuItem'
import { MenuOptions } from './menu'
import { MenuItemOptions } from './menuItem'
import { PredefinedMenuItemOptions } from './predefinedMenuItem'
import { SubmenuOptions } from './submenu'

export type ItemKind =
  | 'MenuItem'
  | 'Predefined'
  | 'Check'
  | 'Icon'
  | 'Submenu'
  | 'Menu'

type MenuItemOptionsAlias =
  | MenuItemOptions
  | SubmenuOptions
  | IconMenuItemOptions
  | PredefinedMenuItemOptions
  | CheckMenuItemOptions

function injectChannel(i: MenuItemOptionsAlias): MenuItemOptionsAlias & {
  handler?: Channel<string>
} {
  if ('items' in i) {
    i.items = i.items?.map((item) =>
      'rid' in item ? item : injectChannel(item)
    )
  } else if ('action' in i && i.action) {
    const handler = new Channel<string>()
    handler.onmessage = i.action
    delete i.action
    return { ...i, handler }
  }
  return i
}

export async function newMenu(
  kind: ItemKind,
  opts?:
    | MenuOptions
    | MenuItemOptions
    | SubmenuOptions
    | PredefinedMenuItemOptions
    | CheckMenuItemOptions
    | IconMenuItemOptions
): Promise<[number, string]> {
  const handler = new Channel<string>()

  if (opts && typeof opts === 'object') {
    if ('action' in opts && opts.action) {
      handler.onmessage = opts.action as () => void
      delete opts.action
    }

    // about predefined menu item icon
    if (
      'item' in opts &&
      opts.item &&
      typeof opts.item === 'object' &&
      'About' in opts.item &&
      opts.item.About &&
      typeof opts.item.About === 'object' &&
      'icon' in opts.item.About &&
      opts.item.About.icon
    ) {
      opts.item.About.icon = transformImage(opts.item.About.icon)
    }

    // icon menu item icon
    if ('icon' in opts && opts.icon) {
      opts.icon = transformImage(opts.icon)
    }

    // submenu items
    if ('items' in opts && opts.items) {
      function prepareItem(
        i: { rid: number; kind: string } | MenuItemOptionsAlias
      ): [number, string] | MenuItemOptionsAlias {
        if ('rid' in i) {
          return [i.rid, i.kind]
        }

        if ('item' in i && typeof i.item === 'object' && i.item.About?.icon) {
          i.item.About.icon = transformImage(i.item.About.icon)
        }

        if ('icon' in i && i.icon) {
          i.icon = transformImage(i.icon)
        }

        if ('items' in i && i.items) {
          // @ts-expect-error the `prepareItem` return doesn't exactly match
          // this is fine, because the difference is in `[number, string]` variant
          i.items = i.items.map(prepareItem)
        }

        return injectChannel(i)
      }

      // @ts-expect-error the `prepareItem` return doesn't exactly match
      // this is fine, because the difference is in `[number, string]` variant
      opts.items = (opts.items as []).map(prepareItem)
    }
  }

  return invoke('plugin:menu|new', {
    kind,
    options: opts,
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
