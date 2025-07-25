<script>
  import { invoke } from '@tauri-apps/api/core'
  import {
    getName,
    getVersion,
    getTauriVersion,
    getBundleType
  } from '@tauri-apps/api/app'

  let { onMessage } = $props()

  let version = $state('1.0.0')
  let tauriVersion = $state('1.0.0')
  let appName = $state('Unknown')
  let bundleType = $state('Unknown')

  getName().then((n) => {
    appName = n
  })
  getVersion().then((v) => {
    version = v
  })
  getTauriVersion().then((v) => {
    tauriVersion = v
  })
  getBundleType().then((b) => {
    if (b) {
      bundleType = b
    }
  })

  function contextMenu() {
    invoke('plugin:app-menu|popup')
  }
</script>

<div class="grid gap-8 justify-items-start">
  <p>
    This is a demo of Tauri's API capabilities using the <code
      >@tauri-apps/api</code
    > package. It's used as the main validation app, serving as the test bed of our
    development process. In the future, this app will be used on Tauri's integration
    tests.
  </p>
  <pre>
    App name: <code>{appName}</code>
    App version: <code>{version}</code>
    Tauri version: <code>{tauriVersion}</code>
    Bundle type: <code>{bundleType}</code>
  </pre>

  <button class="btn" onclick={contextMenu}>Context menu</button>
</div>
