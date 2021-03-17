<script>
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/api/shell";

  import Cli from "./components/Cli.svelte";
  import Communication from "./components/Communication.svelte";
  import Dialog from "./components/Dialog.svelte";
  import FileSystem from "./components/FileSystem.svelte";
  import Http from "./components/Http.svelte";
  import Notifications from "./components/Notifications.svelte";
  import Window from "./components/Window.svelte";
  import Shortcuts from "./components/Shortcuts.svelte";
  import Welcome from "./components/Welcome.svelte";
  import Updater from "./components/Updater.svelte";

  const views = [
    {
      label: "Welcome",
      component: Welcome,
    },
    {
      label: "Messages",
      component: Communication,
    },
    {
      label: "CLI",
      component: Cli,
    },
    {
      label: "Dialog",
      component: Dialog,
    },
    {
      label: "File system",
      component: FileSystem,
    },
    {
      label: "HTTP",
      component: Http,
    },
    {
      label: "Notifications",
      component: Notifications,
    },
    {
      label: "Window",
      component: Window,
    },
    {
      label: "Shortcuts",
      component: Shortcuts,
    },
    {
      label: "Updater",
      component: Updater,
    },    
  ];

  let selected = views[0];

  let responses = [""];

  function select(view) {
    selected = view;
  }

  function onMessage(value) {
    responses += typeof value === "string" ? value : JSON.stringify(value);
    responses += "\n";
  }

  function onLogoClick() {
    open("https://tauri.studio/");
  }
</script>

<main>
  <div class="flex row noselect just-around" style="margin=1em;">
    <img src="tauri.png" height="60" on:click={onLogoClick} alt="logo" />
    <div>
      <a class="dark-link" target="_blank" href="https://tauri.studio/en/docs/getting-started/intro">
        Documentation
      </a>
      <a class="dark-link" target="_blank" href="https://github.com/tauri-apps/tauri">
        Github
      </a>
      <a class="dark-link" target="_blank" href="https://github.com/tauri-apps/tauri/tree/dev/tauri/examples/api">
        Source
      </a>
    </div>
  </div>
  <div class="flex row">
    <div style="width:15em; margin-left:0.5em">
      {#each views as view}
      <p class="nv noselect {selected === view ? 'nv_selected' : ''}" on:click={()=> select(view)}
        >
        {view.label}
      </p>
      {/each}
    </div>
    <div class="content">
      <svelte:component this={selected.component} {onMessage} />
    </div>
  </div>
  <div id="response">
    <p class="flex row just-around">
      <strong>Tauri Console</strong>
      <a class="nv" on:click={()=> {
        responses = [""];
        }}>clear</a>
    </p>
    {responses}
  </div>
</main>
