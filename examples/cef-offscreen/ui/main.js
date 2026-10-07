// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

document.querySelector('#invoke').onclick = async () => {
  const result = document.querySelector('#result')
  try {
    result.textContent = await window.__TAURI__.core.invoke('greet')
  } catch (error) {
    result.textContent = String(error)
  }
}

let clicks = 0
window.addEventListener('click', () => {
  document.querySelector('#clicks').textContent = `Browser clicks: ${++clicks}`
})

const drag = document.querySelector('#drag')
drag.onpointerdown = (event) => drag.setPointerCapture(event.pointerId)
drag.onpointermove = (event) => {
  if (drag.hasPointerCapture(event.pointerId)) {
    drag.textContent = `Dragging: ${Math.round(event.clientX)}, ${Math.round(event.clientY)}`
  }
}
drag.onpointerup = () => {
  drag.textContent = 'Drag finished'
}
drag.onpointercancel = () => {
  drag.textContent = 'Drag cancelled'
}
