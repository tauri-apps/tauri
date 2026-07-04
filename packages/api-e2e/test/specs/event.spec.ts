// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describeApi } from '../helpers/index.js'

describeApi('event', () => {
  it('listen receives an emitted payload', async () => {
    const payload = await tauri(
      (api) =>
        new Promise<{ hello: string }>((resolve, reject) => {
          api.event
            .listen<{ hello: string }>('e2e://ping', (event) =>
              resolve(event.payload)
            )
            .then((unlisten) => {
              api.event.emit('e2e://ping', { hello: 'world' }).catch(reject)
              setTimeout(() => {
                unlisten()
                reject(new Error('event was not received'))
              }, 5000)
            })
            .catch(reject)
        })
    )
    expect(payload).toEqual({ hello: 'world' })
  })

  it('once fires exactly once even if the event is emitted twice', async () => {
    const count = await tauri(
      (api) =>
        new Promise<number>((resolve, reject) => {
          let calls = 0
          api.event
            .once('e2e://once', () => {
              calls++
            })
            .then(async () => {
              await api.event.emit('e2e://once')
              await api.event.emit('e2e://once')
              setTimeout(() => resolve(calls), 1000)
            })
            .catch(reject)
        })
    )
    expect(count).toBe(1)
  })

  it('unlisten stops further delivery', async () => {
    const count = await tauri(
      (api) =>
        new Promise<number>((resolve, reject) => {
          let calls = 0
          api.event
            .listen('e2e://off', () => {
              calls++
            })
            .then(async (unlisten) => {
              await api.event.emit('e2e://off')
              setTimeout(async () => {
                await unlisten()
                await api.event.emit('e2e://off')
                setTimeout(() => resolve(calls), 500)
              }, 500)
            })
            .catch(reject)
        })
    )
    expect(count).toBe(1)
  })

  it('emitTo delivers only to the targeted label', async () => {
    const result = await tauri(
      (api) =>
        new Promise<{ mainCount: number }>((resolve, reject) => {
          let mainCount = 0
          api.event
            .listen('e2e://target', () => {
              mainCount++
            })
            .then(async (unlisten) => {
              await api.event.emitTo('main', 'e2e://target') // delivered
              await api.event.emitTo('does-not-exist', 'e2e://target') // dropped
              setTimeout(async () => {
                await unlisten()
                resolve({ mainCount })
              }, 800)
            })
            .catch(reject)
        })
    )
    expect(result.mainCount).toBe(1)
  })

  it('bridges events to and from the Rust backend', async () => {
    // examples/api listens for `js-event` and replies with `rust-event { data }`.
    const reply = await tauri(
      (api) =>
        new Promise<{ data: string }>((resolve, reject) => {
          api.event
            .listen<{ data: string }>('rust-event', (event) =>
              resolve(event.payload)
            )
            .then((unlisten) => {
              api.event.emit('js-event', 'hello from e2e').catch(reject)
              setTimeout(() => {
                unlisten()
                reject(new Error('no rust-event reply received'))
              }, 5000)
            })
            .catch(reject)
        })
    )
    expect(reply).toEqual({ data: 'something else' })
  })
})
