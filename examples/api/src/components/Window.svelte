<script>
  import { appWindow, WebviewWindow, LogicalSize, UserAttentionType, PhysicalSize, PhysicalPosition } from "@tauri-apps/api/window";
  import { open as openDialog } from "@tauri-apps/api/dialog";
  import { open } from "@tauri-apps/api/shell";

  let selectedWindow = appWindow.label;
  const windowMap = {
    [selectedWindow]: appWindow
  }

  export let onMessage;

  let urlValue = "https://tauri.studio";
  let resizable = true;
  let maximized = false;
  let transparent = false;
  let decorations = true;
  let alwaysOnTop = false;
  let fullscreen = false;
  let width = 900;
  let height = 700;
  let minWidth = 600;
  let minHeight = 600;
  let maxWidth = null;
  let maxHeight = null;
  let x = 100;
  let y = 100;
  let scaleFactor = 1;
  let innerPosition = new PhysicalPosition(x, y);
  let outerPosition = new PhysicalPosition(x, y);
  let innerSize = new PhysicalSize(width, height);
  let outerSize = new PhysicalSize(width, height);
  let resizeEventUnlisten;
  let moveEventUnlisten;

  let windowTitle = "Awesome Tauri Example!";

  function openUrl() {
    open(urlValue);
  }

  function setTitle_() {
    windowMap[selectedWindow].setTitle(windowTitle);
  }

  function hide_() {
    windowMap[selectedWindow].hide();
    setTimeout(windowMap[selectedWindow].show, 2000);
  }

  function minimize_() {
    windowMap[selectedWindow].minimize();
    setTimeout(windowMap[selectedWindow].unminimize, 2000);
  }

  function getIcon() {
    openDialog({
      multiple: false,
    }).then(path => {
      if (typeof path === 'string') {
        windowMap[selectedWindow].setIcon(path)
      }
    });
  }

  function createWindow() {
    const label = Math.random().toString().replace('.', '');
    const webview = new WebviewWindow(label);
    windowMap[label] = webview;
    webview.once('tauri://error', function () {
      onMessage("Error creating new webview")
    })
  }

  function handleWindowResize() {
    windowMap[selectedWindow].innerSize().then(response => {
      innerSize = response
      width = innerSize.width
      height = innerSize.height
    });
    windowMap[selectedWindow].outerSize().then(response => {
      outerSize = response
    });
  }

  function handleWindowMove() {
    windowMap[selectedWindow].innerPosition().then(response => {
      innerPosition = response
    });
    windowMap[selectedWindow].outerPosition().then(response => {
      outerPosition = response
      x = outerPosition.x
      y = outerPosition.y
    });
  }

  async function addWindowEventListeners(window) {
    if (resizeEventUnlisten) {
      resizeEventUnlisten();
    }
    if(moveEventUnlisten) {
      moveEventUnlisten();
    }
    moveEventUnlisten = await window.listen('tauri://move', handleWindowMove);
    resizeEventUnlisten = await window.listen('tauri://resize', handleWindowResize);
  }

  async function requestUserAttention_() {
    await windowMap[selectedWindow].minimize();
    await windowMap[selectedWindow].requestUserAttention(UserAttentionType.Critical);
    await new Promise(resolve => setTimeout(resolve, 3000));
    await windowMap[selectedWindow].requestUserAttention(null);
  }

  $: windowMap[selectedWindow].setResizable(resizable);
  $: maximized ? windowMap[selectedWindow].maximize() : windowMap[selectedWindow].unmaximize();
  $: windowMap[selectedWindow].setDecorations(decorations);
  $: windowMap[selectedWindow].setAlwaysOnTop(alwaysOnTop);
  $: windowMap[selectedWindow].setFullscreen(fullscreen);

  $: windowMap[selectedWindow].setSize(new PhysicalSize(width, height));
  $: minWidth && minHeight ? windowMap[selectedWindow].setMinSize(new LogicalSize(minWidth, minHeight)) : windowMap[selectedWindow].setMinSize(null);
  $: maxWidth && maxHeight ? windowMap[selectedWindow].setMaxSize(new LogicalSize(maxWidth, maxHeight)) : windowMap[selectedWindow].setMaxSize(null);
  $: windowMap[selectedWindow].setPosition(new PhysicalPosition(x, y));
  $: windowMap[selectedWindow].scaleFactor().then(factor => scaleFactor = factor);
  $: addWindowEventListeners(windowMap[selectedWindow]);
</script>

