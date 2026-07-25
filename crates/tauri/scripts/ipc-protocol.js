// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  const processIpcMessage = __RAW_process_ipc_message_fn__
  const osName = __TEMPLATE_os_name__
  const fetchChannelDataCommand = __TEMPLATE_fetch_channel_data_command__
  let customProtocolIpcFailed = null

  // on Android we never use it because Android does not have support to reading the request body
  const canUseCustomProtocol = osName !== 'android'

  function sendIpcMessage(message) {

    if (canUseCustomProtocol && !customProtocolIpcFailed) {
      const { cmd, callback, error, payload, options } = message

      const headers = {
        'Tauri-Callback': callback,
        'Tauri-Error': error
      }

      if (options?.headers) {
        Object.assign(headers, options.headers)
      }

      fetch(
        window.__TAURI_INTERNALS__.postMessage
          ? 'http://ipc.localhost'
          : 'https://ipc.localhost',
        {
          method: 'POST',
          body: processIpcMessage(payload),
          headers
        }
      )
        .then((response) => {
          customProtocolIpcFailed = false
          const callbackId =
            response.headers.get('Tauri-Response') === 'ok' ? callback : error
          // we need to split here because on Android the content-type gets duplicated
          const contentType = response.headers
            .get('content-type')
            ?.split(',')[0]
            .trim()
          if (contentType === 'application/json') {
            return response.json().then((data) => [callbackId, data])
          } else {
            return response.arrayBuffer().then((data) => [callbackId, data])
          }
        })
        .catch((e) => {
          if (customProtocolIpcFailed !== false) {
            console.warn(
              'IPC custom protocol failed, Tauri will now use the postMessage interface instead',
              e
            )
            // failed to use the custom protocol IPC (either the webview blocked a custom protocol or it was a CSP error)
            // so we need to fallback to the postMessage interface
            customProtocolIpcFailed = true
            sendIpcMessage(message)
          } else {
            throw e
          }
        })
        .then(([callbackId, data]) => {
          window.__TAURI_INTERNALS__.runCallback(callbackId, data)
        })
    } else {
      window.__TAURI_INTERNALS__.postMessage(message)
    }
  }

  window.__TAURI_INTERNALS__.invoke = function (cmd, payload = {}, options) {
    return new Promise((resolve, reject) => {
      const callback = window.__TAURI_INTERNALS__.transformCallback(
        (response) => {
          resolve(response)
        },
        true
      )
      const error = window.__TAURI_INTERNALS__.transformCallback(
        (response) => {
          reject(response)
        },
        true
      )

      sendIpcMessage({
        cmd,
        callback,
        error,
        payload,
        options
      })
    })
  }

  // send raw message to IPC
  window.__TAURI_INTERNALS__.postIpcMessage = sendIpcMessage

  window.__TAURI_INTERNALS__.fetchChannelData = function (channelId) {
    return window.__TAURI_INTERNALS__.invoke(fetchChannelDataCommand, {
      channelId
    })
  }
})()
