<script>
  import { writable } from 'svelte/store'
  import { open } from '@tauri-apps/api/shell'
  import { appWindow, getCurrent } from '@tauri-apps/api/window'
  import * as os from '@tauri-apps/api/os'

  import Welcome from './views/Welcome.svelte'
  import Cli from './views/Cli.svelte'
  import Communication from './views/Communication.svelte'
  import Dialog from './views/Dialog.svelte'
  import FileSystem from './views/FileSystem.svelte'
  import Http from './views/Http.svelte'
  import Notifications from './views/Notifications.svelte'
  import Window from './views/Window.svelte'
  import Shortcuts from './views/Shortcuts.svelte'
  import Shell from './views/Shell.svelte'
  import Updater from './views/Updater.svelte'
  import Clipboard from './views/Clipboard.svelte'
  import WebRTC from './views/WebRTC.svelte'

  import { onMount } from 'svelte'
  import { listen } from '@tauri-apps/api/event'
  import { ask } from '@tauri-apps/api/dialog'

  appWindow.listen('tauri://file-drop', function (event) {
    onMessage(`File drop: ${event.payload}`)
  })

  const views = [
    {
      label: 'Welcome',
      component: Welcome,
      icon: 'i-ph-hand-waving'
    },
    {
      label: 'Communication',
      component: Communication,
      icon: 'i-codicon-radio-tower'
    },
    {
      label: 'CLI',
      component: Cli,
      icon: 'i-codicon-terminal'
    },
    {
      label: 'Dialog',
      component: Dialog,
      icon: 'i-codicon-multiple-windows'
    },
    {
      label: 'File system',
      component: FileSystem,
      icon: 'i-codicon-files'
    },
    {
      label: 'HTTP',
      component: Http,
      icon: 'i-ph-globe-hemisphere-west'
    },
    {
      label: 'Notifications',
      component: Notifications,
      icon: 'i-codicon-bell-dot'
    },
    {
      label: 'Window',
      component: Window,
      icon: 'i-codicon-window'
    },
    {
      label: 'Shortcuts',
      component: Shortcuts,
      icon: 'i-codicon-record-keys'
    },
    {
      label: 'Shell',
      component: Shell,
      icon: 'i-codicon-terminal-bash'
    },
    {
      label: 'Updater',
      component: Updater,
      icon: 'i-codicon-cloud-download'
    },
    {
      label: 'Clipboard',
      component: Clipboard,
      icon: 'i-codicon-clippy'
    },
    {
      label: 'WebRTC',
      component: WebRTC,
      icon: 'i-ph-broadcast'
    }
  ]

  let selected = views[0]
  function select(view) {
    selected = view
  }

  // Window controls
  let isWindowMaximized
  onMount(async () => {
    const window = getCurrent()
    isWindowMaximized = await window.isMaximized()
    listen('tauri://resize', async () => {
      isWindowMaximized = await window.isMaximized()
    })
  })

  function minimize() {
    getCurrent().minimize()
  }

  async function toggleMaximize() {
    const window = getCurrent()
    ;(await window.isMaximized()) ? window.unmaximize() : window.maximize()
  }

  let confirmed_close = false
  async function close() {
    if (!confirmed_close) {
      confirmed_close = await ask(
        'Are you sure that you want to close this window?',
        {
          title: 'Tauri API'
        }
      )
      if (confirmed_close) {
        getCurrent().close()
      }
    }
  }

  // dark/light
  let isDark
  onMount(() => {
    isDark = localStorage.getItem('theme') == 'dark'
    applyTheme(isDark)
  })
  function applyTheme(isDark) {
    const html = document.querySelector('html')
    isDark ? html.classList.add('dark') : html.classList.remove('dark')
    localStorage.setItem('theme', isDark ? 'dark' : '')
  }
  function toggleDark() {
    isDark = !isDark
    applyTheme(isDark)
  }

  // Console
  let messages = writable([])
  function onMessage(value) {
    messages.update((r) => [
      {
        html:
          `<pre><strong class="text-accent dark:text-darkAccent">[${new Date().toLocaleTimeString()}]:</strong> ` +
          (typeof value === 'string' ? value : JSON.stringify(value, null, 1)) +
          '</pre>'
      },
      ...r
    ])
  }

  // this function is renders HTML without sanitizing it so it's insecure
  // we only use it with our own input data
  function insecureRenderHtml(html) {
    messages.update((r) => [
      {
        html:
          `<pre><strong class="text-accent dark:text-darkAccent">[${new Date().toLocaleTimeString()}]:</strong> ` +
          html +
          '</pre>'
      },
      ...r
    ])
  }

  function clear() {
    messages.update(() => [])
  }

  let consoleEl, consoleH, cStartY
  let minConsoleHeight = 50
  function startResizingConsole(e) {
    cStartY = e.clientY

    const styles = window.getComputedStyle(consoleEl)
    consoleH = parseInt(styles.height, 10)

    const moveHandler = (e) => {
      const dy = e.clientY - cStartY
      const newH = consoleH - dy
      consoleEl.style.height = `${
        newH < minConsoleHeight ? minConsoleHeight : newH
      }px`
    }
    const upHandler = () => {
      document.removeEventListener('mouseup', upHandler)
      document.removeEventListener('mousemove', moveHandler)
    }
    document.addEventListener('mouseup', upHandler)
    document.addEventListener('mousemove', moveHandler)
  }

  let isWindows
  onMount(async () => {
    isWindows = (await os.platform()) === 'win32'
  })
