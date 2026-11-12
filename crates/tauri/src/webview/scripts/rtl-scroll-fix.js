// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Mitigate scroll jump during RTL text input in WKWebView (macOS).
//
// On certain macOS/WebKit versions, editing wrapped RTL text in
// contenteditable elements causes ancestor scroll containers to jump
// to incorrect positions on every keystroke. This happens because
// WebKit's native scroll-into-view for the caret uses incorrect
// geometry for wrapped RTL lines, and the resulting scroll bypasses
// JavaScript event handlers.
//
// Fix: on `beforeinput`, temporarily make scrollable ancestor
// containers non-scrollable (`overflow: hidden`) so the native
// scroll-into-view has no effect, then restore overflow and scroll
// positions in `requestAnimationFrame` (before the next paint).

;(function () {
  var frozen = null
  var raf = 0

  function getScrollParents(el) {
    var parents = []
    var node = el.parentElement
    while (node) {
      var s = getComputedStyle(node)
      if (
        s.overflowY === 'auto' ||
        s.overflowY === 'scroll' ||
        s.overflowX === 'auto' ||
        s.overflowX === 'scroll'
      ) {
        parents.push(node)
      }
      node = node.parentElement
    }
    var dse = document.scrollingElement
    if (dse && parents.indexOf(dse) === -1) {
      parents.push(dse)
    }
    return parents
  }

  function hasRTLDir(el) {
    if (getComputedStyle(el).direction === 'rtl') return true
    var ancestor = el.closest('[dir]')
    return (
      ancestor != null &&
      (ancestor.getAttribute('dir') === 'rtl' ||
        ancestor.getAttribute('dir') === 'auto')
    )
  }

  document.addEventListener(
    'beforeinput',
    function () {
      var active = document.activeElement
      if (!active) return
      if (!active.isContentEditable && active.tagName !== 'TEXTAREA') return
      if (!hasRTLDir(active)) return

      var parents = getScrollParents(active)
      if (!parents.length) return

      // Save state and freeze containers
      frozen = parents.map(function (p) {
        return {
          el: p,
          top: p.scrollTop,
          left: p.scrollLeft,
          ov: p.style.getPropertyValue('overflow'),
          ovp: p.style.getPropertyPriority('overflow')
        }
      })

      for (var i = 0; i < frozen.length; i++) {
        frozen[i].el.style.setProperty('overflow', 'hidden', 'important')
      }

      cancelAnimationFrame(raf)
      raf = requestAnimationFrame(function () {
        if (!frozen) return
        for (var i = 0; i < frozen.length; i++) {
          var f = frozen[i]
          // Restore original overflow
          if (f.ov) {
            f.el.style.setProperty('overflow', f.ov, f.ovp)
          } else {
            f.el.style.removeProperty('overflow')
          }
          // Restore scroll position
          f.el.scrollTop = f.top
          f.el.scrollLeft = f.left
        }
        frozen = null
      })
    },
    true
  )
})()
