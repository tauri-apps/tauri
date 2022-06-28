<script>
  import { getName, getVersion, getTauriVersion } from '@tauri-apps/api/app'
  import { relaunch, exit } from '@tauri-apps/api/process'

  let version = '0.0.0'
  let tauriVersion = '0.0.0'
  let appName = 'Unknown'

  getName().then((n) => {
    appName = n
  })
  getVersion().then((v) => {
    version = v
  })
  getTauriVersion().then((v) => {
    tauriVersion = v
  })

  async function closeApp() {
    await exit()
  }

  async function relaunchApp() {
    await relaunch()
  }
</script>

<p>
  This is a demo of Tauri's API capabilities using the <code
    >@tauri-apps/api</code
  > package. It's used as the main validation app, serving as the test bed of our
  development process. In the future, this app will be used on Tauri's integration
  tests.
</p>

<br />
<br />
<pre>
App name: <code>{appName}</code>
App version: <code>{version}</code>
Tauri version: <code>{tauriVersion}</code>
</pre>
<br />
<div class="flex flex-wrap gap-1 shadow-">
  <button class="btn" on:click={closeApp}>Close application</button>
  <button class="btn" on:click={relaunchApp}>Relaunch application</button>
</div>
