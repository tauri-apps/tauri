<script>
  import {
    LogicalSize,
    UserAttentionType,
    PhysicalSize,
    PhysicalPosition,
    Effect,
    EffectState,
    ProgressBarStatus,
    Window
  } from '@tauri-apps/api/window'
  import { getCurrent, Webview } from '@tauri-apps/api/webview'
  import { invoke } from '@tauri-apps/api/primitives'

  const webview = getCurrent()

  let selectedWindow = webview.label
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

  export let onMessage
  const mainEl = document.querySelector('main')

  let newWindowLabel

  let urlValue = 'https://tauri.app'
  let resizable = true
  let maximizable = true
  let minimizable = true
  let closable = true
  let maximized = false
  let decorations = true
  let alwaysOnTop = false
  let alwaysOnBottom = false
  let contentProtected = true
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
  let cursorIcon = 'default'
  let cursorIgnoreEvents = false
  let windowTitle = 'Awesome Tauri Example!'

  let effects = []
  let selectedEffect
  let effectState
  let effectRadius
  let effectR, effectG, effectB, effectA

  let selectedProgressBarStatus = 'none'
  let progress = 0

  let windowIconPath

  function setTitle_() {
    webviewMap[selectedWindow].window.setTitle(windowTitle)
  }

  function hide_() {
    webviewMap[selectedWindow].window.hide()
    setTimeout(webviewMap[selectedWindow].window.show, 2000)
  }

  function minimize_() {
    webviewMap[selectedWindow].window.minimize()
    setTimeout(webviewMap[selectedWindow].window.unminimize, 2000)
  }

  function changeIcon() {
    webviewMap[selectedWindow].window.setIcon(path)
  }

  function createWindow() {
    if (!newWindowLabel) return

    const window = new Window(newWindowLabel, { visible: false })
    window.once('tauri://error', function (e) {
      onMessage('Error creating new window ' + JSON.stringify(e))
    })
    window.once('tauri://created', function () {
      onMessage('window created')
      const webview = new Webview(window, newWindowLabel)
      window.show()
      webviewMap[newWindowLabel] = webview
      webview.once('tauri://error', function (e) {
        onMessage('Error creating new webview ' + JSON.stringify(e))
      })
    })
  }

  function loadWindowSize() {
    webviewMap[selectedWindow].window.innerSize().then((response) => {
      innerSize = response
      width = innerSize.width
      height = innerSize.height
    })
    webviewMap[selectedWindow].window.outerSize().then((response) => {
      outerSize = response
    })
  }

  function loadWindowPosition() {
    webviewMap[selectedWindow].window.innerPosition().then((response) => {
      innerPosition = response
    })
    webviewMap[selectedWindow].window.outerPosition().then((response) => {
      outerPosition = response
      x = outerPosition.x
      y = outerPosition.y
    })
  }

  async function addWindowEventListeners(window) {
    if (!window) return
    if (resizeEventUnlisten) {
      resizeEventUnlisten()
    }
    if (moveEventUnlisten) {
      moveEventUnlisten()
    }
    moveEventUnlisten = await window.listen('tauri://move', loadWindowPosition)
    resizeEventUnlisten = await window.listen('tauri://resize', loadWindowSize)
  }

  async function requestUserAttention_() {
    await webviewMap[selectedWindow].window.minimize()
    await webviewMap[selectedWindow].window.requestUserAttention(
      UserAttentionType.Critical
    )
    await new Promise((resolve) => setTimeout(resolve, 3000))
    await webviewMap[selectedWindow].window.requestUserAttention(null)
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
    await webviewMap[selectedWindow].window.clearEffects()
    await webviewMap[selectedWindow].window.setEffects(payload)
  }

  async function clearEffects() {
    effects = []
    await webviewMap[selectedWindow].window.clearEffects()
    mainEl.classList.add('bg-primary')
    mainEl.classList.add('dark:bg-darkPrimary')
  }

  $: {
    webviewMap[selectedWindow]
    loadWindowPosition()
    loadWindowSize()
  }
  $: webviewMap[selectedWindow]?.window.setResizable(resizable)
  $: webviewMap[selectedWindow]?.window.setMaximizable(maximizable)
  $: webviewMap[selectedWindow]?.window.setMinimizable(minimizable)
  $: webviewMap[selectedWindow]?.window.setClosable(closable)
  $: maximized
    ? webviewMap[selectedWindow]?.window.maximize()
    : webviewMap[selectedWindow]?.window.unmaximize()
  $: webviewMap[selectedWindow]?.window.setDecorations(decorations)
  $: webviewMap[selectedWindow]?.window.setAlwaysOnTop(alwaysOnTop)
  $: webviewMap[selectedWindow]?.window.setAlwaysOnBottom(alwaysOnBottom)
  $: webviewMap[selectedWindow]?.window.setContentProtected(contentProtected)
  $: webviewMap[selectedWindow]?.window.setFullscreen(fullscreen)

  $: width &&
    height &&
    webviewMap[selectedWindow]?.window.setSize(new PhysicalSize(width, height))
  $: minWidth && minHeight
    ? webviewMap[selectedWindow]?.window.setMinSize(
        new LogicalSize(minWidth, minHeight)
      )
    : webviewMap[selectedWindow]?.window.setMinSize(null)
  $: maxWidth > 800 && maxHeight > 400
    ? webviewMap[selectedWindow]?.window.setMaxSize(
        new LogicalSize(maxWidth, maxHeight)
      )
    : webviewMap[selectedWindow]?.window.setMaxSize(null)
  $: x !== null &&
    y !== null &&
    webviewMap[selectedWindow].window?.setPosition(new PhysicalPosition(x, y))
  $: webviewMap[selectedWindow]?.window
    .scaleFactor()
    .then((factor) => (scaleFactor = factor))
  $: addWindowEventListeners(webviewMap[selectedWindow].window)

  $: webviewMap[selectedWindow]?.window.setCursorGrab(cursorGrab)
  $: webviewMap[selectedWindow]?.window.setCursorVisible(cursorVisible)
  $: webviewMap[selectedWindow]?.window.setCursorIcon(cursorIcon)
  $: cursorX !== null &&
    cursorY !== null &&
    webviewMap[selectedWindow]?.window.setCursorPosition(
      new PhysicalPosition(cursorX, cursorY)
    )
  $: webviewMap[selectedWindow]?.window.setIgnoreCursorEvents(
    cursorIgnoreEvents
  )
  $: webviewMap[selectedWindow]?.window.setProgressBar({
    status: selectedProgressBarStatus,
    progress
  })
