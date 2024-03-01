// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { Channel, invoke, Resource } from '../core'
import { CheckMenuItemOptions } from './checkMenuItem'
import { IconMenuItemOptions } from './iconMenuItem'
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

function injectChannel(
  i:
    | MenuItemOptions
    | SubmenuOptions
    | IconMenuItemOptions
    | PredefinedMenuItemOptions
    | CheckMenuItemOptions
):
  | MenuItemOptions
  | SubmenuOptions
  | IconMenuItemOptions
  | PredefinedMenuItemOptions
  | (CheckMenuItemOptions & { handler?: Channel<string> }) {
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
  opts?: unknown
): Promise<[number, string]> {
  const handler = new Channel<string>()
  let items: null | Array<
    | [number, string]
    | MenuItemOptions
    | SubmenuOptions
    | IconMenuItemOptions
    | PredefinedMenuItemOptions
    | CheckMenuItemOptions
  > = null
  if (opts && typeof opts === 'object') {
    if ('action' in opts && opts.action) {
      handler.onmessage = opts.action as () => void
      delete opts.action
    }

    if ('items' in opts && opts.items) {
      items = (
        opts.items as Array<
          | { rid: number; kind: string }
          | MenuItemOptions
          | SubmenuOptions
          | IconMenuItemOptions
          | PredefinedMenuItemOptions
          | CheckMenuItemOptions
        >
      ).map((i) => {
        if ('rid' in i) {
          return [i.rid, i.kind]
        }
        return injectChannel(i)
      })
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
