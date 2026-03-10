// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  //-----------------------//
  // drag on mousedown and maximize on double click on Windows and Linux
  // while macOS maximization should be on mouseup and if the mouse
  // moves after the double click, it should be cancelled (see https://github.com/tauri-apps/tauri/issues/8306)
  //-----------------------//
  const TAURI_DRAG_REGION_ATTR = 'data-tauri-drag-region'

  function isClickableElement(el) {
    const tag = el.tagName && el.tagName.toLowerCase()

    return (
      tag === 'a'
      || tag === 'button'
      || tag === 'input'
      || tag === 'select'
      || tag === 'textarea'
      || tag === 'label'
      || tag === 'summary'
      || (el.hasAttribute('contenteditable')
        && el.getAttribute('contenteditable') !== 'false')
      || (el.hasAttribute('tabindex') && el.getAttribute('tabindex') !== '-1')
    )
  }

  // Walk the composed path from target upward. If a clickable element or a
  // data-tauri-drag-region="false" element is encountered, return false (don't drag).
  // Otherwise return true.
  //
  // Supported values for data-tauri-drag-region:
  //   (bare / no value) -> self: only direct clicks on this element trigger drag
  //   "deep"            -> deep: clicks anywhere in the subtree trigger drag
  //   "false"           -> disabled: drag is blocked here (and for ancestors)
  function isDragRegion(composedPath) {
    for (const el of composedPath) {
      if (!(el instanceof HTMLElement)) continue

      // if we hit a clickable element or a disabled drag region, don't drag
      if (
        isClickableElement(el)
        || el.getAttribute(TAURI_DRAG_REGION_ATTR) === 'false'
      ) {
        return false
      }

      const attr = el.getAttribute(TAURI_DRAG_REGION_ATTR)
      if (attr !== null) {
        // deep: the whole subtree is a drag region
        if (attr === 'deep') return true
        // bare (or any unrecognized value): self-only
        if (el === composedPath[0]) return true
        // click was on a child of a self-only region - stop walking, don't drag
        return false
      }
    }

    return false
  }

  const osName = __TEMPLATE_os_name__

  // initial mousedown position for macOS
  let initialX = 0
  let initialY = 0

  document.addEventListener('mousedown', (e) => {
    if (
      // was left mouse button
      e.button === 0
      // and was normal click to drag or double click to maximize
      && (e.detail === 1 || e.detail === 2)
      // and is drag region
      && isDragRegion(e.composedPath())
    ) {
      // macOS maximization happens on `mouseup`,
      // so we save needed state and early return
      if (osName === 'macos' && e.detail === 2) {
        initialX = e.clientX
        initialY = e.clientY
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

  // on macOS we maximize on mouseup instead, to match the system behavior where maximization can be canceled
  // if the mouse moves outside the data-tauri-drag-region
  if (osName === 'macos') {
    document.addEventListener('mouseup', (e) => {
      if (
        // was left mouse button
        e.button === 0
        // and was double click
        && e.detail === 2
        // and the cursor hasn't moved from initial mousedown
        && e.clientX === initialX
        && e.clientY === initialY
        // and the event path contains a drag region (with no clickable element in between)
        && isDragRegion(e.composedPath())
      ) {
        window.__TAURI_INTERNALS__.invoke(
          'plugin:window|internal_toggle_maximize'
        )
      }
    })
  }
})()
