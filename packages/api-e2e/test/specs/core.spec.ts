// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describeApi } from '../helpers/index.js'

describeApi('core', () => {
  it('isTauri reports true inside the app', async () => {
    expect(await tauri((api) => api.core.isTauri())).toBe(true)
  })

  it('invoke round-trips a raw request through the echo command', async () => {
    const echoed = await tauri((api) =>
      api.core.invoke('echo', { some: 'payload', n: 42 })
    )
    expect(echoed).toEqual({ some: 'payload', n: 42 })
  })

  it('invoke rejects for a command that is not registered', async () => {
    // `removeUnusedCommands` strips ungranted commands from the app entirely.
    const message = await tauriError((api) =>
      api.core.invoke('this_command_does_not_exist')
    )
    expect(message).toMatch(/not found|not allowed|denied/i)
  })

  it('command scopes gate log_operation by event', async () => {
    // The run-app capability scopes `log_operation` to the `tauri-click` event.
    await tauri((api) =>
      api.core.invoke('log_operation', {
        event: 'tauri-click',
        payload: 'from e2e'
      })
    )
    const message = await tauriError((api) =>
      api.core.invoke('log_operation', { event: 'out-of-scope-event' })
    )
    expect(message).toMatch(/not allowed|denied/i)
  })

  it('Channel receives every message streamed from the backend in order', async () => {
    // `spam` sends 1..=1000 over the channel.
    const result = await tauri(
      (api, count) =>
        new Promise<{ length: number; ordered: boolean; last: number }>(
          (resolve, reject) => {
            const received: number[] = []
            const channel = new api.core.Channel<number>()
            channel.onmessage = (value: number) => {
              received.push(value)
              if (received.length === count) {
                let ordered = true
                for (let i = 0; i < received.length; i++) {
                  if (received[i] !== i + 1) {
                    ordered = false
                    break
                  }
                }
                resolve({
                  length: received.length,
                  ordered,
                  last: received[received.length - 1]
                })
              }
            }
            api.core.invoke('spam', { channel }).catch(reject)
            setTimeout(
              () =>
                reject(
                  new Error(
                    `only received ${received.length} of ${count} messages`
                  )
                ),
              15000
            )
          }
        ),
      1000
    )
    expect(result.length).toBe(1000)
    expect(result.ordered).toBe(true)
    expect(result.last).toBe(1000)
  })

  it('convertFileSrc builds a platform-appropriate asset URL', async () => {
    const url = await tauri((api) =>
      api.core.convertFileSrc('/path/to/file.png')
    )
    expect(url).toMatch(/(asset:\/\/localhost\/|https?:\/\/asset\.localhost\/)/)
  })
})
