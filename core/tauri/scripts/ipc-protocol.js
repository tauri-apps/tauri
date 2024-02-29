// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  const processIpcMessage = __RAW_process_ipc_message_fn__
  const osName = __TEMPLATE_os_name__
  const fetchChannelDataCommand = __TEMPLATE_fetch_channel_data_command__
  const linuxIpcProtocolEnabled = __TEMPLATE_linux_ipc_protocol_enabled__
  let customProtocolIpcFailed = false

  // on Linux we only use the custom-protocol-based IPC if the the linux-ipc-protocol Cargo feature is enabled
  // on Android we never use it because Android does not have support to reading the request body
  const canUseCustomProtocol =
    osName === 'linux' ? linuxIpcProtocolEnabled : osName !== 'android'

  function sendIpcMessage(message) {
    const { cmd, callback, error, payload, options } = message

    if (
      !customProtocolIpcFailed &&
      (canUseCustomProtocol || cmd === fetchChannelDataCommand)
    ) {
      const { contentType, data } = processIpcMessage(payload)
      fetch(window.__TAURI_INTERNALS__.convertFileSrc(cmd, 'ipc'), {
        method: 'POST',
        body: data,
        headers: {
          'Content-Type': contentType,
          'Tauri-Callback': callback,
          'Tauri-Error': error,
          ...options?.headers
        }
      })
        .then((response) => {
          const cb = response.ok ? callback : error
          // we need to split here because on Android the content-type gets duplicated
          switch ((response.headers.get('content-type') || '').split(',')[0]) {
            case 'application/json':
              return response.json().then((r) => [cb, r])
            case 'text/plain':
              return response.text().then((r) => [cb, r])
            default:
              return response.arrayBuffer().then((r) => [cb, r])
          }
        })
        .then(([cb, data]) => {
          if (window[`_${cb}`]) {
            window[`_${cb}`](data)
          } else {
            console.warn(
              `[TAURI] Couldn't find callback id {cb} in window. This might happen when the app is reloaded while Rust is running an asynchronous operation.`
            )
          }
        })
        .catch(() => {
          // failed to use the custom protocol IPC (either the webview blocked a custom protocol or it was a CSP error)
          // so we need to fallback to the postMessage interface
          customProtocolIpcFailed = true
          sendIpcMessage(message)
        })
    } else {
      // otherwise use the postMessage interface
      const { data } = processIpcMessage({
        cmd,
        callback,
        error,
        options,
        payload
      })
      window.ipc.postMessage(data)
    }
  }

  Object.defineProperty(window.__TAURI_INTERNALS__, 'postMessage', {
    value: sendIpcMessage
  })
})()