<div class="flex col">
  <select class="button" bind:value={selectedWindow}>
      {#each Object.keys(windowMap) as label}
        <option value={label}>{label}</option>
      {/each}
  </select>
  <div>
    <label>
      <input type="checkbox" bind:checked={resizable} />
      Resizable
    </label>
    <label>
      <input type="checkbox" bind:checked={maximized} />
      Maximize
    </label>
    <button title="Unminimizes after 2 seconds" on:click={() => windowMap[selectedWindow].center()}>
      Center
    </button>
    <button title="Unminimizes after 2 seconds" on:click={minimize_}>
      Minimize
    </button>
    <button title="Visible again after 2 seconds" on:click={hide_}>
      Hide
    </button>
    <label>
      <input type="checkbox" bind:checked={transparent} />
      Transparent
    </label>
    <label>
      <input type="checkbox" bind:checked={decorations} />
      Has decorations
    </label>
    <label>
      <input type="checkbox" bind:checked={alwaysOnTop} />
      Always on top
    </label>
    <label>
      <input type="checkbox" bind:checked={fullscreen} />
      Fullscreen
    </label>
    <button on:click={getIcon}> Change icon </button>
  </div>
  <div>
    <div class="window-controls flex flex-row">
      <div class="flex col grow">
        <div>
          X
          <input type="number" bind:value={x} min="0" />
        </div>
        <div>
          Y
          <input type="number" bind:value={y} min="0" />
        </div>
      </div>

      <div class="flex col grow">
        <div>
          Width
          <input type="number" bind:value={width} min="400" />
        </div>
        <div>
          Height
          <input type="number" bind:value={height} min="400" />
        </div>
      </div>

      <div class="flex col grow">
        <div>
          Min width
          <input type="number" bind:value={minWidth} />
        </div>
        <div>
          Min height
          <input type="number" bind:value={minHeight} />
        </div>
      </div>

      <div class="flex col grow">
        <div>
          Max width
          <input type="number" bind:value={maxWidth} min="400" />
        </div>
        <div>
          Max height
          <input type="number" bind:value={maxHeight} min="400" />
        </div>
      </div>
    </div>
  </div>
</div>
<div>
  <div class="flex">
    <div class="grow window-property">
      <div>Inner Size</div>
      <span>Width: {innerSize.width}</span>
      <span>Height: {innerSize.height}</span>
    </div>
    <div class="grow window-property">
      <div>Outer Size</div>
      <span>Width: {outerSize.width}</span>
      <span>Height: {outerSize.height}</span>
    </div>
  </div>
  <div class="flex">
    <div class="grow window-property">
      <div>Inner Logical Size</div>
      <span>Width: {innerSize.toLogical(scaleFactor).width}</span>
      <span>Height: {innerSize.toLogical(scaleFactor).height}</span>
    </div>
    <div class="grow window-property">
      <div>Outer Logical Size</div>
      <span>Width: {outerSize.toLogical(scaleFactor).width}</span>
      <span>Height: {outerSize.toLogical(scaleFactor).height}</span>
    </div>
  </div>
  <div class="flex">
    <div class="grow window-property">
      <div>Inner Position</div>
      <span>x: {innerPosition.x}</span>
      <span>y: {innerPosition.y}</span>
    </div>
    <div class="grow window-property">
      <div>Outer Position</div>
      <span>x: {outerPosition.x}</span>
      <span>y: {outerPosition.y}</span>
    </div>
  </div>
  <div class="flex">
    <div class="grow window-property">
      <div>Inner Logical Position</div>
      <span>x: {innerPosition.toLogical(scaleFactor).x}</span>
      <span>y: {innerPosition.toLogical(scaleFactor).y}</span>
    </div>
    <div class="grow window-property">
      <div>Outer Logical Position</div>
      <span>x: {outerPosition.toLogical(scaleFactor).x}</span>
      <span>y: {outerPosition.toLogical(scaleFactor).y}</span>
    </div>
  </div>
</div>
<form on:submit|preventDefault={setTitle_}>
  <input id="title" bind:value={windowTitle} />
  <button class="button" type="submit">Set title</button>
</form>
<form on:submit|preventDefault={openUrl}>
  <input id="url" bind:value={urlValue} />
  <button class="button" id="open-url"> Open URL </button>
</form>
<button class="button" on:click={requestUserAttention_} title="Minimizes the window, requests attention for 3s and then resets it">Request attention</button>
<button class="button" on:click={createWindow}>New window</button>

<style>
  form {
    margin-top: 24px;
  }

  .flex-row {
    flex-direction: row;
  }

  .grow {
    flex-grow: 1;
  }

  .window-controls input {
    width: 50px;
  }

  .window-property {
    margin-top: 12px;
  }
  .window-property span {
    font-size: 0.8rem;
  }
</style>
