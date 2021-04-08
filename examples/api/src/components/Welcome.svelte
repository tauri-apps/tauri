<script>
  import { getName, getVersion, getTauriVersion, relaunch, exit } from "@tauri-apps/api/app";
  let version = 0.0;
  let tauriVersion = 0.0;
  let appName = 'Unknown';

  getName().then(n => {appName = n});
  getVersion().then(v => {version = v});
  getTauriVersion().then(v => {tauriVersion = v});

  async function closeApp() {
    await exit();
  }

  async function relaunchApp() {
    await relaunch();
  }
</script>
<h1>Welcome</h1>
<p>
  Tauri's API capabilities using the ` @tauri-apps/api ` package. It's used as
  the main validation app, serving as the testbed of our development process. In
  the future, this app will be used on Tauri's integration tests.
</p>

<p>Current App version: {version}</p>
<p>Current Tauri version: {tauriVersion}</p>
<p>Current App name: {appName}</p>

<button class="button" on:click={closeApp}>Close application</button>
<button class="button" on:click={relaunchApp}>Relaunch application</button>