</script>

<div class="flex flex-col children:grow gap-2">
  <div class="flex gap-1">
    <input
      class="input grow"
      type="text"
      placeholder="New Window label.."
      bind:value={newWindowLabel}
    />
    <button class="btn" on:click={createWindow}>New window</button>
  </div>
  <br />
  {#if Object.keys(webviewMap).length >= 1}
    <span class="font-700 text-sm">Selected window:</span>
    <select class="input" bind:value={selectedWindow}>
      <option value="" disabled selected>Choose a window...</option>
      {#each Object.keys(webviewMap) as label}
        <option value={label}>{label}</option>
      {/each}
    </select>
  {/if}
  {#if webviewMap[selectedWindow]}
    <br />
    <div class="flex gap-1 items-center">
      <label> Icon path </label>
      <form class="flex gap-1 grow" on:submit|preventDefault={setTitle_}>
        <input class="input grow" bind:value={windowIconPath} />
        <button class="btn" type="submit"> Change window icon </button>
      </form>
    </div>
    <br />
    <div class="flex flex-wrap gap-2">
      <button
        class="btn"
        title="Unminimizes after 2 seconds"
        on:click={() => webviewMap[selectedWindow].window.center()}
      >
        Center
      </button>
      <button
        class="btn"
        title="Unminimizes after 2 seconds"
        on:click={minimize_}
      >
        Minimize
      </button>
      <button
        class="btn"
        title="Visible again after 2 seconds"
        on:click={hide_}
      >
        Hide
      </button>
      <button
        class="btn"
        on:click={requestUserAttention_}
        title="Minimizes the window, requests attention for 3s and then resets it"
        >Request attention</button
      >
    </div>

    <div class="flex flex-wrap gap-2">
      <label>
        Maximized
        <input type="checkbox" bind:checked={maximized} />
      </label>
      <label>
        Resizable
        <input type="checkbox" bind:checked={resizable} />
      </label>
      <label>
        Maximizable
        <input type="checkbox" bind:checked={maximizable} />
      </label>
      <label>
        Minimizable
        <input type="checkbox" bind:checked={minimizable} />
      </label>
      <label>
        Closable
        <input type="checkbox" bind:checked={closable} />
      </label>
      <label>
        Has decorations
        <input type="checkbox" bind:checked={decorations} />
      </label>
      <label>
        Always on top
        <input type="checkbox" bind:checked={alwaysOnTop} />
      </label>
      <label>
        Always on bottom
        <input type="checkbox" bind:checked={alwaysOnBottom} />
      </label>
      <label>
        Content protected
        <input type="checkbox" bind:checked={contentProtected} />
      </label>
      <label>
        Fullscreen
        <input type="checkbox" bind:checked={fullscreen} />
      </label>
    </div>
    <br />
    <div class="flex flex-row gap-2 flex-wrap">
      <div class="flex children:grow flex-col">
        <div>
          X
          <input class="input" type="number" bind:value={x} min="0" />
        </div>
        <div>
          Y
          <input class="input" type="number" bind:value={y} min="0" />
        </div>
      </div>

      <div class="flex children:grow flex-col">
        <div>
          Width
          <input class="input" type="number" bind:value={width} min="400" />
        </div>
        <div>
          Height
          <input class="input" type="number" bind:value={height} min="400" />
        </div>
      </div>

      <div class="flex children:grow flex-col">
        <div>
          Min width
          <input class="input" type="number" bind:value={minWidth} />
        </div>
        <div>
          Min height
          <input class="input" type="number" bind:value={minHeight} />
        </div>
      </div>

      <div class="flex children:grow flex-col">
        <div>
          Max width
          <input class="input" type="number" bind:value={maxWidth} min="800" />
        </div>
        <div>
          Max height
          <input class="input" type="number" bind:value={maxHeight} min="400" />
        </div>
      </div>
    </div>
    <br />
    <div>
      <div class="flex">
        <div class="grow">
          <div class="text-accent dark:text-darkAccent font-700">
            Inner Size
          </div>
          <span>Width: {innerSize.width}</span>
          <span>Height: {innerSize.height}</span>
        </div>
        <div class="grow">
          <div class="text-accent dark:text-darkAccent font-700">
            Outer Size
          </div>
          <span>Width: {outerSize.width}</span>
          <span>Height: {outerSize.height}</span>
        </div>
      </div>
      <div class="flex">
        <div class="grow">
          <div class="text-accent dark:text-darkAccent font-700">
            Inner Logical Size
          </div>
          <span>Width: {innerSize.toLogical(scaleFactor).width}</span>
          <span>Height: {innerSize.toLogical(scaleFactor).height}</span>
        </div>
        <div class="grow">
          <div class="text-accent dark:text-darkAccent font-700">
            Outer Logical Size
          </div>
          <span>Width: {outerSize.toLogical(scaleFactor).width}</span>
          <span>Height: {outerSize.toLogical(scaleFactor).height}</span>
        </div>
      </div>
      <div class="flex">
        <div class="grow">
          <div class="text-accent dark:text-darkAccent font-700">
            Inner Position
          </div>
          <span>x: {innerPosition.x}</span>
          <span>y: {innerPosition.y}</span>
        </div>
        <div class="grow">
          <div class="text-accent dark:text-darkAccent font-700">
            Outer Position
          </div>
          <span>x: {outerPosition.x}</span>
          <span>y: {outerPosition.y}</span>
        </div>
      </div>
      <div class="flex">
        <div class="grow">
          <div class="text-accent dark:text-darkAccent font-700">
            Inner Logical Position
          </div>
          <span>x: {innerPosition.toLogical(scaleFactor).x}</span>
          <span>y: {innerPosition.toLogical(scaleFactor).y}</span>
        </div>
        <div class="grow">
          <div class="text-accent dark:text-darkAccent font-700">
            Outer Logical Position
          </div>
          <span>x: {outerPosition.toLogical(scaleFactor).x}</span>
          <span>y: {outerPosition.toLogical(scaleFactor).y}</span>
        </div>
      </div>
    </div>
    <br />
    <h4 class="mb-2">Cursor</h4>
    <div class="flex gap-2">
      <label>
        <input type="checkbox" bind:checked={cursorGrab} />
        Grab
      </label>
      <label>
        <input type="checkbox" bind:checked={cursorVisible} />
        Visible
      </label>
      <label>
        <input type="checkbox" bind:checked={cursorIgnoreEvents} />
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
    <br />
    <div class="flex flex-col gap-1">
      <form class="flex gap-1" on:submit|preventDefault={setTitle_}>
        <input class="input grow" id="title" bind:value={windowTitle} />
        <button class="btn" type="submit">Set title</button>
      </form>
    </div>

    <br />

    <div class="flex flex-col gap-1">
      <div class="flex gap-2">
        <label>
          Progress Status
          <select class="input" bind:value={selectedProgressBarStatus}>
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
          />
        </label>
      </div>
    </div>

    {#if isWindows || isMacOS}
      <div class="flex flex-col gap-1">
        <div class="flex">
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
            <div class="flex">
              <input
                style="max-width: 120px;"
                class="input"
                type="number"
                placeholder="R"
                bind:value={effectR}
              />
              <input
                style="max-width: 120px;"
                class="input"
                type="number"
                placeholder="G"
                bind:value={effectG}
              />
              <input
                style="max-width: 120px;"
                class="input"
                type="number"
                placeholder="B"
                bind:value={effectB}
              />
              <input
                style="max-width: 120px;"
                class="input"
                type="number"
                placeholder="A"
                bind:value={effectA}
              />
            </div>
          </label>
        </div>

        <div class="flex">
          <button class="btn" style="width: 80px;" on:click={addEffect}
            >Add</button
          >
        </div>

        <div class="flex">
          <div>
            Applied effects: {effects.length ? effects.join(',') : 'None'}
          </div>

          <button class="btn" style="width: 80px;" on:click={clearEffects}
            >Clear</button
          >
        </div>
      </div>
    {/if}
  {/if}
</div>
