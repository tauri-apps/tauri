<script>
  import { onDestroy } from 'svelte'
  import {
    LogicalSize,
    UserAttentionType,
    PhysicalSize,
    PhysicalPosition,
    Effect,
    EffectState,
    ProgressBarStatus
  } from '@tauri-apps/api/window'
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

  export let onMessage

  const webview = WebviewWindow.getCurrent()

  let selectedWebview = webview.label
  const webviewMap = {
    [webview.label]: webview
  }

  const cursorIconOptions = [
    'default',
    'crosshair',
    'hand',
    'arrow',
    'move',
    'text',
    'wait',
    'help',
    'progress',
    // something cannot be done
    'notAllowed',
    'contextMenu',
    'cell',
    'verticalText',
    'alias',
    'copy',
    'noDrop',
    // something can be grabbed
    'grab',
    /// something is grabbed
    'grabbing',
    'allScroll',
    'zoomIn',
    'zoomOut',
    // edge is to be moved
    'eResize',
    'nResize',
    'neResize',
    'nwResize',
    'sResize',
    'seResize',
    'swResize',
    'wResize',
    'ewResize',
    'nsResize',
    'neswResize',
    'nwseResize',
    'colResize',
    'rowResize'
  ]

  const windowsEffects = [
    'mica',
    'blur',
    'acrylic',
    'tabbed',
    'tabbedDark',
    'tabbedLight'
  ]
  const isWindows = navigator.appVersion.includes('Windows')
  const isMacOS = navigator.appVersion.includes('Macintosh')
  let effectOptions = isWindows
    ? windowsEffects
    : Object.keys(Effect)
        .map((effect) => Effect[effect])
        .filter((e) => !windowsEffects.includes(e))
  const effectStateOptions = Object.keys(EffectState).map(
    (state) => EffectState[state]
  )

  const progressBarStatusOptions = Object.keys(ProgressBarStatus).map(
    (s) => ProgressBarStatus[s]
  )

  const mainEl = document.querySelector('main')

  let newWebviewLabel

  let resizable = true
  let maximizable = true
  let minimizable = true
  let closable = true
  let maximized = false
  let decorations = true
  let alwaysOnTop = false
  let alwaysOnBottom = false
  let contentProtected = false
  let fullscreen = false
  let width = null
  let height = null
  let minWidth = null
  let minHeight = null
  let maxWidth = null
  let maxHeight = null
  let x = null
  let y = null
  let scaleFactor = 1
  let innerPosition = new PhysicalPosition(x, y)
  let outerPosition = new PhysicalPosition(x, y)
  let innerSize = new PhysicalSize(width, height)
  let outerSize = new PhysicalSize(width, height)
  let resizeEventUnlisten
  let moveEventUnlisten
  let cursorGrab = false
  let cursorVisible = true
  let cursorX = null
  let cursorY = null
  /** @type {import('@tauri-apps/api/window').CursorIcon} */
  let cursorIcon = 'default'
  let cursorIgnoreEvents = false
  let windowTitle = 'Awesome Tauri Example!'

  /** @type {import('@tauri-apps/api/window').Theme | 'auto'} */
  let theme = 'auto'

  let effects = []
  let selectedEffect
  let effectState
  let effectRadius
  let effectR, effectG, effectB, effectA

  /** @type {ProgressBarStatus} */
  let selectedProgressBarStatus = ProgressBarStatus.None
  let progress = 0

  let windowIconPath

  function setTitle() {
    webviewMap[selectedWebview].setTitle(windowTitle)
  }

  async function hide() {
    let visible = await webviewMap[selectedWebview].isVisible()
    onMessage('window is ' + (visible ? 'visible' : 'invisible'))
    await webviewMap[selectedWebview].hide()

    setTimeout(async () => {
      visible = await webviewMap[selectedWebview].isVisible()
      onMessage('window is ' + (visible ? 'visible' : 'invisible'))

      await webviewMap[selectedWebview].show()
      visible = await webviewMap[selectedWebview].isVisible()
      onMessage('window is ' + (visible ? 'visible' : 'invisible'))
    }, 2000)
  }

  async function disable() {
    let enabled = await webviewMap[selectedWebview].isEnabled()
    onMessage('window is ' + (enabled ? 'enabled' : 'disabled'))

    await webviewMap[selectedWebview].setEnabled(false)

    setTimeout(async () => {
      enabled = await webviewMap[selectedWebview].isEnabled()
      onMessage('window is ' + (enabled ? 'enabled' : 'disabled'))

      await webviewMap[selectedWebview].setEnabled(true)
      enabled = await webviewMap[selectedWebview].isEnabled()
      onMessage('window is ' + (enabled ? 'enabled' : 'disabled'))
    }, 2000)
  }

  function minimize() {
    webviewMap[selectedWebview].minimize()
    setTimeout(webviewMap[selectedWebview].unminimize, 2000)
  }

  function changeIcon() {
    webviewMap[selectedWebview].setIcon(windowIconPath)
  }

  function createWebviewWindow() {
    if (!newWebviewLabel) return

    const label = `main-${newWebviewLabel}`
    const webview = new WebviewWindow(label)
    webviewMap[label] = webview
    webview.once('tauri://error', function (e) {
      onMessage('Error creating new webview ' + JSON.stringify(e))
    })
    webview.once('tauri://created', function () {
      onMessage('webview created')
    })
  }

  function loadWindowSize() {
    webviewMap[selectedWebview].innerSize().then((response) => {
      innerSize = response
      width = innerSize.width
      height = innerSize.height
    })
    webviewMap[selectedWebview].outerSize().then((response) => {
      outerSize = response
    })
  }

  function loadWindowPosition() {
    webviewMap[selectedWebview].innerPosition().then((response) => {
      innerPosition = response
    })
    webviewMap[selectedWebview].outerPosition().then((response) => {
      outerPosition = response
      x = outerPosition.x
      y = outerPosition.y
    })
  }

  async function addWindowEventListeners(window) {
    if (!window) return
    resizeEventUnlisten?.()
    moveEventUnlisten?.()
    moveEventUnlisten = await window.listen('tauri://move', loadWindowPosition)
    resizeEventUnlisten = await window.listen('tauri://resize', loadWindowSize)
  }

  async function requestUserAttention() {
    await webviewMap[selectedWebview].minimize()
    await webviewMap[selectedWebview].requestUserAttention(
      UserAttentionType.Critical
    )
    await new Promise((resolve) => setTimeout(resolve, 3000))
    await webviewMap[selectedWebview].requestUserAttention(null)
  }

  async function switchTheme() {
    switch (theme) {
      case 'dark':
        theme = 'light'
        break
      case 'light':
        theme = 'auto'
        break
      case 'auto':
        theme = 'dark'
        break
    }
    await webviewMap[selectedWebview].setTheme(theme === 'auto' ? null : theme)
  }

  async function updateProgressBar() {
    webviewMap[selectedWebview]?.setProgressBar({
      status: selectedProgressBarStatus,
      progress
    })
  }

  async function addEffect() {
    if (!effects.includes(selectedEffect)) {
      effects = [...effects, selectedEffect]
    }

    const payload = {
      effects,
      state: effectState,
      radius: effectRadius
    }
    if (
      Number.isInteger(effectR) &&
      Number.isInteger(effectG) &&
      Number.isInteger(effectB) &&
      Number.isInteger(effectA)
    ) {
      payload.color = [effectR, effectG, effectB, effectA]
    }

    mainEl.classList.remove('bg-primary')
    mainEl.classList.remove('dark:bg-darkPrimary')
    await webviewMap[selectedWebview].clearEffects()
    await webviewMap[selectedWebview].setEffects(payload)
  }

  async function clearEffects() {
    effects = []
    await webviewMap[selectedWebview].clearEffects()
    mainEl.classList.add('bg-primary')
    mainEl.classList.add('dark:bg-darkPrimary')
  }

  async function updatePosition() {
    webviewMap[selectedWebview]?.setPosition(new PhysicalPosition(x, y))
  }

  async function updateSize() {
    webviewMap[selectedWebview]?.setSize(new PhysicalSize(width, height))
  }

  $: {
    webviewMap[selectedWebview]
    loadWindowPosition()
    loadWindowSize()
  }
  $: webviewMap[selectedWebview]?.setResizable(resizable)
  $: webviewMap[selectedWebview]?.setMaximizable(maximizable)
  $: webviewMap[selectedWebview]?.setMinimizable(minimizable)
  $: webviewMap[selectedWebview]?.setClosable(closable)
  $: maximized
    ? webviewMap[selectedWebview]?.maximize()
    : webviewMap[selectedWebview]?.unmaximize()
  $: webviewMap[selectedWebview]?.setDecorations(decorations)
  $: webviewMap[selectedWebview]?.setAlwaysOnTop(alwaysOnTop)
  $: webviewMap[selectedWebview]?.setAlwaysOnBottom(alwaysOnBottom)
  $: webviewMap[selectedWebview]?.setContentProtected(contentProtected)
  $: webviewMap[selectedWebview]?.setFullscreen(fullscreen)

  $: minWidth && minHeight
    ? webviewMap[selectedWebview]?.setMinSize(
        new LogicalSize(minWidth, minHeight)
      )
    : webviewMap[selectedWebview]?.setMinSize(null)
  $: maxWidth > 800 && maxHeight > 400
    ? webviewMap[selectedWebview]?.setMaxSize(
        new LogicalSize(maxWidth, maxHeight)
      )
    : webviewMap[selectedWebview]?.setMaxSize(null)
  $: webviewMap[selectedWebview]
    ?.scaleFactor()
    .then((factor) => (scaleFactor = factor))
  $: addWindowEventListeners(webviewMap[selectedWebview])

  $: webviewMap[selectedWebview]?.setCursorGrab(cursorGrab)
  $: webviewMap[selectedWebview]?.setCursorVisible(cursorVisible)
  $: webviewMap[selectedWebview]?.setCursorIcon(cursorIcon)
  $: cursorX !== null &&
    cursorY !== null &&
    webviewMap[selectedWebview]?.setCursorPosition(
      new PhysicalPosition(cursorX, cursorY)
    )
  $: webviewMap[selectedWebview]?.setIgnoreCursorEvents(cursorIgnoreEvents)

  onDestroy(() => {
    resizeEventUnlisten?.()
    moveEventUnlisten?.()
  })
