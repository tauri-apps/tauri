// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  const PERSISTENT_CHANNEL_PLUGIN_NAME = '__TAURI_PERSISTENT_CHANNEL__'
  const PERSISTENT_CHANNEL_ID_PREFIX = '__PERSISTENT_CHANNEL__:'

  const channels = Object.create(null)
  const pendingConnects = Object.create(null)

  const invoke = window.__TAURI_INTERNALS__.invoke
  const transformCallback = window.__TAURI_INTERNALS__.transformCallback

  class ChannelClosedError extends Error {
    constructor(message) {
      super(message || 'Channel is closed')
      this.name = 'ChannelClosedError'
    }
  }

  class ChannelTimeoutError extends Error {
    constructor(message) {
      super(message || 'Channel operation timed out')
      this.name = 'ChannelTimeoutError'
    }
  }

  function generateChannelId() {
    return PERSISTENT_CHANNEL_ID_PREFIX + Date.now() + '-' + Math.random().toString(36).substring(2, 11)
  }

  class PersistentChannel {
    constructor(id, onMessage, onError, onClose) {
      this._id = id
      this._onMessage = onMessage || (() => {})
      this._onError = onError || (() => {})
      this._onClose = onClose || (() => {})
      this._isClosed = false
      this._messageIndex = 0
      this._pendingAcks = Object.create(null)
      this._messageQueue = []
      this._isConnected = false

      this._callbackId = transformCallback((response) => {
        this._handleResponse(response)
      })
    }

    get id() {
      return this._id
    }

    get isClosed() {
      return this._isClosed
    }

    get isConnected() {
      return this._isConnected
    }

    _handleResponse(response) {
      if (this._isClosed) return

      if (response.end) {
        this._closeInternal('Remote closed the channel')
        return
      }

      if (response.message) {
        this._messageIndex = response.index || this._messageIndex
        try {
          const msg = response.message
          if (msg && typeof msg === 'object' && msg.type) {
            switch (msg.type) {
              case 'pong':
                this._handlePong()
                break
              case 'ack':
                this._handleAck(msg.index)
                break
              default:
                this._onMessage(msg)
            }
          } else {
            this._onMessage(response.message)
          }
        } catch (e) {
          this._onError(e)
        }
      }
    }

    async _sendInternal(message) {
      if (this._isClosed) {
        throw new ChannelClosedError()
      }

      return invoke('plugin:' + PERSISTENT_CHANNEL_PLUGIN_NAME + '|send_message', {
        channelId: this._id,
        message: message
      })
    }

    async send(data, options = {}) {
      if (this._isClosed) {
        throw new ChannelClosedError()
      }

      const message = {
        type: 'data',
        payload: data,
        index: this._messageIndex++
      }

      if (options.timeout) {
        return this._sendWithTimeout(message, options.timeout)
      }

      if (options.requireAck) {
        return this._sendWithAck(message, options.timeout || 30000)
      }

      return this._sendInternal(message)
    }

    async _sendWithTimeout(message, timeout) {
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => {
          reject(new ChannelTimeoutError('Send operation timed out'))
        }, timeout)
      })

      return Promise.race([this._sendInternal(message), timeoutPromise])
    }

    async _sendWithAck(message, timeout) {
      const index = message.index
      const ackPromise = new Promise((resolve, reject) => {
        const timeoutId = setTimeout(() => {
          delete this._pendingAcks[index]
          reject(new ChannelTimeoutError('Ack not received'))
        }, timeout)

        this._pendingAcks[index] = {
          resolve: (response) => {
            clearTimeout(timeoutId)
            delete this._pendingAcks[index]
            resolve(response)
          },
          reject: (error) => {
            clearTimeout(timeoutId)
            delete this._pendingAcks[index]
            reject(error)
          }
        }
      })

      await this._sendInternal(message)
      return ackPromise
    }

    _handleAck(index) {
      const ack = this._pendingAcks[index]
      if (ack) {
        ack.resolve()
      }
    }

    async sendBinary(data) {
      if (this._isClosed) {
        throw new ChannelClosedError()
      }

      let bytes
      if (data instanceof ArrayBuffer) {
        bytes = new Uint8Array(data)
      } else if (data instanceof Uint8Array) {
        bytes = data
      } else if (ArrayBuffer.isView(data)) {
        bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength)
      } else {
        throw new Error('Unsupported binary data type. Use ArrayBuffer or TypedArray.')
      }

      return invoke('plugin:' + PERSISTENT_CHANNEL_PLUGIN_NAME + '|send_binary', {
        channelId: this._id,
        data: Array.from(bytes)
      })
    }

    async ping(timeout = 5000) {
      if (this._isClosed) {
        throw new ChannelClosedError()
      }

      const pongPromise = new Promise((resolve, reject) => {
        const timeoutId = setTimeout(() => {
          this._pongHandler = null
          reject(new ChannelTimeoutError('Ping timed out'))
        }, timeout)

        this._pongHandler = () => {
          clearTimeout(timeoutId)
          this._pongHandler = null
          resolve()
        }
      })

      await this._sendInternal({ type: 'ping' })
      return pongPromise
    }

    _handlePong() {
      if (this._pongHandler) {
        this._pongHandler()
      }
    }

    async close() {
      if (this._isClosed) {
        return
      }

      try {
        await this._sendInternal({ type: 'close' })
      } catch (e) {
        // Ignore errors during close
      }

      this._closeInternal('Closed by client')
    }

    _closeInternal(reason) {
      if (this._isClosed) {
        return
      }

      this._isClosed = true
      this._isConnected = false

      // Reject all pending acks
      for (const index in this._pendingAcks) {
        if (Object.prototype.hasOwnProperty.call(this._pendingAcks, index)) {
          this._pendingAcks[index].reject(new ChannelClosedError('Channel closed before ack received'))
        }
      }
      this._pendingAcks = Object.create(null)

      // Unregister callback
      window.__TAURI_INTERNALS__.unregisterCallback(this._callbackId)

      // Remove from channels map
      delete channels[this._id]

      this._onClose(reason)
    }

    onMessage(handler) {
      this._onMessage = handler
      return this
    }

    onError(handler) {
      this._onError = handler
      return this
    }

    onClose(handler) {
      this._onClose = handler
      return this
    }
  }

  class ChannelBuilder {
    constructor() {
      this._onMessage = null
      this._onError = null
      this._onClose = null
      this._timeout = 30000
    }

    onMessage(handler) {
      this._onMessage = handler
      return this
    }

    onError(handler) {
      this._onError = handler
      return this
    }

    onClose(handler) {
      this._onClose = handler
      return this
    }

    timeout(ms) {
      this._timeout = ms
      return this
    }

    async connect() {
      const channelId = generateChannelId()
      const channel = new PersistentChannel(
        channelId,
        this._onMessage,
        this._onError,
        this._onClose
      )

      channels[channelId] = channel

      try {
        const response = await invoke('plugin:' + PERSISTENT_CHANNEL_PLUGIN_NAME + '|connect', {
          channelId: channelId,
          callbackId: channel._callbackId
        })

        channel._isConnected = true
        return channel
      } catch (e) {
        delete channels[channelId]
        window.__TAURI_INTERNALS__.unregisterCallback(channel._callbackId)
        throw e
      }
    }
  }

  function getChannel(channelId) {
    return channels[channelId] || null
  }

  function getAllChannels() {
    return Object.values(channels)
  }

  async function broadcast(data) {
    return invoke('plugin:' + PERSISTENT_CHANNEL_PLUGIN_NAME + '|broadcast', {
      message: data
    })
  }

  function createChannel() {
    return new ChannelBuilder()
  }

  Object.defineProperty(window.__TAURI_INTERNALS__, 'persistentChannel', {
    value: Object.freeze({
      create: createChannel,
      get: getChannel,
      getAll: getAllChannels,
      broadcast: broadcast,
      Channel: PersistentChannel,
      ChannelClosedError,
      ChannelTimeoutError
    })
  })

  window.__TAURI_PERSISTENT_CHANNEL__ = {
    create: createChannel,
    get: getChannel,
    getAll: getAllChannels,
    broadcast: broadcast
  }
})()