</script>

{#if isWindows}
  <div
    class="w-screen select-none h-8 pl-2 flex justify-between items-center absolute text-primaryText dark:text-darkPrimaryText"
    data-tauri-drag-region
  >
    <span class="text-darkPrimaryText">Tauri API Validation</span>
    <span
      class="
      h-100%
      children:h-100% children:w-12 children:inline-flex
      children:items-center children:justify-center"
    >
      <span
        title={isDark ? 'Switch to Light mode' : 'Switch to Dark mode'}
        class="hover:bg-hoverOverlay  dark:hover:bg-darkHoverOverlay"
        on:click={toggleDark}
      >
        {#if isDark}
          <div class="i-ph-sun" />
        {:else}
          <div class="i-ph-moon" />
        {/if}
      </span>
      <span
        title="Minimize"
        class="hover:bg-hoverOverlay  dark:hover:bg-darkHoverOverlay"
        on:click={minimize}
      >
        <div class="i-codicon-chrome-minimize" />
      </span>
      <span
        title={isWindowMaximized ? 'Restore' : 'Maximize'}
        class="hover:bg-hoverOverlay  dark:hover:bg-darkHoverOverlay"
        on:click={toggleMaximize}
      >
        {#if isWindowMaximized}
          <div class="i-codicon-chrome-restore" />
        {:else}
          <div class="i-codicon-chrome-maximize" />
        {/if}
      </span>
      <span
        title="Close"
        class="hover:bg-red-700 dark:hover:bg-red-700 hover:text-darkPrimaryText"
        on:click={close}
      >
        <div class="i-codicon-chrome-close" />
      </span>
    </span>
  </div>
{/if}

<div
  class="flex h-screen w-screen overflow-hidden children-pt8 children-pb-2 text-primaryText dark:text-darkPrimaryText"
>
  <aside
    class="w-75 {isWindows
      ? 'bg-darkPrimaryLighter/60'
      : 'bg-darkPrimaryLighter'} transition-colors-250 overflow-hidden grid select-none px-2"
  >
    <img
      on:click={() => open('https://tauri.app/')}
      class="self-center p-7 cursor-pointer"
      src="tauri_logo.png"
      alt="Tauri logo"
    />
    {#if !isWindows}
      <a href="##" class="nv justify-between h-8" on:click={toggleDark}>
        {#if isDark}
          Switch to Light mode
          <div class="i-ph-sun" />
        {:else}
          Switch to Dark mode
          <div class="i-ph-moon" />
        {/if}
      </a>
      <br />
      <div class="bg-white/5 h-2px" />
      <br />
    {/if}

    <a
      class="nv justify-between h-8"
      target="_blank"
      href="https://tauri.app/v1/guides/"
    >
      Documentation
      <span class="i-codicon-link-external" />
    </a>
    <a
      class="nv justify-between h-8"
      target="_blank"
      href="https://github.com/tauri-apps/tauri"
    >
      Github
      <span class="i-codicon-link-external" />
    </a>
    <a
      class="nv justify-between h-8"
      target="_blank"
      href="https://github.com/tauri-apps/tauri/tree/dev/examples/api"
    >
      Source
      <span class="i-codicon-link-external" />
    </a>
    <br />
    <div class="bg-white/5 h-2px" />
    <br />
    <div
      class="flex flex-col overflow-y-auto children-h-10 children-flex-none gap-1"
    >
      {#each views as view}
        <a
          href="##"
          class="mr-1 nv {selected === view ? 'nv_selected' : ''}"
          on:click={() => select(view)}
        >
          <div class="{view.icon} mr-2" />
          <p>{view.label}</p></a
        >
      {/each}
    </div>
  </aside>
  <main
    class="flex-1 bg-primary dark:bg-darkPrimary transition-colors-250 grid grid-rows-[2fr_auto]"
  >
    <div class="px-5 overflow-hidden grid grid-rows-[auto_1fr]">
      <h1>{selected.label}</h1>
      <div class="overflow-y-auto">
        <div class="mr-2">
          <svelte:component
            this={selected.component}
            {onMessage}
            {insecureRenderHtml}
          />
        </div>
      </div>
    </div>

    <div
      bind:this={consoleEl}
      id="console"
      class="select-none h-15rem grid grid-rows-[2px_2rem_1fr] gap-1 overflow-hidden"
    >
      <div
        on:mousedown={startResizingConsole}
        class="bg-black/20 h-2px cursor-ns-resize"
      />
      <div class="flex justify-between items-center px-2">
        <p class="font-semibold">Console</p>
        <div
          class="cursor-pointer h-85% rd-1 p-1 flex justify-center items-center
                hover:bg-hoverOverlay dark:hover:bg-darkHoverOverlay
                active:bg-hoverOverlay/25 dark:active:bg-darkHoverOverlay/25
          "
          on:click={clear}
        >
          <div class="i-codicon-clear-all" />
        </div>
      </div>
      <div class="px-2 overflow-y-auto all:font-mono">
        {#each $messages as r}
          {@html r.html}
        {/each}
      </div>
    </div>
  </main>
</div>