</script>

<div class="flex flex-col children:grow gap-8 mb-4">
  <div
    class="flex flex-wrap items-center gap-4 pb-6 border-b-solid border-b-1 border-code"
  >
    {#if Object.keys(webviewMap).length >= 1}
      <div class="grid gap-1">
        <h4 class="my-2">Selected Window</h4>
        <select class="input" bind:value={selectedWebview}>
          <option value="" disabled selected>Choose a window...</option>
          {#each Object.keys(webviewMap) as label}
            <option value={label}>{label}</option>
          {/each}
        </select>
      </div>
    {/if}
    <div class="grid gap-1">
      <h4 class="my-2">Create New Window</h4>
      <form class="flex gap-2" on:submit|preventDefault={createWebviewWindow}>
        <input
          class="input"
          type="text"
          placeholder="New window label.."
          bind:value={newWebviewLabel}
        />
        <button class="btn" type="submit">Create</button>
      </form>
    </div>
  </div>
  {#if webviewMap[selectedWebview]}
    <div class="flex flex-wrap items-center gap-4">
      <div class="grid gap-1 grow">
        <h4 class="my-2">Change Window Icon</h4>
        <form class="flex gap-2" on:submit|preventDefault={changeIcon}>
          <input
            class="input flex-1 min-w-10"
            placeholder="Window icon path"
            bind:value={windowIconPath}
          />
          <button class="btn" type="submit">Change</button>
        </form>
      </div>
      <div class="grid gap-1 grow">
        <h4 class="my-2">Set Window Title</h4>
        <form class="flex gap-2" on:submit|preventDefault={setTitle}>
          <input class="input flex-1 min-w-10" bind:value={windowTitle} />
          <button class="btn" type="submit">Set</button>
        </form>
      </div>
    </div>
    <div class="flex flex-wrap gap-2">
      <button
        class="btn"
        title="Unminimizes after 2 seconds"
        on:click={() => webviewMap[selectedWebview].center()}
      >
        Center
      </button>
      <button
        class="btn"
        title="Unminimizes after 2 seconds"
        on:click={minimize}
      >
        Minimize
      </button>
      <button class="btn" title="Visible again after 2 seconds" on:click={hide}>
        Hide
      </button>
      <button
        class="btn"
        title="Enabled again after 2 seconds"
        on:click={disable}
      >
        Disable
      </button>
      <button
        class="btn"
        on:click={requestUserAttention}
        title="Minimizes the window, requests attention for 3s and then resets it"
        >Request attention</button
      >
      <button class="btn" on:click={switchTheme}>Switch Theme ({theme})</button>
    </div>
    <div class="grid cols-[repeat(auto-fill,minmax(180px,1fr))]">
      <label>
        <input type="checkbox" class="checkbox" bind:checked={resizable} />
        Resizable
      </label>
      <label>
        <input type="checkbox" class="checkbox" bind:checked={maximizable} />
        Maximizable
      </label>
      <label>
        <input type="checkbox" class="checkbox" bind:checked={minimizable} />
        Minimizable
      </label>
      <label>
        <input type="checkbox" class="checkbox" bind:checked={closable} />
        Closable
      </label>
      <label>
        <input type="checkbox" class="checkbox" bind:checked={decorations} />
        Has decorations
      </label>
      <label>
        <input type="checkbox" class="checkbox" bind:checked={alwaysOnTop} />
        Always on top
      </label>
      <label>
        <input type="checkbox" class="checkbox" bind:checked={alwaysOnBottom} />
        Always on bottom
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={contentProtected}
        />
        Content protected
      </label>
      <label>
        <input type="checkbox" class="checkbox" bind:checked={maximized} />
        Maximized
      </label>
      <label>
        <input type="checkbox" class="checkbox" bind:checked={fullscreen} />
        Fullscreen
      </label>
    </div>
    <div class="flex flex-wrap children:flex-basis-30 gap-2">
      <div class="grid gap-1 children:grid">
        <label>
          X
          <input
            class="input"
            type="number"
            bind:value={x}
            on:change={updatePosition}
            min="0"
          />
        </label>
        <label>
          Y
          <input
            class="input"
            type="number"
            bind:value={y}
            on:change={updatePosition}
            min="0"
          />
        </label>
      </div>
      <div class="grid gap-1 children:grid">
        <label>
          Width
          <input
            class="input"
            type="number"
            bind:value={width}
            on:change={updateSize}
            min="400"
          />
        </label>
        <div>
          Height
          <input
            class="input"
            type="number"
            bind:value={height}
            on:change={updateSize}
            min="400"
          />
        </div>
      </div>
      <div class="grid gap-1 children:grid">
        <label>
          Min width
          <input class="input" type="number" bind:value={minWidth} />
        </label>
        <label>
          Min height
          <input class="input" type="number" bind:value={minHeight} />
        </label>
      </div>
      <div class="grid gap-1 children:grid">
        <label>
          Max width
          <input class="input" type="number" bind:value={maxWidth} min="800" />
        </label>
        <label>
          Max height
          <input class="input" type="number" bind:value={maxHeight} min="400" />
        </label>
      </div>
    </div>
    <div class="grid grid-cols-2 gap-2 max-inline-2xl">
      <div>
        <div class="text-accent dark:text-darkAccent font-700 m-block-1">
          Inner Size
        </div>
        <span>Width: {innerSize.width}</span>
        <span>Height: {innerSize.height}</span>
      </div>
      <div>
        <div class="text-accent dark:text-darkAccent font-700 m-block-1">
          Outer Size
        </div>
        <span>Width: {outerSize.width}</span>
        <span>Height: {outerSize.height}</span>
      </div>
      <div>
        <div class="text-accent dark:text-darkAccent font-700 m-block-1">
          Inner Logical Size
        </div>
        <span>Width: {innerSize.toLogical(scaleFactor).width.toFixed(3)}</span>
        <span>Height: {innerSize.toLogical(scaleFactor).height.toFixed(3)}</span
        >
      </div>
      <div>
        <div class="text-accent dark:text-darkAccent font-700 m-block-1">
          Outer Logical Size
        </div>
        <span>Width: {outerSize.toLogical(scaleFactor).width.toFixed(3)}</span>
        <span>Height: {outerSize.toLogical(scaleFactor).height.toFixed(3)}</span
        >
      </div>
      <div>
        <div class="text-accent dark:text-darkAccent font-700 m-block-1">
          Inner Position
        </div>
        <span>x: {innerPosition.x}</span>
        <span>y: {innerPosition.y}</span>
      </div>
      <div>
        <div class="text-accent dark:text-darkAccent font-700 m-block-1">
          Outer Position
        </div>
        <span>x: {outerPosition.x}</span>
        <span>y: {outerPosition.y}</span>
      </div>
      <div>
        <div class="text-accent dark:text-darkAccent font-700 m-block-1">
          Inner Logical Position
        </div>
        <span>x: {innerPosition.toLogical(scaleFactor).x.toFixed(3)}</span>
        <span>y: {innerPosition.toLogical(scaleFactor).y.toFixed(3)}</span>
      </div>
      <div>
        <div class="text-accent dark:text-darkAccent font-700 m-block-1">
          Outer Logical Position
        </div>
        <span>x: {outerPosition.toLogical(scaleFactor).x.toFixed(3)}</span>
        <span>y: {outerPosition.toLogical(scaleFactor).y.toFixed(3)}</span>
      </div>
    </div>
    <div class="grid gap-2">
      <h4 class="my-2">Cursor</h4>
      <div class="flex gap-2">
        <label>
          <input type="checkbox" class="checkbox" bind:checked={cursorGrab} />
          Grab
        </label>
        <label>
          <input
            type="checkbox"
            class="checkbox"
            bind:checked={cursorVisible}
          />
          Visible
        </label>
        <label>
          <input
            type="checkbox"
            class="checkbox"
            bind:checked={cursorIgnoreEvents}
          />
          Ignore events
        </label>
      </div>
      <div class="flex gap-2">
        <label>
          Icon
          <select class="input" bind:value={cursorIcon}>
            {#each cursorIconOptions as kind}
              <option value={kind}>{kind}</option>
            {/each}
          </select>
        </label>
        <label>
          X position
          <input class="input" type="number" bind:value={cursorX} />
        </label>
        <label>
          Y position
          <input class="input" type="number" bind:value={cursorY} />
        </label>
      </div>
    </div>

    <div class="flex flex-col gap-1">
      <div class="flex gap-2">
        <label>
          Progress Status
          <select
            class="input"
            bind:value={selectedProgressBarStatus}
            on:change={updateProgressBar}
          >
            {#each progressBarStatusOptions as status}
              <option value={status}>{status}</option>
            {/each}
          </select>
        </label>

        <label>
          Progress
          <input
            class="input"
            type="number"
            min="0"
            max="100"
            bind:value={progress}
            on:change={updateProgressBar}
          />
        </label>
      </div>
    </div>

    {#if isWindows || isMacOS}
      <div class="flex flex-col gap-2">
        <div class="flex items-center gap-2">
          <div>
            Applied effects: {effects.length ? effects.join(', ') : 'None'}
          </div>

          <button class="btn" on:click={clearEffects}>Clear</button>
        </div>

        <div class="flex gap-2">
          <label>
            Effect
            <select class="input" bind:value={selectedEffect}>
              {#each effectOptions as effect}
                <option value={effect}>{effect}</option>
              {/each}
            </select>
          </label>

          <label>
            State
            <select class="input" bind:value={effectState}>
              {#each effectStateOptions as state}
                <option value={state}>{state}</option>
              {/each}
            </select>
          </label>

          <label>
            Radius
            <input class="input" type="number" bind:value={effectRadius} />
          </label>
        </div>

        <div class="flex">
          <label>
            Color
            <div class="flex gap-2 children:flex-basis-30">
              <input
                class="input"
                type="number"
                placeholder="R"
                bind:value={effectR}
              />
              <input
                class="input"
                type="number"
                placeholder="G"
                bind:value={effectG}
              />
              <input
                class="input"
                type="number"
                placeholder="B"
                bind:value={effectB}
              />
              <input
                class="input"
                type="number"
                placeholder="A"
                bind:value={effectA}
              />
            </div>
          </label>
        </div>

        <div class="flex">
          <button class="btn" on:click={addEffect}>Add</button>
        </div>
      </div>
    {/if}
  {/if}
</div>
