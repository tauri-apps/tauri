<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
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
  import type { CursorIcon, Effects, Theme } from '@tauri-apps/api/window'
  import type { UnlistenFn } from '@tauri-apps/api/event'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()

  const webview = WebviewWindow.getCurrent()

  const webviewMap = $state({
    [webview.label]: webview
  })
  let selectedWebviewLabel = $state(webview.label)
  let selectedWebview = $derived(webviewMap[selectedWebviewLabel])

  let focusable = $state(true)

  const cursorIconOptions: CursorIcon[] = [
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

  const windowsEffects: Effect[] = [
    Effect.Mica,
    Effect.Blur,
    Effect.Acrylic,
    Effect.Tabbed,
    Effect.TabbedDark,
    Effect.TabbedLight
  ]
  const isWindows = navigator.appVersion.includes('Windows')
  const isMacOS = navigator.appVersion.includes('Macintosh')
  const effectOptions: Effect[] = isWindows
    ? windowsEffects
    : (Object.values(Effect) as Effect[]).filter(
        (e) => !windowsEffects.includes(e)
      )
  const effectStateOptions = Object.values(EffectState) as EffectState[]

  const progressBarStatusOptions = Object.values(ProgressBarStatus)

  const mainEl = document.querySelector('main')!

  let newWebviewLabel = $state<string>()

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
  let width = $state(100)
  let height = $state(100)
  let minWidth = $state<number | null>(null)
  let minHeight = $state<number | null>(null)
  let maxWidth = $state<number | null>(null)
  let maxHeight = $state<number | null>(null)
  let x = $state(0)
  let y = $state(0)
  let scaleFactor = $state(1)
  let innerPosition = $state(new PhysicalPosition(0, 0))
  let outerPosition = $state(new PhysicalPosition(0, 0))
  let innerSize = $state(new PhysicalSize(0, 0))
  let outerSize = $state(new PhysicalSize(0, 0))
  let resizeEventUnlisten: UnlistenFn | undefined
  let moveEventUnlisten: UnlistenFn | undefined
  let cursorGrab = $state(false)
  let cursorVisible = $state(true)
  let cursorX = $state<number | null>(null)
  let cursorY = $state<number | null>(null)
  let cursorIcon = $state<CursorIcon>('default')
  let cursorIgnoreEvents = $state(false)
  let windowTitle = $state('Awesome Tauri Example!')

  let theme = $state<Theme | 'auto'>('auto')

  let effects = $state<Effect[]>([])
  let selectedEffect = $state<Effect>()
  let effectState = $state<EffectState>()
  let effectRadius = $state<number>()
  let effectR = $state<number>(),
    effectG = $state<number>(),
    effectB = $state<number>(),
    effectA = $state<number>()

  let selectedProgressBarStatus = $state<ProgressBarStatus>(
    ProgressBarStatus.None
  )
  let progress = $state(0)

  let windowIconPath = $state('')

  function setTitle() {
    selectedWebview.setTitle(windowTitle)
  }

  async function hide() {
    let visible = await selectedWebview.isVisible()
    onMessage('window is ' + (visible ? 'visible' : 'invisible'))
    await selectedWebview.hide()

    setTimeout(async () => {
      visible = await selectedWebview.isVisible()
      onMessage('window is ' + (visible ? 'visible' : 'invisible'))

      await selectedWebview.show()
      visible = await selectedWebview.isVisible()
      onMessage('window is ' + (visible ? 'visible' : 'invisible'))
    }, 2000)
  }

  async function disable() {
    let enabled = await selectedWebview.isEnabled()
    onMessage('window is ' + (enabled ? 'enabled' : 'disabled'))

    await selectedWebview.setEnabled(false)

    setTimeout(async () => {
      enabled = await selectedWebview.isEnabled()
      onMessage('window is ' + (enabled ? 'enabled' : 'disabled'))

      await selectedWebview.setEnabled(true)
      enabled = await selectedWebview.isEnabled()
      onMessage('window is ' + (enabled ? 'enabled' : 'disabled'))
    }, 2000)
  }

  function minimize() {
    selectedWebview.minimize()
    setTimeout(selectedWebview.unminimize, 2000)
  }

  function changeIcon() {
    selectedWebview.setIcon(windowIconPath)
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
    selectedWebview.innerSize().then((response) => {
      innerSize = response
      width = innerSize.width
      height = innerSize.height
    })
    selectedWebview.outerSize().then((response) => {
      outerSize = response
    })
  }

  function loadWindowPosition() {
    selectedWebview.innerPosition().then((response) => {
      innerPosition = response
    })
    selectedWebview.outerPosition().then((response) => {
      outerPosition = response
      x = outerPosition.x
      y = outerPosition.y
    })
  }

  async function addWindowEventListeners(window: WebviewWindow | undefined) {
    if (!window) return
    resizeEventUnlisten?.()
    moveEventUnlisten?.()
    moveEventUnlisten = await window.listen('tauri://move', loadWindowPosition)
    resizeEventUnlisten = await window.listen('tauri://resize', loadWindowSize)
  }

  async function requestUserAttention() {
    await selectedWebview.minimize()
    await selectedWebview.requestUserAttention(UserAttentionType.Critical)
    await new Promise((resolve) => setTimeout(resolve, 3000))
    await selectedWebview.requestUserAttention(null)
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
    await selectedWebview.setTheme(theme === 'auto' ? null : theme)
  }

  async function updateProgressBar() {
    selectedWebview.setProgressBar({
      status: selectedProgressBarStatus,
      progress
    })
  }

  async function addEffect() {
    if (selectedEffect && !effects.includes(selectedEffect)) {
      effects = [...effects, selectedEffect]
    }

    const payload: Effects = {
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
      payload.color = [effectR!, effectG!, effectB!, effectA!]
    }

    mainEl.classList.remove('bg-primary')
    mainEl.classList.remove('dark:bg-darkPrimary')
    await selectedWebview.clearEffects()
    await selectedWebview.setEffects(payload)
  }

  async function clearEffects() {
    effects = []
    await selectedWebview.clearEffects()
    mainEl.classList.add('bg-primary')
    mainEl.classList.add('dark:bg-darkPrimary')
  }

  async function updatePosition() {
    selectedWebview.setPosition(new PhysicalPosition(x, y))
  }

  async function updateSize() {
    if (width && height) {
      selectedWebview.setSize(new PhysicalSize(width, height))
    }
  }

  function refreshSelectedWebview() {
    loadWindowPosition()
    loadWindowSize()
    selectedWebview.scaleFactor().then((factor) => (scaleFactor = factor))
    addWindowEventListeners(selectedWebview)
  }

  function updateResizable() {
    selectedWebview.setResizable(resizable)
  }

  function updateMaximizable() {
    selectedWebview.setMaximizable(maximizable)
  }

  function updateMinimizable() {
    selectedWebview.setMinimizable(minimizable)
  }

  function updateClosable() {
    selectedWebview.setClosable(closable)
  }

  function updateMaximized() {
    maximized ? selectedWebview.maximize() : selectedWebview.unmaximize()
  }

  function updateDecorations() {
    selectedWebview.setDecorations(decorations)
  }

  function updateAlwaysOnTop() {
    selectedWebview.setAlwaysOnTop(alwaysOnTop)
  }

  function updateAlwaysOnBottom() {
    selectedWebview.setAlwaysOnBottom(alwaysOnBottom)
  }

  function updateContentProtected() {
    selectedWebview.setContentProtected(contentProtected)
  }

  function updateFullscreen() {
    selectedWebview.setFullscreen(fullscreen)
  }

  function updateSimpleFullscreen() {
    selectedWebview.setSimpleFullscreen(simpleFullscreen)
  }

  function updateMinSize() {
    minWidth && minHeight
      ? selectedWebview.setMinSize(new LogicalSize(minWidth, minHeight))
      : selectedWebview.setMinSize(null)
  }

  function updateMaxSize() {
    maxWidth && maxHeight && maxWidth > 800 && maxHeight > 400
      ? selectedWebview.setMaxSize(new LogicalSize(maxWidth, maxHeight))
      : selectedWebview.setMaxSize(null)
  }

  function updateCursorGrab() {
    selectedWebview.setCursorGrab(cursorGrab)
  }

  function updateCursorVisible() {
    selectedWebview.setCursorVisible(cursorVisible)
  }

  function updateCursorIgnoreEvents() {
    selectedWebview.setIgnoreCursorEvents(cursorIgnoreEvents)
  }

  function updateCursorIcon() {
    selectedWebview.setCursorIcon(cursorIcon)
  }

  function updateCursorPosition() {
    cursorX !== null
      && cursorY !== null
      && selectedWebview.setCursorPosition(
        new PhysicalPosition(cursorX, cursorY)
      )
  }

  $effect(() => {
    selectedWebview
    refreshSelectedWebview()
  })

  onDestroy(() => {
    resizeEventUnlisten?.()
    moveEventUnlisten?.()
  })
</script>

<div class="flex flex-col *:grow gap-8 mb-4">
  <div
    class="flex flex-wrap items-center gap-4 pb-6 border-b-solid border-b-1 border-code"
  >
    {#if Object.keys(webviewMap).length >= 1}
      <div class="grid gap-1">
        <h4 class="my-2">Selected Window</h4>
        <select
          class="input"
          bind:value={selectedWebviewLabel}
          onchange={refreshSelectedWebview}
        >
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
  {#if selectedWebview}
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
        onclick={() => selectedWebview.center()}
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
          selectedWebview.setFocusable(!focusable)
        }}
      >
        Set focusable to {!focusable}
      </button>
    </div>
    <div class="grid cols-[repeat(auto-fill,minmax(180px,1fr))] *:flex *:gap-2">
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={resizable}
          onchange={updateResizable}
        />
        Resizable
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={maximizable}
          onchange={updateMaximizable}
        />
        Maximizable
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={minimizable}
          onchange={updateMinimizable}
        />
        Minimizable
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={closable}
          onchange={updateClosable}
        />
        Closable
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={decorations}
          onchange={updateDecorations}
        />
        Has decorations
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={alwaysOnTop}
          onchange={updateAlwaysOnTop}
        />
        Always on top
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={alwaysOnBottom}
          onchange={updateAlwaysOnBottom}
        />
        Always on bottom
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={contentProtected}
          onchange={updateContentProtected}
        />
        Content protected
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={maximized}
          onchange={updateMaximized}
        />
        Maximized
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={fullscreen}
          onchange={updateFullscreen}
        />
        Fullscreen
      </label>
      <label>
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={simpleFullscreen}
          onchange={updateSimpleFullscreen}
        />
        Simple fullscreen
      </label>
    </div>
    <div class="flex flex-wrap *:flex-basis-30 gap-2">
      <div class="grid gap-1 *:grid">
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
      <div class="grid gap-1 *:grid">
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
      <div class="grid gap-1 *:grid">
        <label>
          Min width
          <input
            class="input"
            type="number"
            bind:value={minWidth}
            onchange={updateMinSize}
          />
        </label>
        <label>
          Min height
          <input
            class="input"
            type="number"
            bind:value={minHeight}
            onchange={updateMinSize}
          />
        </label>
      </div>
      <div class="grid gap-1 *:grid">
        <label>
          Max width
          <input
            class="input"
            type="number"
            bind:value={maxWidth}
            onchange={updateMaxSize}
            min="800"
          />
        </label>
        <label>
          Max height
          <input
            class="input"
            type="number"
            bind:value={maxHeight}
            onchange={updateMaxSize}
            min="400"
          />
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
      <div class="flex gap-4 *:flex *:gap-2">
        <label>
          <input
            type="checkbox"
            class="checkbox"
            bind:checked={cursorGrab}
            onchange={updateCursorGrab}
          />
          Grab
        </label>
        <label>
          <input
            type="checkbox"
            class="checkbox"
            bind:checked={cursorVisible}
            onchange={updateCursorVisible}
          />
          Visible
        </label>
        <label>
          <input
            type="checkbox"
            class="checkbox"
            bind:checked={cursorIgnoreEvents}
            onchange={updateCursorIgnoreEvents}
          />
          Ignore events
        </label>
      </div>
      <div class="flex gap-2">
        <label>
          Icon
          <select
            class="input"
            bind:value={cursorIcon}
            onchange={updateCursorIcon}
          >
            {#each cursorIconOptions as kind}
              <option value={kind}>{kind}</option>
            {/each}
          </select>
        </label>
        <label>
          X position
          <input
            class="input"
            type="number"
            bind:value={cursorX}
            onchange={updateCursorPosition}
          />
        </label>
        <label>
          Y position
          <input
            class="input"
            type="number"
            bind:value={cursorY}
            onchange={updateCursorPosition}
          />
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
            <div class="flex gap-2 *:flex-basis-30">
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
