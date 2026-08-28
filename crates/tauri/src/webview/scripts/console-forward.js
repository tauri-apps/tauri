// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  if (window.__TAURI_CONSOLE_FORWARDER__) {
    return
  }
  window.__TAURI_CONSOLE_FORWARDER__ = true

  const MAX_ARG_LENGTH = 8 * 1024
  const MAX_MESSAGE_LENGTH = 32 * 1024

  const original = {
    error: console.error.bind(console),
    warn: console.warn.bind(console),
    info: console.info.bind(console),
    log: console.log.bind(console),
    debug: console.debug.bind(console)
  }

  let reported = false
  function reportOnce(error) {
    if (!reported) {
      reported = true
      original.warn('failed to forward console message to the host:', error)
    }
  }

  function truncate(str, max) {
    return str.length > max ? str.slice(0, max) + '…' : str
  }

  function serializeArg(arg) {
    if (typeof arg === 'string') {
      return arg
    }
    if (arg instanceof Error) {
      // WebKit stacks do not repeat name and message, so always prefix them
      return arg.stack
        ? `${arg.name}: ${arg.message}\n${arg.stack}`
        : `${arg.name}: ${arg.message}`
    }
    try {
      const seen = new WeakSet()
      const json = JSON.stringify(arg, (_, value) => {
        if (typeof value === 'object' && value !== null) {
          if (seen.has(value)) {
            return '[circular]'
          }
          seen.add(value)
        }
        if (typeof value === 'bigint') {
          return String(value)
        }
        return value
      })
      return json === undefined ? String(arg) : json
    } catch (_) {
      return String(arg)
    }
  }

  let forwarding = false
  function send(level, message, extra) {
    if (forwarding) {
      return
    }
    forwarding = true
    try {
      const payload = Object.assign(
        { level, message: truncate(message, MAX_MESSAGE_LENGTH) },
        extra
      )
      // the attached catch keeps a failing invoke from re-entering through
      // the unhandledrejection listener
      window.__TAURI_INTERNALS__
        .invoke('plugin:webview|internal_log', { payload })
        .catch(reportOnce)
    } catch (error) {
      reportOnce(error)
    } finally {
      forwarding = false
    }
  }

  for (const level of ['error', 'warn', 'info', 'log', 'debug']) {
    console[level] = function (...args) {
      original[level](...args)
      send(
        level,
        args
          .map((arg) => truncate(serializeArg(arg), MAX_ARG_LENGTH))
          .join(' '),
        { kind: 'console' }
      )
    }
  }

  // capture phase: failed <script>/<link>/<img> loads fire on the element and
  // never bubble to a window listener
  window.addEventListener(
    'error',
    (event) => {
      const target = event.target
      if (target && target !== window && (target.src || target.href)) {
        const url = String(target.src || target.href)
        send(
          'error',
          `failed to load <${target.tagName.toLowerCase()}> resource: ${url}`,
          {
            kind: 'resourceError'
          }
        )
      } else {
        send('error', event.message || 'uncaught error', {
          kind: 'error',
          url: event.filename || undefined,
          line: event.lineno || undefined,
          col: event.colno || undefined,
          stack:
            event.error && event.error.stack
              ? String(event.error.stack)
              : undefined
        })
      }
    },
    true
  )

  window.addEventListener('unhandledrejection', (event) => {
    const reason = event.reason
    const message =
      reason instanceof Error
        ? `${reason.name}: ${reason.message}`
        : serializeArg(reason)
    send(
      'error',
      `unhandled promise rejection: ${truncate(message, MAX_ARG_LENGTH)}`,
      {
        kind: 'unhandledRejection',
        stack: reason && reason.stack ? String(reason.stack) : undefined
      }
    )
  })
})()
