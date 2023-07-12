// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

(function () {
  function toggleDevtoolsHotkey() {
    const isHotkey = navigator.appVersion.includes("Mac")
      ? (event) => event.metaKey && event.altKey && event.key === "I"
      : (event) => event.ctrlKey && event.shiftKey && event.key === "I";

    document.addEventListener("keydown", (event) => {
      if (isHotkey(event)) {
        window.__TAURI_INVOKE__('tauri', {
          __tauriModule: 'Window',
          message: {
            cmd: 'manage',
            data: {
              cmd: {
                type: '__toggleDevtools'
              }
            }
          }
        });
      }
    });
  }

  if (
    document.readyState === "complete" ||
    document.readyState === "interactive"
  ) {
    toggleDevtoolsHotkey();
  } else {
    window.addEventListener("DOMContentLoaded", toggleDevtoolsHotkey, true);
  }
})();
