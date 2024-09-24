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
          'Tauri-Invoke-Key': __TAURI_INVOKE_KEY__,
          ...((options && options.headers) || {})
        }
      })
        .then((response) => {
          const cb =
            response.headers.get('Tauri-Response') === 'ok' ? callback : error
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
        .catch((e) => {
          console.warn(
            'IPC custom protocol failed, Tauri will now use the postMessage interface instead',
            e
          )
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
        options: {
          ...options,
          customProtocolIpcBlocked: customProtocolIpcFailed
        },
        payload,
        __TAURI_INVOKE_KEY__
      })
      window.ipc.postMessage(data)
    }
  }

  Object.defineProperty(window.__TAURI_INTERNALS__, 'postMessage', {
    value: sendIpcMessage
  })
})()
