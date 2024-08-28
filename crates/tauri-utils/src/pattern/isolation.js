// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * IMPORTANT: See ipc.js for the main frame implementation.
 * main frame -> isolation frame = isolation payload
 * isolation frame -> main frame = isolation message
 */

;(async function () {
  /**
   * Sends the message to the isolation frame.
   * @param {any} message
   */
  function sendMessage(message) {
    window.parent.postMessage(message, '*')
  }

  /**
   * @type {string} - The main frame origin.
   */
  const origin = __TEMPLATE_origin__

  /**
   * @type {Uint8Array} - Injected by Tauri during runtime
   */
  const aesGcmKeyRaw = new Uint8Array(__TEMPLATE_runtime_aes_gcm_key__)

  /**
   * @type {CryptoKey}
   */
  const aesGcmKey = await window.crypto.subtle.importKey(
    'raw',
    aesGcmKeyRaw,
    'AES-GCM',
    false,
    ['encrypt']
  )

  /**
   * @param {object} data
   * @return {Promise<{nonce: number[], payload: number[]}>}
   */
  async function encrypt(payload) {
    const algorithm = Object.create(null)
    algorithm.name = 'AES-GCM'
    algorithm.iv = window.crypto.getRandomValues(new Uint8Array(12))

    const { contentType, data } = __RAW_process_ipc_message_fn__(payload)

    const message =
      typeof data === 'string'
        ? new TextEncoder().encode(data)
        : ArrayBuffer.isView(data) || data instanceof ArrayBuffer
          ? data
          : new Uint8Array(data)

    return window.crypto.subtle
      .encrypt(algorithm, aesGcmKey, message)
      .then((payload) => {
        const result = Object.create(null)
        result.nonce = Array.from(new Uint8Array(algorithm.iv))
        result.payload = Array.from(new Uint8Array(payload))
        result.contentType = contentType
        return result
      })
  }

  /**
   * Detects if a message event is a valid isolation message.
   *
   * @param {MessageEvent<object>} event - a message event that is expected to be an isolation message
   * @return {boolean} - if the event was a valid isolation message
   */
  function isIsolationMessage(data) {
    if (typeof data === 'object' && typeof data.payload === 'object') {
      const keys = data.payload ? Object.keys(data.payload) : []
      return (
        keys.length > 0 &&
        keys.every(
          (key) => key === 'nonce' || key === 'payload' || key === 'contentType'
        )
      )
    }
    return false
  }

  /**
   * Detect if a message event is a valid isolation payload.
   *
   * @param {MessageEvent<object>} event - a message event that is expected to be an isolation payload
   * @return boolean
   */
  function isIsolationPayload(data) {
    return (
      typeof data === 'object' &&
      'callback' in data &&
      'error' in data &&
      !isIsolationMessage(data)
    )
  }

  /**
   * Handle incoming payload events.
   * @param {MessageEvent<any>} event
   */
  async function payloadHandler(event) {
    if (event.origin !== origin || !isIsolationPayload(event.data)) {
      return
    }

    let data = event.data

    if (typeof window.__TAURI_ISOLATION_HOOK__ === 'function') {
      // await even if it's not async so that we can support async ones
      data = await window.__TAURI_ISOLATION_HOOK__(data)
    }

    const message = Object.create(null)
    message.cmd = data.cmd
    message.callback = data.callback
    message.error = data.error
    message.options = data.options
    message.payload = await encrypt(data.payload)
    sendMessage(message)
  }

  window.addEventListener('message', payloadHandler, false)

  /**
   * @type {number} - How many milliseconds to wait between ready checks
   */
  const readyIntervalMs = 50

  /**
   * Wait until this Isolation context is ready to receive messages, and let the main frame know.
   */
  function waitUntilReady() {
    // consider either a function or an explicitly set null value as the ready signal
    if (
      typeof window.__TAURI_ISOLATION_HOOK__ === 'function' ||
      window.__TAURI_ISOLATION_HOOK__ === null
    ) {
      sendMessage('__TAURI_ISOLATION_READY__')
    } else {
      setTimeout(waitUntilReady, readyIntervalMs)
    }
  }

  setTimeout(waitUntilReady, readyIntervalMs)
})()

document.currentScript?.remove()
