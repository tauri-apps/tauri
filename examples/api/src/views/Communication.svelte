<script>
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
  import { Channel, invoke } from '@tauri-apps/api/core'
  import { getVersion } from '@tauri-apps/api/app'
  import { onMount, onDestroy } from 'svelte'

  let { onMessage } = $props()
  let unlisten

  const webviewWindow = getCurrentWebviewWindow()

  onMount(async () => {
    unlisten = await webviewWindow.listen('rust-event', onMessage)
  })
  onDestroy(() => {
    if (unlisten) {
      unlisten()
    }
  })

  function log() {
    invoke('log_operation', {
      event: 'tauri-click',
      payload: 'this payload is optional because we used Option in Rust'
    })
  }

  function performRequest() {
    invoke('perform_request', {
      endpoint: 'dummy endpoint arg',
      body: {
        id: 5,
        name: 'test'
      }
    })
      .then(onMessage)
      .catch(onMessage)
  }

  function echo() {
    invoke('echo', {
      message: 'Tauri JSON request!'
    })
      .then(onMessage)
      .catch(onMessage)

    invoke('echo', [1, 2, 3]).then(onMessage).catch(onMessage)
  }

  function spam() {
    const channel = new Channel()
    channel.onmessage = onMessage
    invoke('spam', { channel })
  }

  function testEval() {
    console.log('Calling test_eval...')
    invoke('test_eval')
      .then(() => {
        console.log('test_eval succeeded')
        onMessage('✅ test_eval succeeded')
      })
      .catch((e) => {
        console.error('test_eval failed:', e)
        onMessage('❌ test_eval failed: ' + e)
      })
  }

  function testNavigate() {
    console.log('Calling test_navigate...')
    // 导航到一个测试页面，这里用一个简单的 URL
    invoke('test_navigate', { url: 'https://tauri.app/' })
      .then(() => console.log('test_navigate succeeded'))
      .catch((e) => {
        console.error('test_navigate failed:', e)
        onMessage('test_navigate error: ' + e)
      })
  }

  function testReload() {
    console.log('Calling test_reload...')
    invoke('test_reload')
      .then(() => console.log('test_reload succeeded'))
      .catch((e) => {
        console.error('test_reload failed:', e)
        onMessage('test_reload error: ' + e)
      })
  }

  // Test 1: Custom URI scheme
  function testCustomScheme() {
    console.log('Testing custom scheme: myapp://test')
    window.location.href = 'myapp://test/path?param=123'
  }

  // Test 2: Navigation intercept (we'll just navigate somewhere)
  function testNavigationIntercept() {
    console.log('Testing navigation intercept...')
    window.location.href = 'https://example.com'
  }

  // Test 3: Web resource request intercept (load an image)
  function testResourceIntercept() {
    console.log('Testing resource intercept...')
    const img = document.createElement('img')
    img.src = 'tauri://localhost/assets/images/tauri.svg'
    img.onload = () => onMessage('✅ Resource loaded, check console for intercept log')
    img.onerror = () => onMessage('❌ Resource load failed')
    document.body.appendChild(img)
  }

  // Test 4: Data isolation - create window A
  function createWindowA() {
    console.log('Creating isolated window A...')
    invoke('create_isolated_window', { windowId: 'window_a', dataSuffix: 'a' })
      .then(() => console.log('✅ Window A created (isolated data dir A)'))
      .catch((e) => console.error('❌ Error: ' + e))
  }

  // Test 4: Data isolation - create window B
  function createWindowB() {
    console.log('Creating isolated window B...')
    invoke('create_isolated_window', { windowId: 'window_b', dataSuffix: 'b' })
      .then(() => console.log('✅ Window B created (isolated data dir B)'))
      .catch((e) => console.error('❌ Error: ' + e))
  }

  // Test: Set a value to a variable (simpler test)
  function setLocalStorage() {
    try {
      const value = 'value_' + Date.now()
      // 先用一个全局变量测试
      window._test_value = value
      const msg = '✅ Set test value: ' + value
      console.log(msg)
      onMessage(msg)
    } catch (e) {
      const msg = '❌ Set error: ' + e
      console.error(msg)
      onMessage(msg)
    }
  }

  // Test: Get the value
  function getLocalStorage() {
    try {
      const value = window._test_value || '(not set yet)'
      const msg = '📦 Test value = ' + value
      console.log(msg)
      onMessage(msg)
    } catch (e) {
      const msg = '❌ Get error: ' + e
      console.error(msg)
      onMessage(msg)
    }
  }

  function emitEvent() {
    webviewWindow.emit('js-event', 'this is the payload string')
  }

  // Test: Create window with background throttling disabled
  function createWindowNoThrottle() {
    console.log('Creating window with background throttling disabled...')
    invoke('create_window_no_throttle', {
      windowId: 'window_no_throttle'
    })
      .then(() => {
        const msg = '✅ Window with background throttling disabled created'
        console.log(msg)
        onMessage(msg)
      })
      .catch((e) => {
        const msg = '❌ Error: ' + e
        console.error(msg)
        onMessage(msg)
      })
  }

  // Test: Trigger a download
  function testDownload() {
    console.log('Testing download...')

    // Create a test file and trigger download
    const content = 'Hello from Tauri Download Test!\nGenerated at: ' + new Date().toISOString()
    const blob = new Blob([content], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)

    const a = document.createElement('a')
    a.href = url
    a.download = 'test-download.txt'
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)

    const msg = '✅ Download triggered, check console/log for intercept events'
    console.log(msg)
    onMessage(msg)
  }

  // Test: Create window with custom User-Agent
  function createWindowWithCustomUA() {
    console.log('Creating window with custom User-Agent...')
    invoke('create_window_with_custom_ua', {
      windowId: 'window_custom_ua',
      userAgent: 'MyTauriApp/1.0 (CustomUserAgent/Test)'
    })
      .then(() => {
        const msg = '✅ Window with custom User-Agent created'
        console.log(msg)
        onMessage(msg)
      })
      .catch((e) => {
        const msg = '❌ Error: ' + e
        console.error(msg)
        onMessage(msg)
      })
  }

  // Test: Create transparent borderless window
  function createTransparentWindow() {
    console.log('Creating transparent borderless window...')
    invoke('create_transparent_window', {
      windowId: 'window_transparent'
    })
      .then(() => {
        const msg = '✅ Transparent borderless window created'
        console.log(msg)
        onMessage(msg)
      })
      .catch((e) => {
        const msg = '❌ Error: ' + e
        console.error(msg)
        onMessage(msg)
      })
  }

  function getAppVersion() {
    console.log('Getting app version...')
    getVersion()
      .then((v) => {
        const msg = '📦 App version: ' + v
        console.log(msg)
        onMessage(msg)
      })
      .catch((e) => {
        const msg = '❌ Get version error: ' + e
        console.error(msg)
        onMessage(msg)
      })
  }
