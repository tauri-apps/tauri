// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

if (location.href !== __TEMPLATE_isolation_src__) {
  window.addEventListener('DOMContentLoaded', () => {
    let style = document.createElement('style')
    style.textContent = __TEMPLATE_style__
    document.head.append(style)

    let iframe = document.createElement('iframe')
    iframe.id = '__tauri_isolation__'
    iframe.sandbox.add('allow-scripts')
    iframe.src = __TEMPLATE_isolation_src__
    document.body.append(iframe)
  })
}
