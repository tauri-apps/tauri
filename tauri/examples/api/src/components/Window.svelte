<script>
  import {
    setResizable,
    setTitle as setTitle,
    maximize,
    unmaximize,
    minimize,
    unminimize,
    show,
    hide,
    setTransparent,
    setDecorations,
    setAlwaysOnTop,
    setWidth,
    setHeight,
    // resize,
    setMinSize,
    setMaxSize,
    setX,
    setY,
    // setPosition,
    setFullscreen,
    setIcon
  } from "@tauri-apps/api/window";
  import { open as openDialog } from '@tauri-apps/api/dialog'
  import { open } from "@tauri-apps/api/shell";

  let urlValue = "https://tauri.studio";
  let resizable = true
  let maximized = false
  let transparent = false
  let decorations = false
  let alwaysOnTop = false
  let fullscreen = false
  let width = 600
  let height = 600
  let minWidth = 600
  let minHeight = 600
  let maxWidth = null
  let maxHeight = null
  let x = 300
  let y = 300

  let windowTitle = 'Awesome Tauri Example!';

  function openUrl() {
    open(urlValue);
  }

  function setTitle_() {
    setTitle(windowTitle);
  }

  function hide_() {
    hide()
    setTimeout(show, 2000)
  }

  function minimize_() {
    minimize()
    setTimeout(unminimize, 2000)
  }

  function getIcon() {
    openDialog({
      multiple: false
    }).then(setIcon)
  }

  $: setResizable(resizable)
  $: maximized ? maximize() : unmaximize()
  $: setTransparent(transparent)
  $: setDecorations(decorations)
  $: setAlwaysOnTop(alwaysOnTop)
  $: setFullscreen(fullscreen)

  $: setWidth(width)
  $: setHeight(height)
  $: minWidth && minHeight && setMinSize(minWidth, minHeight)
  $: maxWidth && maxHeight && setMaxSize(maxWidth, maxHeight)
  $: setX(x)
  $: setY(y)
</script>

<style>
  .flex {
    display: flex;
  }

  .flex-row {
    flex-direction: row;
  }

  .flex-column {
    flex-direction: column;
  }

  .grow {
    flex-grow: 1;
  }

  .window-controls input {
    width: 50px;
  }
</style>

<div class="flex flex-column">
  <div>
    <label>
      <input type="checkbox" bind:checked={resizable}>
      Resizable
    </label>
    <label>
      <input type="checkbox" bind:checked={maximized}>
      Maximize
    </label>
    <button title="Unminimizes after 2 seconds" on:click={minimize_}>
      Minimize
    </button>
    <button title="Visible again after 2 seconds" on:click={hide_}>
      Hide
    </button>
    <label>
      <input type="checkbox" bind:checked={transparent}>
      Transparent
    </label>
    <label>
      <input type="checkbox" bind:checked={decorations}>
      Has decorations
    </label>
    <label>
      <input type="checkbox" bind:checked={alwaysOnTop}>
      Always on top
    </label>
    <label>
      <input type="checkbox" bind:checked={fullscreen}>
      Fullscreen
    </label>
    <button on:click={getIcon}>
      Change icon
    </button>
  </div>
  <div>
    <div class="window-controls flex flex-row">
      <div class="flex flex-column grow">
        <div>
          X
          <input type="number" bind:value={x} min="0">
        </div>
        <div>
          Y
          <input type="number" bind:value={y} min="0">
        </div>
      </div>

      <div class="flex flex-column grow">
        <div>
          Width
          <input type="number" bind:value={width} min="400">
        </div>
        <div>
          Height
          <input type="number" bind:value={height} min="400">
        </div>
      </div>

      <div class="flex flex-column grow">
        <div>
          Min width
          <input type="number" bind:value={minWidth}>
        </div>
        <div>
          Min height
          <input type="number" bind:value={minHeight}>
        </div>
      </div>

      <div class="flex flex-column grow">
        <div>
          Max width
          <input type="number" bind:value={maxWidth} min="400">
        </div>
        <div>
          Max height
          <input type="number" bind:value={maxHeight} min="400">
        </div>
      </div>
    </div>
  </div>
</div>
<form style="margin-top: 24px" on:submit|preventDefault={setTitle_}>
  <input id="title" bind:value={windowTitle} />
  <button class="button" type="submit">Set title</button>
</form>
<form style="margin-top: 24px" on:submit|preventDefault={openUrl}>
  <input id="url" bind:value={urlValue} />
  <button class="button" id="open-url">
    Open URL
  </button>
</form>