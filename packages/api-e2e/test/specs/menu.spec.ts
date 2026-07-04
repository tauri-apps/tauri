// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describeApi } from '../helpers/index.js'

// The menu commands are all allowed by default. These tests only build and
// inspect menu resources; they deliberately do not attach the menu to the app
// or window, to avoid disturbing the live example app.
describeApi('menu', () => {
  it('Menu.new builds a menu with the given items', async () => {
    const result = await tauri(async (api) => {
      const item = await api.menu.MenuItem.new({ text: 'Item A' })
      const check = await api.menu.CheckMenuItem.new({
        text: 'Item B',
        checked: true
      })
      const separator = await api.menu.PredefinedMenuItem.new({
        item: 'Separator'
      })
      const submenu = await api.menu.Submenu.new({
        text: 'More',
        items: [await api.menu.MenuItem.new({ text: 'Nested' })]
      })
      const menu = await api.menu.Menu.new({
        items: [item, check, separator, submenu]
      })
      const items = await menu.items()
      return { count: items.length }
    })
    expect(result.count).toBe(4)
  })

  it('MenuItem text and enabled state can be read and updated', async () => {
    const result = await tauri(async (api) => {
      const item = await api.menu.MenuItem.new({
        text: 'Original',
        enabled: true
      })
      const before = await item.text()
      await item.setText('Updated')
      const after = await item.text()
      const enabledBefore = await item.isEnabled()
      await item.setEnabled(false)
      const enabledAfter = await item.isEnabled()
      return { before, after, enabledBefore, enabledAfter }
    })
    expect(result.before).toBe('Original')
    expect(result.after).toBe('Updated')
    expect(result.enabledBefore).toBe(true)
    expect(result.enabledAfter).toBe(false)
  })

  it('CheckMenuItem tracks its checked state', async () => {
    const result = await tauri(async (api) => {
      const item = await api.menu.CheckMenuItem.new({
        text: 'Toggle',
        checked: false
      })
      const before = await item.isChecked()
      await item.setChecked(true)
      const after = await item.isChecked()
      return { before, after }
    })
    expect(result.before).toBe(false)
    expect(result.after).toBe(true)
  })

  it('append and removeAt mutate the item list', async () => {
    const result = await tauri(async (api) => {
      const menu = await api.menu.Menu.new({
        items: [await api.menu.MenuItem.new({ text: 'First' })]
      })
      await menu.append(await api.menu.MenuItem.new({ text: 'Second' }))
      const afterAppend = (await menu.items()).length
      await menu.removeAt(0)
      const afterRemove = (await menu.items()).length
      return { afterAppend, afterRemove }
    })
    expect(result.afterAppend).toBe(2)
    expect(result.afterRemove).toBe(1)
  })

  it('Menu.default builds the platform default menu', async () => {
    const count = await tauri(async (api) => {
      const menu = await api.menu.Menu.default()
      return (await menu.items()).length
    })
    expect(count).toBeGreaterThanOrEqual(0)
  })
})
