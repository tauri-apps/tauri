<script>
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/api/window"

  import Cli from './components/Cli.svelte'
  import Communication from './components/Communication.svelte'
  import Dialog from './components/Dialog.svelte'
  import FileSystem from './components/FileSystem.svelte'
  import Http from './components/Http.svelte'
  import Notifications from './components/Notifications.svelte'
  import Window from './components/Window.svelte'
  import Shortcuts from './components/Shortcuts.svelte'

  const views = [{
    label: 'Messages',
    component: Communication
  }, {
    label: 'CLI',
    component: Cli
  }, {
    label: 'Dialog',
    component: Dialog
  }, {
    label: 'File system',
    component: FileSystem
  }, {
    label: 'HTTP',
    component: Http
  }, {
    label: 'Notifications',
    component: Notifications
  }, {
    label: 'Window',
    component: Window
  }, {
    label: 'Shortcuts',
    component: Shortcuts
  }]

  let selected = views[0].label;

  let response = '';

  function select(view) {
    selected = view.label
  }

  function onMessage(value) {
    response = typeof value === "string" ? value : JSON.stringify(value);
  }

  function onLogoClick() {
    open("https://tauri.studio/");
  }
</script>

<main>
  <div class="logo-container">
    <img src="icon.png" class="logo" on:click={onLogoClick} alt="logo" />
  </div>

  <div class="tabs-container">
    <div class="tabs">
      {#each views as view}
      <div class="tab">
        <input id={`tab-${view.label}`} type="radio" checked={view.label===selected} />
        <label for={`tab-${view.label}`} class="tabber" on:click={()=> select(view)}>{view.label}</label>
        <div class="content">
          <svelte:component this={view.component} {onMessage} />
        </div>
      </div>
      {/each}
    </div>
  </div>
  <div id="response">{@html response}</div>
  <div class="bottom">
    <a class="dark-link" target="_blank" href="https://tauri.studio">
      Tauri Documentation
    </a>
    &nbsp;&nbsp;&nbsp;
    <a class="dark-link" target="_blank" href="https://github.com/tauri-apps/tauri">
      Github Repo
    </a>
    &nbsp;&nbsp;&nbsp;
    <a class="dark-link" target="_blank"
      href="https://github.com/tauri-apps/tauri/tree/dev/tauri/examples/communication">
      Source for this App
    </a>
  </div>
</main>