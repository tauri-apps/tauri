<script>
  import {
    appWindow,
    WebviewWindow,
    LogicalSize,
    UserAttentionType,
    PhysicalSize,
    PhysicalPosition
  } from '@tauri-apps/api/window'
  import { open as openDialog } from '@tauri-apps/api/dialog'
  import { open } from '@tauri-apps/api/shell'

  let selectedWindow = appWindow.label
  const windowMap = {
    [appWindow.label]: appWindow
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

  export let onMessage

  let newWindowLabel

  let urlValue = 'https://tauri.app'
  let resizable = true
  let maximized = false
  let decorations = true
  let alwaysOnTop = false
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

  function openUrl() {
    open(urlValue)
  }

  function setTitle_() {
    windowMap[selectedWindow].setTitle(windowTitle)
  }

  function hide_() {
    windowMap[selectedWindow].hide()
    setTimeout(windowMap[selectedWindow].show, 2000)
  }

  function minimize_() {
    windowMap[selectedWindow].minimize()
    setTimeout(windowMap[selectedWindow].unminimize, 2000)
  }

  function getIcon() {
    openDialog({
      multiple: false
    }).then((path) => {
      if (typeof path === 'string') {
        windowMap[selectedWindow].setIcon(path)
      }
    })
  }

  function createWindow() {
    if (!newWindowLabel) return

    const webview = new WebviewWindow(newWindowLabel)
    windowMap[newWindowLabel] = webview
    webview.once('tauri://error', function () {
      onMessage('Error creating new webview')
    })
  }

  function loadWindowSize() {
    windowMap[selectedWindow].innerSize().then((response) => {
      innerSize = response
      width = innerSize.width
      height = innerSize.height
    })
    windowMap[selectedWindow].outerSize().then((response) => {
      outerSize = response
    })
  }

  function loadWindowPosition() {
    windowMap[selectedWindow].innerPosition().then((response) => {
      innerPosition = response
    })
    windowMap[selectedWindow].outerPosition().then((response) => {
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
    await windowMap[selectedWindow].minimize()
    await windowMap[selectedWindow].requestUserAttention(
      UserAttentionType.Critical
    )
    await new Promise((resolve) => setTimeout(resolve, 3000))
    await windowMap[selectedWindow].requestUserAttention(null)
  }

  $: {
    windowMap[selectedWindow]
    loadWindowPosition()
    loadWindowSize()
  }
  $: windowMap[selectedWindow]?.setResizable(resizable)
  $: maximized
    ? windowMap[selectedWindow]?.maximize()
    : windowMap[selectedWindow]?.unmaximize()
  $: windowMap[selectedWindow]?.setDecorations(decorations)
  $: windowMap[selectedWindow]?.setAlwaysOnTop(alwaysOnTop)
  $: windowMap[selectedWindow]?.setFullscreen(fullscreen)

  $: width &&
    height &&
    windowMap[selectedWindow]?.setSize(new PhysicalSize(width, height))
  $: minWidth && minHeight
    ? windowMap[selectedWindow]?.setMinSize(
        new LogicalSize(minWidth, minHeight)
      )
    : windowMap[selectedWindow]?.setMinSize(null)
  $: maxWidth > 800 && maxHeight > 400
    ? windowMap[selectedWindow]?.setMaxSize(
        new LogicalSize(maxWidth, maxHeight)
      )
    : windowMap[selectedWindow]?.setMaxSize(null)
  $: x !== null &&
    y !== null &&
    windowMap[selectedWindow]?.setPosition(new PhysicalPosition(x, y))
  $: windowMap[selectedWindow]
    ?.scaleFactor()
    .then((factor) => (scaleFactor = factor))
  $: addWindowEventListeners(windowMap[selectedWindow])

  $: windowMap[selectedWindow]?.setCursorGrab(cursorGrab)
  $: windowMap[selectedWindow]?.setCursorVisible(cursorVisible)
  $: windowMap[selectedWindow]?.setCursorIcon(cursorIcon)
  $: cursorX !== null &&
    cursorY !== null &&
    windowMap[selectedWindow]?.setCursorPosition(
      new PhysicalPosition(cursorX, cursorY)
    )
  $: windowMap[selectedWindow]?.setIgnoreCursorEvents(cursorIgnoreEvents)
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
  {#if Object.keys(windowMap).length >= 1}
    <span class="font-700 text-sm">Selected window:</span>
    <select class="input" bind:value={selectedWindow}>
      <option value="" disabled selected>Choose a window...</option>
      {#each Object.keys(windowMap) as label}
        <option value={label}>{label}</option>
      {/each}
    </select>
  {/if}
  {#if windowMap[selectedWindow]}
    <br />
    <div class="flex flex-wrap gap-2">
      <button
        class="btn"
        title="Unminimizes after 2 seconds"
        on:click={() => windowMap[selectedWindow].center()}
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
      <button class="btn" on:click={getIcon}> Change icon </button>
      <button
        class="btn"
        on:click={requestUserAttention_}
        title="Minimizes the window, requests attention for 3s and then resets it"
        >Request attention</button
      >
    </div>
    <br />
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
        Has decorations
        <input type="checkbox" bind:checked={decorations} />
      </label>
      <label>
        Always on top
        <input type="checkbox" bind:checked={alwaysOnTop} />
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
      <form class="flex gap-1" on:submit|preventDefault={openUrl}>
        <input class="input grow" id="url" bind:value={urlValue} />
        <button class="btn" id="open-url"> Open URL </button>
      </form>
    </div>
  {/if}
</div>
