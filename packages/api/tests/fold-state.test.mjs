// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Run after building packages/api: node --test tests/fold-state.test.mjs
import test from 'node:test'
import assert from 'node:assert/strict'
import { getFoldState, onFoldStateChanged } from '../dist/app.js'

test('fold snapshot preserves unknown/unavailable and native states', async (t) => {
  t.after(() => {
    delete globalThis.window
  })
  for (const state of [
    null,
    { isFoldable: false, status: 'unknown' },
    { isFoldable: true, status: 'halfFolded' }
  ]) {
    globalThis.window = {
      __TAURI_INTERNALS__: {
        invoke: async (command) => {
          assert.equal(command, 'plugin:app|fold_state')
          return state
        }
      }
    }
    assert.deepEqual(await getFoldState(), state)
  }
})

test('fold listener forwards payloads and removes its native registration', async (t) => {
  t.after(() => {
    delete globalThis.window
  })
  let handler
  const calls = []
  const received = []
  globalThis.window = {
    __TAURI_INTERNALS__: {
      transformCallback: (callback) => {
        handler = callback
        return 42
      },
      invoke: async (command, args) => {
        calls.push([command, args])
        return 7
      }
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: {
      unregisterListener: (...args) => calls.push(['unregister', ...args])
    }
  }
  const stop = await onFoldStateChanged((event) => received.push(event.payload))
  assert.deepEqual(calls[0], [
    'plugin:event|listen',
    {
      event: 'tauri://fold-state-changed',
      target: { kind: 'Any' },
      handler: 42
    }
  ])
  assert.deepEqual(received, []) // No synthetic initial/expanded state.
  for (const payload of [{ isFoldable: true, status: 'folded' }, null]) {
    handler({ event: 'tauri://fold-state-changed', id: 7, payload })
  }
  assert.deepEqual(received, [{ isFoldable: true, status: 'folded' }, null])
  await stop()
  assert.deepEqual(calls.slice(1), [
    ['unregister', 'tauri://fold-state-changed', 7],
    [
      'plugin:event|unlisten',
      { event: 'tauri://fold-state-changed', eventId: 7 }
    ]
  ])
})

test('permission failures are not reported as ordinary or expanded devices', async (t) => {
  t.after(() => {
    delete globalThis.window
  })
  globalThis.window = {
    __TAURI_INTERNALS__: {
      invoke: async () => {
        throw new Error('permission denied')
      },
      transformCallback: () => 42
    }
  }
  await assert.rejects(getFoldState, /permission denied/)
  await assert.rejects(() => onFoldStateChanged(() => {}), /permission denied/)
})
