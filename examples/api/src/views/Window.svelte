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

  let { onMessage } = $props()

  const webview = WebviewWindow.getCurrent()

  let selectedWebview = $state(webview.label)
  const webviewMap = $state({
    [webview.label]: webview
  })

  let focusable = $state(true)

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

  let newWebviewLabel = $state()

  let resizable = $state(true)
  let maximizable = $state(true)
  let minimizable = $state(true)
  let closable = $state(true)
  let maximized = $state(false)
  let decorations = $state(true)
  let alwaysOnTop = $state(false)
  let alwaysOnBottom = $state(false)
  let contentProtected = $state(false)
  let fullscreen = $state(false)
  let simpleFullscreen = $state(false)
  let width = $state(null)
  let height = $state(null)
  let minWidth = $state(null)
  let minHeight = $state(null)
  let maxWidth = $state(null)
  let maxHeight = $state(null)
  let x = $state(null)
  let y = $state(null)
  let scaleFactor = $state(1)
  let innerPosition = $state(new PhysicalPosition(0, 0))
  let outerPosition = $state(new PhysicalPosition(0, 0))
  let innerSize = $state(new PhysicalSize(0, 0))
  let outerSize = $state(new PhysicalSize(0, 0))
  let resizeEventUnlisten
  let moveEventUnlisten
  let cursorGrab = $state(false)
  let cursorVisible = $state(true)
  let cursorX = $state(null)
  let cursorY = $state(null)
  /** @type {import('@tauri-apps/api/window').CursorIcon} */
  let cursorIcon = $state('default')
  let cursorIgnoreEvents = $state(false)
  let windowTitle = $state('Awesome Tauri Example!')

  /** @type {import('@tauri-apps/api/window').Theme | 'auto'} */
  let theme = $state('auto')

  let effects = $state([])
  let selectedEffect = $state()
  let effectState = $state()
  let effectRadius = $state()
  let effectR = $state(),
    effectG = $state(),
    effectB = $state(),
    effectA = $state()

  /** @type {ProgressBarStatus} */
  let selectedProgressBarStatus = $state(ProgressBarStatus.None)
  let progress = $state(0)

  let windowIconPath = $state()

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
      Number.isInteger(effectR)
      && Number.isInteger(effectG)
      && Number.isInteger(effectB)
      && Number.isInteger(effectA)
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

  $effect(() => {
    webviewMap[selectedWebview]
    loadWindowPosition()
    loadWindowSize()
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setResizable(resizable)
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setMaximizable(maximizable)
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setMinimizable(minimizable)
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setClosable(closable)
  })
  $effect(() => {
    maximized
      ? webviewMap[selectedWebview]?.maximize()
      : webviewMap[selectedWebview]?.unmaximize()
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setDecorations(decorations)
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setAlwaysOnTop(alwaysOnTop)
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setAlwaysOnBottom(alwaysOnBottom)
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setContentProtected(contentProtected)
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setFullscreen(fullscreen)
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setSimpleFullscreen(simpleFullscreen)
  })

  $effect(() => {
    minWidth && minHeight
      ? webviewMap[selectedWebview]?.setMinSize(
          new LogicalSize(minWidth, minHeight)
        )
      : webviewMap[selectedWebview]?.setMinSize(null)
  })
  $effect(() => {
    maxWidth > 800 && maxHeight > 400
      ? webviewMap[selectedWebview]?.setMaxSize(
          new LogicalSize(maxWidth, maxHeight)
        )
      : webviewMap[selectedWebview]?.setMaxSize(null)
  })
  $effect(() => {
    webviewMap[selectedWebview]
      ?.scaleFactor()
      .then((factor) => (scaleFactor = factor))
  })
  $effect(() => {
    addWindowEventListeners(webviewMap[selectedWebview])
  })

  $effect(() => {
    webviewMap[selectedWebview]?.setCursorGrab(cursorGrab)
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setCursorVisible(cursorVisible)
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setCursorIcon(cursorIcon)
  })
  $effect(() => {
    cursorX !== null
      && cursorY !== null
      && webviewMap[selectedWebview]?.setCursorPosition(
        new PhysicalPosition(cursorX, cursorY)
      )
  })
  $effect(() => {
    webviewMap[selectedWebview]?.setIgnoreCursorEvents(cursorIgnoreEvents)
  })

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
      <form
        class="flex gap-2"
        onsubmit={(ev) => {
          createWebviewWindow()
          ev.preventDefault()
        }}
      >
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
        <form
          class="flex gap-2"
          onsubmit={(ev) => {
            changeIcon()
            ev.preventDefault()
          }}
        >
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
        <form
          class="flex gap-2"
          onsubmit={(ev) => {
            setTitle()
            ev.preventDefault()
          }}
        >
          <input class="input flex-1 min-w-10" bind:value={windowTitle} />
          <button class="btn" type="submit">Set</button>
        </form>
      </div>
    </div>
    <div class="flex flex-wrap gap-2">
      <button
        class="btn"
        title="Unminimizes after 2 seconds"
        onclick={() => webviewMap[selectedWebview].center()}
      >
        Center
      </button>
      <button
        class="btn"
        title="Unminimizes after 2 seconds"
        onclick={minimize}
      >
        Minimize
      </button>
      <button class="btn" title="Visible again after 2 seconds" onclick={hide}>
        Hide
      </button>
      <button
        class="btn"
        title="Enabled again after 2 seconds"
        onclick={disable}
      >
        Disable
      </button>
      <button
        class="btn"
        onclick={requestUserAttention}
        title="Minimizes the window, requests attention for 3s and then resets it"
        >Request attention</button
      >
      <button class="btn" onclick={switchTheme}>Switch Theme ({theme})</button>
      <button
        class="btn"
        onclick={() => {
          focusable = !focusable
          webviewMap[selectedWebview].setFocusable(!focusable)
        }}
      >
        Set focusable to {!focusable}
      </button>
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
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={simpleFullscreen}
        />
        Simple fullscreen
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
            onchange={updatePosition}
            min="0"
          />
        </label>
        <label>
          Y
          <input
            class="input"
            type="number"
            bind:value={y}
            onchange={updatePosition}
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
            onchange={updateSize}
            min="400"
          />
        </label>
        <div>
          Height
          <input
            class="input"
            type="number"
            bind:value={height}
            onchange={updateSize}
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
            onchange={updateProgressBar}
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
            onchange={updateProgressBar}
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

          <button class="btn" onclick={clearEffects}>Clear</button>
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
          <button class="btn" onclick={addEffect}>Add</button>
        </div>
      </div>
    {/if}
  {/if}
</div>
