// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const assert = require('node:assert/strict')
const fs = require('node:fs')
const path = require('node:path')
const { test } = require('node:test')
const vm = require('node:vm')

const source = fs.readFileSync(path.join(__dirname, 'zoom-hotkey.js'), 'utf8')

for (const os of ['linux', 'macos']) {
  for (const input of ['keyboard', 'wheel']) {
    test(`${os} ${input} sends relative changes across document recreation`, () => {
      const calls = []
      function document() {
        const listeners = {}
        const window = {
          addEventListener: (name, handler) => {
            listeners[name] = handler
          },
          __TAURI_INTERNALS__: {
            invoke: (command, args) => {
              calls.push([command, args.value])
              return Promise.resolve()
            }
          }
        }
        vm.runInNewContext(
          source.replace('__TEMPLATE_os_name__', JSON.stringify(os)),
          { window }
        )
        return (direction) => {
          if (input === 'keyboard') {
            listeners.keydown({
              key: direction === 0 ? '0' : direction > 0 ? '=' : '-',
              ctrlKey: os !== 'macos',
              metaKey: os === 'macos'
            })
          } else {
            listeners.mousewheel({
              ctrlKey: true,
              deltaY: -direction,
              preventDefault() {}
            })
          }
        }
      }
      const first = document()
      first(1)
      first(1)
      const reloaded = document()
      reloaded(1)
      reloaded(-1)
      assert.deepEqual(JSON.parse(JSON.stringify(calls)), [
        ['plugin:webview|set_webview_zoom', { delta: 0.2 }],
        ['plugin:webview|set_webview_zoom', { delta: 0.2 }],
        ['plugin:webview|set_webview_zoom', { delta: 0.2 }],
        ['plugin:webview|set_webview_zoom', { delta: -0.2 }]
      ])
      if (input === 'keyboard') {
        reloaded(0)
        assert.equal(calls.at(-1)[1], 1)
      }
    })
  }
}