</script>

<div>
  <button class="btn" id="log" onclick={log}>Call Log API</button>
  <button class="btn" id="request" onclick={performRequest}>
    Call Request (async) API
  </button>
  <button class="btn" id="event" onclick={emitEvent}>
    Send event to Rust
  </button>
  <button class="btn" id="request" onclick={echo}> Echo </button>
  <button class="btn" id="request" onclick={spam}> Spam </button>
  <button class="btn" id="test-eval" onclick={testEval}> Test Eval </button>
  <button class="btn" id="test-navigate" onclick={testNavigate}> Test Navigate </button>
  <button class="btn" id="test-reload" onclick={testReload}> Test Reload </button>
  <br><br>
  <button class="btn" id="test-custom-scheme" onclick={testCustomScheme}> 📡 Test Custom Scheme (myapp://) </button>
  <button class="btn" id="test-nav-intercept" onclick={testNavigationIntercept}> 🔗 Test Navigation Intercept </button>
  <button class="btn" id="test-resource-intercept" onclick={testResourceIntercept}> 📄 Test Resource Intercept </button>
  <br><br>
  <button class="btn" id="create-window-a" onclick={createWindowA}> 🪟 Create Isolated Window A </button>
  <button class="btn" id="create-window-b" onclick={createWindowB}> 🪟 Create Isolated Window B </button>
  <button class="btn" id="set-storage" onclick={setLocalStorage}> 💾 Set localStorage </button>
  <button class="btn" id="get-storage" onclick={getLocalStorage}> 📦 Get localStorage </button>
  <br><br>
  <button class="btn" id="custom-ua" onclick={createWindowWithCustomUA}> 🎭 Create Window with Custom User-Agent </button>
  <button class="btn" id="no-throttle" onclick={createWindowNoThrottle}> ⚡ Create Window with No Background Throttling </button>
  <br><br>
  <button class="btn" id="test-download" onclick={testDownload}> 📥 Test Download Intercept </button>
  <button class="btn" id="transparent-window" onclick={createTransparentWindow}> 🪟 Create Transparent Borderless Window </button>
  <br><br>
  <button class="btn" id="get-version" onclick={getAppVersion}> 📦 Get App Version </button>
</div>
