// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const { invoke, Channel } = window.__TAURI__.core
const { listen, emit } = window.__TAURI__.event

const PAYLOAD_SIZE = 3 * 1024 * 1024

// Every scenario is timed as a whole and divided by its iterations:
// the webviews round `performance.now()` (to 1ms on WebKit), so a single call can't be timed.
async function time(iterations, run) {
  const start = performance.now()
  for (let i = 0; i < iterations; i++) {
    await run(i)
  }
  return (performance.now() - start) / iterations
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message)
  }
}

async function main() {
  const metrics = {}

  // let the page settle and warm up the IPC
  for (let i = 0; i < 100; i++) {
    await invoke('noop')
  }

  // one call at a time: the latency of a round trip
  metrics.invoke_ms = await time(2000, () => invoke('noop'))

  // many calls in flight: the throughput
  metrics.invoke_concurrent_ms =
    (await time(5, () =>
      Promise.all(Array.from({ length: 1000 }, () => invoke('noop')))
    )) / 1000

  // a command of a plugin, which also goes through the access control list
  metrics.invoke_plugin_ms = await time(1000, (i) => emit('bench-js', i))

  let records = await invoke('json_out')
  metrics.invoke_json_response_3mb_ms = await time(10, async () => {
    records = await invoke('json_out')
  })
  assert(records.length === 16000, 'unexpected JSON response')

  metrics.invoke_json_request_3mb_ms = await time(10, async () => {
    const count = await invoke('json_in', { records })
    assert(count === records.length, 'unexpected JSON request result')
  })

  let bytes = await invoke('raw_out')
  metrics.invoke_raw_response_3mb_ms = await time(10, async () => {
    bytes = await invoke('raw_out')
  })
  assert(bytes.byteLength === PAYLOAD_SIZE, 'unexpected raw response')

  const body = new Uint8Array(bytes)
  metrics.invoke_raw_request_3mb_ms = await time(10, async () => {
    const length = await invoke('raw_in', body)
    assert(length === PAYLOAD_SIZE, 'unexpected raw request result')
  })

  metrics.channel_1000_messages_ms = await time(
    5,
    () =>
      new Promise((resolve, reject) => {
        const count = 1000
        let received = 0
        const channel = new Channel()
        channel.onmessage = () => {
          received += 1
          if (received === count) {
            resolve()
          }
        }
        invoke('channel_messages', { channel, count }).catch(reject)
      })
  )

  metrics.channel_3mb_chunks_ms = await time(
    5,
    () =>
      new Promise((resolve, reject) => {
        let received = 0
        const channel = new Channel()
        channel.onmessage = (chunk) => {
          received += chunk.byteLength
          if (received === PAYLOAD_SIZE) {
            resolve()
          }
        }
        invoke('channel_chunks', { channel, chunkSize: 64 * 1024 }).catch(
          reject
        )
      })
  )

  let onEvent = () => {}
  const unlisten = await listen('bench', (event) => onEvent(event))
  metrics.event_1000_messages_ms = await time(
    5,
    () =>
      new Promise((resolve, reject) => {
        const count = 1000
        let received = 0
        onEvent = () => {
          received += 1
          if (received === count) {
            resolve()
          }
        }
        invoke('emit_events', { count }).catch(reject)
      })
  )
  unlisten()

  await invoke('report', { metrics })
}

main().catch((error) => invoke('fail', { message: String(error) }))
