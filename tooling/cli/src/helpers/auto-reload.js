// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/thedodd/trunk/blob/5c799dc35f1f1d8f8d3d30c8723cbb761a9b6a08/src/autoreload.js

;(function () {
  var url = '{{reload_url}}'
  var poll_interval = 5000
  var reload_upon_connect = () => {
    window.setTimeout(() => {
      // when we successfully reconnect, we'll force a
      // reload (since we presumably lost connection to
      // trunk due to it being killed, so it will have
      // rebuilt on restart)
      var ws = new WebSocket(url)
      ws.onopen = () => window.location.reload()
      ws.onclose = reload_upon_connect
    }, poll_interval)
  }

  var ws = new WebSocket(url)
  ws.onmessage = (ev) => {
    const msg = JSON.parse(ev.data)
    if (msg.reload) {
      window.location.reload()
    }
  }
  ws.onclose = reload_upon_connect
})()
