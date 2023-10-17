<script>
  import { onMount } from 'svelte'
  import { writable } from 'svelte/store'
  import { invoke } from '@tauri-apps/api/tauri'

  import Welcome from './views/Welcome.svelte'
  import Communication from './views/Communication.svelte'
  import Window from './views/Window.svelte'
  import WebRTC from './views/WebRTC.svelte'

  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey && event.key === 'b') {
      invoke('toggle_menu')
    }
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
      label: 'Window',
      component: Window,
      icon: 'i-codicon-window'
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

  // dark/light
  let isDark
  onMount(() => {
    isDark = localStorage && localStorage.getItem('theme') == 'dark'
    applyTheme(isDark)
  })
  function applyTheme(isDark) {
    const html = document.querySelector('html')
    isDark ? html.classList.add('dark') : html.classList.remove('dark')
    localStorage && localStorage.setItem('theme', isDark ? 'dark' : '')
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

  // mobile
  let isSideBarOpen = false
  let sidebar
  let sidebarToggle
  let isDraggingSideBar = false
  let draggingStartPosX = 0
  let draggingEndPosX = 0
  const clamp = (min, num, max) => Math.min(Math.max(num, min), max)

  function toggleSidebar(sidebar, isSideBarOpen) {
    sidebar.style.setProperty(
      '--translate-x',
      `${isSideBarOpen ? '0' : '-18.75'}rem`
    )
  }

  onMount(() => {
    sidebar = document.querySelector('#sidebar')
    sidebarToggle = document.querySelector('#sidebarToggle')

    document.addEventListener('click', (e) => {
      if (sidebarToggle.contains(e.target)) {
        isSideBarOpen = !isSideBarOpen
      } else if (isSideBarOpen && !sidebar.contains(e.target)) {
        isSideBarOpen = false
      }
    })

    document.addEventListener('touchstart', (e) => {
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

  $: {
    const sidebar = document.querySelector('#sidebar')
    if (sidebar) {
      toggleSidebar(sidebar, isSideBarOpen)
    }
  }
</script>

<!-- Sidebar toggle, only visible on small screens -->
<div
  id="sidebarToggle"
  class="z-2000 display-none lt-sm:flex justify-center items-center absolute top-2 left-2 w-8 h-8 rd-8
            bg-accent dark:bg-darkAccent active:bg-accentDark dark:active:bg-darkAccentDark"
>
  {#if isSideBarOpen}
    <span class="i-codicon-close animate-duration-300ms animate-fade-in" />
  {:else}
    <span class="i-codicon-menu animate-duration-300ms animate-fade-in" />
  {/if}
</div>

<div
  class="flex h-screen w-screen overflow-hidden children-pt8 children-pb-2 text-primaryText dark:text-darkPrimaryText"
>
  <aside
    id="sidebar"
    class="lt-sm:h-screen lt-sm:shadow-lg lt-sm:shadow lt-sm:transition-transform lt-sm:absolute lt-sm:z-1999
      bg-darkPrimaryLighter transition-colors-250 overflow-hidden grid grid-rows-[min-content_auto] select-none px-2"
  >
    <img
      class="self-center p-7 cursor-pointer"
      src="tauri_logo.png"
      alt="Tauri logo"
    />
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
      GitHub
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
        {#if view}
          <a
            href="##"
            class="nv {selected === view ? 'nv_selected' : ''}"
            on:click={() => {
              select(view)
              isSideBarOpen = false
            }}
          >
            <div class="{view.icon} mr-2" />
            <p>{view.label}</p></a
          >
        {/if}
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
      <div class="px-2 overflow-y-auto all:font-mono code-block all:text-xs">
        {#each $messages as r}
          {@html r.html}
        {/each}
      </div>
    </div>
  </main>
</div>
