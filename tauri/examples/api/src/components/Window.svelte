<script>
  import Slider from 'svelte-slider';
  import debounce from '../debounce'
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
  let width = null
  let height = null
  let minWidth = null
  let minHeight = null
  let maxWidth = null
  let maxHeight = null
  let x = null
  let y = null

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

  const updateWidth = debounce(() => setWidth(width * 1200 + 300), 100)
  const updateHeight = debounce(() => setHeight(height * 1200 + 300), 100)
  const updateMinSize = debounce(() => setMinSize(minWidth * 1200 + 300, minHeight * 1200 + 300), 100)
  const updateMaxSize = debounce(() => setMaxSize(maxWidth * 1200 + 300, maxHeight * 1200 + 300), 100)
  const updateX = debounce(() => setX(x * 1200), 100)
  const updateY = debounce(() => setY(y * 1200), 100)
</script>

<style>
  .slider {
    --sliderPrimary: #FF9800;
    --sliderSecondary: rgba(0, 0, 0, 0.05);
    margin: 16px;
  }

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
    <div class="flex flex-row">
      <div class="flex flex-column grow">
        <div class="slider">
          X
          <Slider on:change={(event)=> (x = event.detail[1]) && updateX()} value={[0, 1]} single />
        </div>
        <div class="slider">
          Y
          <Slider on:change={(event)=> (y = event.detail[1]) && updateY()} value={[0, 1]} single />
        </div>
      </div>

      <div class="flex flex-column grow">
        <div class="slider">
          Width
          <Slider on:change={(event)=> (width = event.detail[1]) && updateWidth()} value={[0, 1]} single />
        </div>
        <div class="slider">
          Height
          <Slider on:change={(event)=> (height = event.detail[1]) && updateHeight()} value={[0, 1]} single />
        </div>
      </div>

      <div class="flex flex-column grow">
        <div class="slider">
          Min width
          <Slider on:change={(event)=> (minWidth = event.detail[1]) && updateMinSize()} value={[0, 1]} single />
        </div>
        <div class="slider">
          Min height
          <Slider on:change={(event)=> (minHeight = event.detail[1]) && updateMinSize()} value={[0, 1]} single />
        </div>
      </div>

      <div class="flex flex-column grow">
        <div class="slider">
          Max width
          <Slider on:change={(event)=> (maxWidth = event.detail[1]) && updateMaxSize()} value={[0, 1]} single />
        </div>
        <div class="slider">
          Max height
          <Slider on:change={(event)=> (maxHeight = event.detail[1]) && updateMaxSize()} value={[0, 1]} single />
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