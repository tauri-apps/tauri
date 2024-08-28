// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  const osName = __TEMPLATE_os_name__

  //-----------------------//
  // drag on mousedown and maximize on double click on Windows and Linux
  // while macOS macos maximization should be on mouseup and if the mouse
  // moves after the double click, it should be cancelled (see https://github.com/tauri-apps/tauri/issues/8306)
  //-----------------------//
  const TAURI_DRAG_REGION_ATTR = 'data-tauri-drag-region'
  let x = 0,
    y = 0
  document.addEventListener('mousedown', (e) => {
    if (
      // element has the magic data attribute
      e.target.hasAttribute(TAURI_DRAG_REGION_ATTR) &&
      // and was left mouse button
      e.button === 0 &&
      // and was normal click to drag or double click to maximize
      (e.detail === 1 || e.detail === 2)
    ) {
      // macOS maximization happens on `mouseup`,
      // so we save needed state and early return
      if (osName === 'macos' && e.detail == 2) {
        x = e.clientX
        y = e.clientY
        return
      }

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
  // on macOS we maximze on mouseup instead, to match the system behavior where maximization can be canceled
  // if the mouse moves outside the data-tauri-drag-region
  if (osName === 'macos') {
    document.addEventListener('mouseup', (e) => {
      if (
        // element has the magic data attribute
        e.target.hasAttribute(TAURI_DRAG_REGION_ATTR) &&
        // and was left mouse button
        e.button === 0 &&
        // and was double click
        e.detail === 2 &&
        // and the cursor hasn't moved from initial mousedown
        e.clientX === x &&
        e.clientY === y
      ) {
        window.__TAURI_INTERNALS__.invoke(
          'plugin:window|internal_toggle_maximize'
        )
      }
    })
  }
})()
