<script module lang="ts">
  export type ViewProps = {
    onMessage: (value: unknown) => void
  }
</script>

<script lang="ts">
  import { onMount, tick } from 'svelte'
  import { writable } from 'svelte/store'
  import { invoke } from '@tauri-apps/api/core'
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
  import { setTheme } from '@tauri-apps/api/app'

  import Welcome from './views/Welcome.svelte'
  import Communication from './views/Communication.svelte'
  import Window from './views/Window.svelte'
  import WebRTC from './views/WebRTC.svelte'
  import App from './views/App.svelte'
  import Menu from './views/Menu.svelte'
  import Tray from './views/Tray.svelte'
  import type { Theme } from '@tauri-apps/api/window'
  import { MediaQuery } from 'svelte/reactivity'

  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey && event.key === 'b') {
      invoke('plugin:app-menu|toggle')
    }
  })

  const appWindow = getCurrentWebviewWindow()
  appWindow.onDragDropEvent((event) => {
    onMessage(event.payload)
  })

  const userAgent = navigator.userAgent.toLowerCase()
  const isMobile = userAgent.includes('android') || userAgent.includes('iphone')

  const desktopViews = [
    {
      label: 'App',
      component: App,
      icon: 'i-codicon-hubot'
    },
    {
      label: 'Window',
      component: Window,
      icon: 'i-codicon-window'
    },
    {
      label: 'Menu',
      component: Menu,
      icon: 'i-ph-list'
    },
    {
      label: 'Tray',
      component: Tray,
      icon: 'i-ph-tray'
    }
  ]

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
    ...(isMobile ? [] : desktopViews),
    {
      label: 'WebRTC',
      component: WebRTC,
      icon: 'i-ph-broadcast'
    }
  ]

  let selected = $state.raw(views[0])

  // dark/light themes
  const preferDark = new MediaQuery('prefers-color-scheme: dark')
  let theme = $state<Theme | 'auto'>(
    (localStorage.getItem('theme') as Theme | null) || 'auto'
  )

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
    applyTheme()
  }

  function applyTheme() {
    const isDark = theme === 'auto' ? preferDark.current : theme === 'dark'
    isDark
      ? document.documentElement.classList.add('dark')
      : document.documentElement.classList.remove('dark')
    setTheme(theme === 'auto' ? null : theme)
    localStorage.setItem('theme', theme)
  }

  $effect(() => {
    applyTheme()
  })

  // Console
  const messages = writable<string[]>([])
  let consoleTextEl: HTMLDivElement

  // this function is renders HTML without sanitizing it so it's insecure
  // we only use it with our own input data
  async function insecureRenderHtml(html: string) {
    messages.update((r) => [
      ...r,
      `<pre><strong class="text-accent dark:text-darkAccent">[${new Date().toLocaleTimeString()}]:</strong> ${html}</pre>`
    ])
    await tick()
    consoleTextEl.scrollTop = consoleTextEl.scrollHeight
  }

  async function onMessage(value: unknown) {
    const valueStr =
      typeof value === 'string'
        ? value
        : JSON.stringify(
            value instanceof ArrayBuffer
              ? Array.from(new Uint8Array(value))
              : value,
            null,
            1
          )
    insecureRenderHtml(valueStr)
  }

  function clear() {
    messages.update(() => [])
  }

  let consoleEl: HTMLDivElement
  let consoleH = 0
  let cStartY = 0
  const minConsoleHeight = 50
  function startResizingConsole(e: MouseEvent) {
    cStartY = e.clientY

    const styles = window.getComputedStyle(consoleEl)
    consoleH = parseInt(styles.height, 10)

    const moveHandler = (e: MouseEvent) => {
      const dy = e.clientY - cStartY
      const newH = consoleH - dy
      consoleEl.style.height = `${newH < minConsoleHeight ? minConsoleHeight : newH}px`
    }
    const upHandler = () => {
      document.removeEventListener('mouseup', upHandler)
      document.removeEventListener('mousemove', moveHandler)
    }
    document.addEventListener('mouseup', upHandler)
    document.addEventListener('mousemove', moveHandler)
  }

  // mobile
  let isSideBarOpen = $state(false)
  let sidebar: HTMLElement
  let sidebarToggle: HTMLElement
  let isDraggingSideBar = false
  let draggingStartPosX = 0
  let draggingEndPosX = 0
  const clamp = (min: number, num: number, max: number) =>
    Math.min(Math.max(num, min), max)

  function toggleSidebar() {
    sidebar.style.setProperty(
      '--translate-x',
      `${isSideBarOpen ? '0' : '-18.75'}rem`
    )
  }

  onMount(() => {
    document.addEventListener('click', (e) => {
      if (!(e.target instanceof Node)) return

      if (sidebarToggle.contains(e.target)) {
        isSideBarOpen = !isSideBarOpen
      } else if (isSideBarOpen && !sidebar.contains(e.target)) {
        isSideBarOpen = false
      }
    })

    document.addEventListener('touchstart', (e) => {
      if (!(e.target instanceof Node)) return
      if (sidebarToggle.contains(e.target)) return

      const x = e.touches[0].clientX
      if ((0 < x && x < 20 && !isSideBarOpen) || isSideBarOpen) {
        isDraggingSideBar = true
        draggingStartPosX = x
      }
    })

    document.addEventListener('touchmove', (e) => {
      if (isDraggingSideBar) {
        const x = e.touches[0].clientX
        draggingEndPosX = x
        const delta = (x - draggingStartPosX) / 10
        sidebar.style.setProperty(
          '--translate-x',
          `-${clamp(0, isSideBarOpen ? 0 - delta : 18.75 - delta, 18.75)}rem`
        )
      }
    })

    document.addEventListener('touchend', () => {
      if (isDraggingSideBar) {
        const delta = (draggingEndPosX - draggingStartPosX) / 10
        isSideBarOpen = isSideBarOpen ? delta > -(18.75 / 2) : delta > 18.75 / 2
      }

      isDraggingSideBar = false
    })
  })

  $effect(() => {
    toggleSidebar()
  })
