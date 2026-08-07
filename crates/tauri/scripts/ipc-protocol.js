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

  // Tracks whether the document is being torn down (a reload or navigation).
  // When that happens, any in-flight custom protocol `fetch` is aborted by the
  // webview and its promise rejects. We must not treat that as a custom
  // protocol failure and re-send the message over the postMessage fallback,
  // because the command may already be running on the backend and would be
  // invoked a second time. See https://github.com/tauri-apps/tauri/issues/14154
  let documentIsUnloading = false
  const markUnloading = () => {
    documentIsUnloading = true
  }
  // `pagehide` covers reloads/navigations across all supported webviews;
  // `beforeunload` fires earlier on Chromium (WebView2) so the flag is set
  // before the aborted fetch's rejection handler runs. Both are passive (they
  // never call `preventDefault`/set `returnValue`), so no unload prompt shows.
  window.addEventListener('pagehide', markUnloading)
  window.addEventListener('beforeunload', markUnloading)

  // on Android we never use it because Android does not have support to reading the request body
  const canUseCustomProtocol = osName !== 'android'

  function sendIpcMessage(message) {
    const { cmd, callback, error, payload, options } = message

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
            // the document is unloading (reload/navigation), so the fetch was
            // aborted rather than blocked. Re-sending over postMessage would
            // re-invoke a command that may already be running on the backend,
            // so bail out. See https://github.com/tauri-apps/tauri/issues/14154
            if (documentIsUnloading) {
              return
            }
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
