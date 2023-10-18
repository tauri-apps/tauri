// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

(function () {
  const osName = __TEMPLATE_os_name__

  function toggleDevtoolsHotkey() {
    const isHotkey = osName === 'macos' ?
      (event) => event.metaKey && event.altKey && event.code === "KeyI" :
      (event) => event.ctrlKey && event.shiftKey && event.code === "KeyI";

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