</script>

<!-- Sidebar toggle, only visible on small screens -->
<div
  id="sidebarToggle"
  bind:this={sidebarToggle}
  class="z-2000 hidden lt-sm:flex justify-center items-center absolute top-2 left-2 w-8 h-8 rd-8
            bg-accent dark:bg-darkAccent active:bg-accentDark dark:active:bg-darkAccentDark"
>
  {#if isSideBarOpen}
    <span class="i-codicon-close animate-duration-300ms animate-fade-in"></span>
  {:else}
    <span class="i-codicon-menu animate-duration-300ms animate-fade-in"></span>
  {/if}
</div>

<div
  class="flex h-screen w-screen overflow-hidden children-pt4 children-pb-2 text-primaryText dark:text-darkPrimaryText"
>
  <aside
    id="sidebar"
    bind:this={sidebar}
    class="lt-sm:h-screen lt-sm:shadow-lg lt-sm:shadow lt-sm:transition-transform lt-sm:absolute lt-sm:z-1999
      bg-darkPrimaryLighter transition-colors-250 overflow-hidden grid grid-rows-[min-content_auto] select-none px-2"
  >
    <img
      class="self-center p-7 cursor-pointer"
      src="tauri_logo.png"
      alt="Tauri logo"
    />
    <a href="##" class="nv justify-between" onclick={switchTheme}>
      {#if theme === 'auto'}
        Switch to Dark mode
        <div class="i-ph-circle-half-fill"></div>
      {:else if theme === 'dark'}
        Switch to Light mode
        <div class="i-ph-moon"></div>
      {:else if theme === 'light'}
        Switch to Auto mode
        <div class="i-ph-sun"></div>
      {/if}
    </a>
    <br />
    <div class="bg-white/5 h-2px"></div>
    <br />

    <a
      class="nv justify-between"
      target="_blank"
      href="https://v2.tauri.app/start/"
    >
      Documentation
      <span class="i-codicon-link-external"></span>
    </a>
    <a
      class="nv justify-between"
      target="_blank"
      href="https://github.com/tauri-apps/tauri"
    >
      GitHub
      <span class="i-codicon-link-external"></span>
    </a>
    <a
      class="nv justify-between"
      target="_blank"
      href="https://github.com/tauri-apps/tauri/tree/dev/examples/api"
    >
      Source
      <span class="i-codicon-link-external"></span>
    </a>
    <br />
    <div class="bg-white/5 h-2px"></div>
    <br />
    <div class="flex flex-col overflow-y-auto children-flex-none gap-1">
      {#each views as view}
        <a
          href="##"
          class="nv {selected === view ? 'nv_selected' : ''}"
          onclick={() => {
            selected = view
            isSideBarOpen = false
          }}
        >
          <div class="{view.icon} mr-2"></div>
          <p>{view.label}</p>
        </a>
      {/each}
    </div>
  </aside>
  <main
    class="flex-1 bg-primary dark:bg-darkPrimary transition-transform transition-colors-250 grid grid-rows-[2fr_auto]"
  >
    <div class="px-5 overflow-hidden grid grid-rows-[auto_1fr]">
      <h1>{selected.label}</h1>
      <div class="overflow-y-auto">
        <div class="mr-2">
          <selected.component {onMessage} />
        </div>
      </div>
    </div>

    <div
      bind:this={consoleEl}
      id="console"
      class="select-none h-15rem grid grid-rows-[2px_2rem_1fr] gap-1 overflow-hidden"
    >
      <div
        role="button"
        tabindex="0"
        onmousedown={startResizingConsole}
        class="bg-black/20 h-4px cursor-ns-resize"
      ></div>
      <div class="flex justify-between items-center px-2">
        <p class="font-semibold">Console</p>
        <div
          role="button"
          tabindex="0"
          class="cursor-pointer h-85% rd-1 p-1 flex justify-center items-center
                hover:bg-hoverOverlay dark:hover:bg-darkHoverOverlay
                active:bg-hoverOverlay/25 dark:active:bg-darkHoverOverlay/25
          "
          onkeypress={(e) => (e.key === 'Enter' ? clear() : {})}
          onclick={clear}
        >
          <div class="i-codicon-clear-all"></div>
        </div>
      </div>
      <div
        bind:this={consoleTextEl}
        class="px-2 overflow-y-auto all:font-mono code-block all:text-xs select-text mr-2"
      >
        {#each $messages as messageHtml}
          {@html messageHtml}
        {/each}
      </div>
    </div>
  </main>
</div>
