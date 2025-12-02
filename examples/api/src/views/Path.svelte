<script>
  import {
    appConfigDir,
    appDataDir,
    appLocalDataDir,
    appCacheDir,
    appLogDir,
    appLibraryDir,
    audioDir,
    cacheDir,
    configDir,
    dataDir,
    desktopDir,
    documentDir,
    downloadDir,
    executableDir,
    fontDir,
    homeDir,
    localDataDir,
    pictureDir,
    publicDir,
    resourceDir,
    runtimeDir,
    templateDir,
    videoDir,
    tempDir
  } from '@tauri-apps/api/path'

  let { onMessage } = $props()

  const items = [
    { label: 'Audio', fn: audioDir },
    { label: 'Cache', fn: cacheDir },
    { label: 'Config', fn: configDir },
    { label: 'Data', fn: dataDir },
    { label: 'LocalData', fn: localDataDir },
    { label: 'Document', fn: documentDir },
    { label: 'Download', fn: downloadDir },
    { label: 'Picture', fn: pictureDir },
    { label: 'Public', fn: publicDir },
    { label: 'Video', fn: videoDir },
    { label: 'Resource', fn: resourceDir },
    { label: 'Temp', fn: tempDir },
    { label: 'AppConfig', fn: appConfigDir },
    { label: 'AppData', fn: appDataDir },
    { label: 'AppLocalData', fn: appLocalDataDir },
    { label: 'AppCache', fn: appCacheDir },
    { label: 'AppLog', fn: appLogDir },
    { label: 'Desktop', fn: desktopDir },
    { label: 'Executable', fn: executableDir },
    { label: 'Font', fn: fontDir },
    { label: 'Home', fn: homeDir },
    { label: 'Runtime', fn: runtimeDir },
    { label: 'Template', fn: templateDir },
    { label: 'AppLibrary', fn: appLibraryDir },
  ]

  async function run(item) {
    try {
      const path = await item.fn()
      onMessage(`${item.label}: ${path}`)
    } catch (e) {
      onMessage(`${item.label} error: ${String(e)}`)
    }
  }

</script>

<div>
  <div class="grid gap-2 grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
    {#each items as item}
      <button class="btn" onclick={() => run(item)}>{item.label}</button>
    {/each}
  </div>
</div>
