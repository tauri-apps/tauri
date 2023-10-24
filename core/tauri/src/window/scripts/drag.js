// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

document.addEventListener('mousedown', (e) => {
  if (e.target.hasAttribute('data-tauri-drag-region') && e.button === 0) {
    // prevents text cursor
    e.preventDefault()
    // fix #2549: double click on drag region edge causes content to maximize without window sizing change
    // https://github.com/tauri-apps/tauri/issues/2549#issuecomment-1250036908
    e.stopImmediatePropagation()

    // start dragging if the element has a `tauri-drag-region` data attribute and maximize on double-clicking it
    const cmd = e.detail === 2 ? 'internal_toggle_maximize' : 'start_dragging'
    window.__TAURI_INTERNALS__.invoke('plugin:window|' + cmd)
  }
})
