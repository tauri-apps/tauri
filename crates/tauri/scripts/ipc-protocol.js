// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  /**
   * A runtime generated key to ensure an IPC call comes from an initialized frame.
   *
   * This is declared outside the `window.__TAURI_INVOKE__` definition to prevent
   * the key from being leaked by `window.__TAURI_INVOKE__.toString()`.
   */
  const __TAURI_INVOKE_KEY__ = __TEMPLATE_invoke_key__

  const processIpcMessage = __RAW_process_ipc_message_fn__
  const osName = __TEMPLATE_os_name__
  const fetchChannelDataCommand = __TEMPLATE_fetch_channel_data_command__
  let customProtocolIpcFailed = false

  // on Android we never use it because Android does not have support to reading the request body
  const canUseCustomProtocol = osName !== 'android'

  function sendIpcMessage(message) {
    const { cmd, callback, error, payload, options } = message

    console.log(customProtocolIpcFailed, canUseCustomProtocol, cmd)
    if (
      !customProtocolIpcFailed
      && (canUseCustomProtocol || cmd === fetchChannelDataCommand)
    ) {
      const { contentType, data } = processIpcMessage(payload)

      const headers = new Headers((options && options.headers) || {})
      headers.set('Content-Type', contentType)
      headers.set('Tauri-Callback', callback)
      headers.set('Tauri-Error', error)
      headers.set('Tauri-Invoke-Key', __TAURI_INVOKE_KEY__)

      // For CEF runtime, try XMLHttpRequest first as it may populate PostData correctly
      // Fetch API has a known issue with custom schemes in CEF - XMLHttpRequest may work
      // Detect CEF: check for CEF-specific properties or user agent
      const useXHR = typeof window.cef !== 'undefined' ||
                      navigator.userAgent.includes('CEF') ||
                      navigator.userAgent.includes('Chromium Embedded')

      if (useXHR) {
        // Try XMLHttpRequest for CEF - it may handle POST bodies to custom schemes correctly
        try {
          const xhr = new XMLHttpRequest()
          const url = window.__TAURI_INTERNALS__.convertFileSrc(cmd, 'ipc')

          xhr.open('POST', url, true)

          // Set headers
          headers.forEach((value, key) => {
            xhr.setRequestHeader(key, value)
          })

          xhr.onload = function () {
            const callbackId = xhr.getResponseHeader('Tauri-Response') === 'ok' ? callback : error
            const contentType = (xhr.getResponseHeader('content-type') || '').split(',')[0]

            let responseData
            switch (contentType) {
              case 'application/json':
                try {
                  responseData = JSON.parse(xhr.responseText)
                } catch {
                  responseData = xhr.responseText
                }
                break
              case 'text/plain':
                responseData = xhr.responseText
                break
              default:
                responseData = xhr.response
                break
            }

            window.__TAURI_INTERNALS__.runCallback(callbackId, responseData)
          }

          xhr.onerror = function () {
            console.warn(
              'IPC custom protocol failed with XMLHttpRequest, trying Fetch API',
              xhr.status
            )
            // Fall through to Fetch API
            sendWithFetch()
          }

          xhr.send(data)
          return
        } catch (e) {
          console.warn('XMLHttpRequest failed, falling back to Fetch API', e)
          // Fall through to Fetch API
        }
      }

      function sendWithFetch() {
        fetch(window.__TAURI_INTERNALS__.convertFileSrc(cmd, 'ipc'), {
          method: 'POST',
          body: data,
          headers
        })
          .then((response) => {
            const callbackId =
              response.headers.get('Tauri-Response') === 'ok' ? callback : error
            // we need to split here because on Android the content-type gets duplicated
            switch ((response.headers.get('content-type') || '').split(',')[0]) {
              case 'application/json':
                return response.json().then((r) => [callbackId, r])
              case 'text/plain':
                return response.text().then((r) => [callbackId, r])
              default:
                return response.arrayBuffer().then((r) => [callbackId, r])
            }
          })
          .then(
            ([callbackId, data]) => {
              window.__TAURI_INTERNALS__.runCallback(callbackId, data)
            },
            (e) => {
              console.warn(
                'IPC custom protocol failed, Tauri will now use the postMessage interface instead',
                e
              )
              // failed to use the custom protocol IPC (either the webview blocked a custom protocol or it was a CSP error)
              // so we need to fallback to the postMessage interface
              customProtocolIpcFailed = true
              sendIpcMessage(message)
            }
          )
      }

      sendWithFetch()
    } else {
      // otherwise use the postMessage interface
      const { data } = processIpcMessage({
        cmd,
        callback,
        error,
        options: {
          ...options,
          customProtocolIpcBlocked: customProtocolIpcFailed
        },
        payload,
        __TAURI_INVOKE_KEY__
      })
      // `window.ipc.postMessage` came from `tauri-runtime-wry` > `wry` [`with_ipc_handler`](https://github.com/tauri-apps/wry/blob/a0403b9e2f1ff9d73be7dce1184f058afcaa1d82/src/lib.rs#L1130)
      window.ipc.postMessage(data)
    }
  }

  Object.defineProperty(window.__TAURI_INTERNALS__, 'postMessage', {
    value: sendIpcMessage
  })
})()
