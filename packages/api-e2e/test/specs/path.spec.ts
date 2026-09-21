// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import path from 'node:path'
import { expect } from '@wdio/globals'
import { tauri, describeApi } from '../helpers/index.js'

describeApi('path', () => {
  it('sep and delimiter match the host platform', async () => {
    const separators = await tauri((api) => ({
      sep: api.path.sep(),
      delimiter: api.path.delimiter()
    }))
    expect(separators.sep).toBe(path.sep)
    expect(separators.delimiter).toBe(path.delimiter)
  })

  it('join, dirname, basename and extname behave like the platform', async () => {
    const result = await tauri(async (api) => ({
      joined: await api.path.join('a', 'b', 'c.txt'),
      dir: await api.path.dirname(await api.path.join('a', 'b', 'c.txt')),
      base: await api.path.basename(await api.path.join('a', 'b', 'c.txt')),
      ext: await api.path.extname(await api.path.join('a', 'b', 'c.txt'))
    }))
    expect(result.joined).toBe(path.join('a', 'b', 'c.txt'))
    expect(result.base).toBe('c.txt')
    expect(result.ext).toBe('txt')
    expect(result.dir.endsWith(path.join('a', 'b'))).toBe(true)
  })

  it('normalize and isAbsolute work', async () => {
    const result = await tauri(async (api) => ({
      normalized: await api.path.normalize('a/b/../c'),
      absoluteIsAbsolute: await api.path.isAbsolute(
        await api.path.resolve('x')
      ),
      relativeIsAbsolute: await api.path.isAbsolute('a/b')
    }))
    expect(result.normalized).toContain('c')
    expect(result.absoluteIsAbsolute).toBe(true)
    expect(result.relativeIsAbsolute).toBe(false)
  })

  it('app directories are absolute and scoped to the identifier', async () => {
    const dirs = await tauri(async (api) => ({
      appData: await api.path.appDataDir(),
      appConfig: await api.path.appConfigDir(),
      home: await api.path.homeDir(),
      temp: await api.path.tempDir()
    }))
    expect(dirs.appData).toContain('com.tauri.api')
    expect(dirs.appConfig.length).toBeGreaterThan(0)
    expect(dirs.home.length).toBeGreaterThan(0)
    expect(dirs.temp.length).toBeGreaterThan(0)
  })
})
