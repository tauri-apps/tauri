// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Create web worker
const THRESHOLD = 10000000
const worker = new Worker('worker.js')
/** @type {HTMLButtonElement} */
const start = document.getElementById('start')
/** @type {HTMLParagraphElement} */
const status = document.getElementById('status')
const results = document.getElementById('results')

const ITERATIONS = 1

let resolver

const onMessage = (message) => {
  // Update the UI
  let prefix = '[Calculating]'

  if (message.data.status === 'done') {
    // tell tauri that we are done
    window.__TAURI__.core.invoke('app_completed_successfully')
  }

  status.innerHTML = `${prefix} Found <code>${message.data.count}</code> prime numbers in <code>${message.data.time}ms</code>`

  if (message.data.status === 'done') {
    resolver(message.data.time)
  }
}

worker.addEventListener('message', onMessage)

const benchmark = () => {
  return new Promise((resolve) => {
    const startTime = Date.now()
    resolver = resolve
    worker.postMessage({ value: THRESHOLD, startTime })
  })
}

const calculate = async () => {
  let total = 0

  for (let i = 0; i < ITERATIONS; i++) {
    const result = await benchmark()
    total += result
  }

  const average = total / ITERATIONS

  results.innerText = `Average time: ${average}ms`
}

window.addEventListener('DOMContentLoaded', calculate)
