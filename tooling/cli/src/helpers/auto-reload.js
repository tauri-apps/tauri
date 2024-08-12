// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/thedodd/trunk/blob/5c799dc35f1f1d8f8d3d30c8723cbb761a9b6a08/src/autoreload.js

;(function () {
  const reload_url = '{{reload_url}}'
  const url = reload_url ? reload_url : window.location.href
  const poll_interval = 5000
  const reload_upon_connect = () => {
    window.setTimeout(() => {
      // when we successfully reconnect, we'll force a
      // reload (since we presumably lost connection to
      // tauri-cli due to it being killed)
      const ws = new WebSocket(url)
      ws.onopen = () => window.location.reload()
      ws.onclose = reload_upon_connect
    }, poll_interval)
  }

  const ws = new WebSocket(url)
  ws.onmessage = (ev) => {
    const msg = JSON.parse(ev.data)
    if (msg.reload) {
      window.location.reload()
    }
  }
  ws.onclose = reload_upon_connect
})()
