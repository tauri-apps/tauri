<script>
  import { onMount } from 'svelte';
  import { runTests } from '../lib/test-runner';
  import { coreTests } from '../lib/tests/core';
  import { pluginTests } from '../lib/tests/plugins';
  import { dpiTests } from '../lib/tests/dpi';
  import { windowDpiTests } from '../lib/tests/window-dpi';
  import { imageTests } from '../lib/tests/image';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow, currentMonitor } from '@tauri-apps/api/window';
  import { appCacheDir } from '@tauri-apps/api/path';
  import { flushConsoleLog, clearConsoleLog } from '../lib/console-capture';

  let { onMessage } = $props();

  let results = $state([]);
  let running = $state(false);
  let report = $state(null);

  // Manual test state
  let manualResult = $state('');
  let focusWatchActive = $state(false);
  let focusWatchUnlisten = null;
  let focusEvents = $state([]);

  const allTests = [...coreTests, ...pluginTests, ...dpiTests, ...windowDpiTests, ...imageTests];

  async function runAll() {
    running = true;
    results = [];
    report = null;
    onMessage('--- Test Run Started ---');

    const r = await runTests(allTests, (result, index, total) => {
      results = [...results, result];
      const icon = result.status === 'pass' ? '[PASS]' : result.status === 'fail' ? '[FAIL]' : '[SKIP]';
      const msg = `${icon} ${result.name}${result.error ? ' - ' + result.error : ''} (${result.duration}ms)`;
      onMessage(msg);
    });

    report = r;
    onMessage(`--- Done: ${r.passed} passed, ${r.failed} failed, ${r.skipped} skipped ---`);
    running = false;

    try {
      await invoke('write_test_report', { report: JSON.stringify(r) });
      onMessage('Report saved to device.');
    } catch (e) {
      onMessage(`Failed to save report: ${e}`);
    }
  }

  async function runCategory(category) {
    running = true;
    results = [];
    report = null;
    const filtered = allTests.filter((t) => t.category === category);
    onMessage(`--- Running ${category} tests (${filtered.length}) ---`);

    const r = await runTests(filtered, (result) => {
      results = [...results, result];
      const icon = result.status === 'pass' ? '[PASS]' : result.status === 'fail' ? '[FAIL]' : '[SKIP]';
      onMessage(`${icon} ${result.name}${result.error ? ' - ' + result.error : ''}`);
    });

    report = r;
    onMessage(`--- Done: ${r.passed} passed, ${r.failed} failed, ${r.skipped} skipped ---`);
    running = false;
  }

  // Auto-run on first mount so opening the Tests view kicks off tests immediately.
  onMount(() => {
    runAll();
  });

  async function wrapManual(name, fn) {
    const start = Date.now();
    console.log('[ManualTest] Starting:', name);
    try {
      await fn();
      if (manualResult) {
        console.log('[ManualTest]', manualResult);
      }
      console.log('[ManualTest] Completed:', name, 'in', Date.now() - start, 'ms');
    } catch (e) {
      console.error('[ManualTest] Failed:', name, e);
    }
    try {
      const path = await flushConsoleLog();
      onMessage(`Console log saved: ${path}`);
    } catch (e) {
      onMessage(`Failed to save console log: ${e}`);
    }
  }

  // ─── Manual Tests ───
  async function manualIsFocused() {
    await wrapManual('isFocused', async () => {
      const focused = await getCurrentWindow().isFocused();
      const ok = focused === true;
      manualResult = `isFocused() → ${focused} ${ok ? '[OK: app in foreground]' : '[UNEXPECTED: should be true since you clicked the button]'}`;
      onMessage(manualResult);
    });
  }

  async function toggleFocusWatch() {
    if (focusWatchActive) {
      focusWatchUnlisten?.();
      focusWatchUnlisten = null;
      focusWatchActive = false;
      manualResult = `Stopped watching focus changes. Total events: ${focusEvents.length}`;
      onMessage(manualResult);
    } else {
      focusEvents = [];
      focusWatchUnlisten = await getCurrentWindow().onFocusChanged(({ payload }) => {
        const ts = new Date().toLocaleTimeString();
        focusEvents = [...focusEvents, `${ts}: focused=${payload}`];
        onMessage(`[onFocusChanged] focused=${payload}`);
      });
      focusWatchActive = true;
      manualResult = 'Watching focus changes. Send the app to background and back to trigger events.';
      onMessage(manualResult);
    }
    try {
      const path = await flushConsoleLog();
      onMessage(`Console log saved: ${path}`);
    } catch (e) {}
  }

  async function manualMonitor() {
    await wrapManual('currentMonitor', async () => {
      const m = await currentMonitor();
      if (!m) {
        manualResult = 'currentMonitor() → null';
      } else {
        manualResult = `Monitor: ${m.size.width}×${m.size.height} @ scale ${m.scaleFactor} | position (${m.position.x}, ${m.position.y}) | name "${m.name ?? ''}"`;
      }
      onMessage(manualResult);
    });
  }

  async function manualAppCacheDir() {
    await wrapManual('appCacheDir', async () => {
      const dir = await appCacheDir();
      manualResult = `appCacheDir() → ${dir}`;
      onMessage(manualResult);
    });
  }

  async function manualWindowDpi() {
    await wrapManual('windowDpi', async () => {
      const win = getCurrentWindow();
      const inner = await win.innerSize();
      const outer = await win.outerSize();
      const innerPos = await win.innerPosition();
      const outerPos = await win.outerPosition();
      const scale = await win.scaleFactor();

      manualResult = `innerSize: ${inner.width}×${inner.height}
outerSize: ${outer.width}×${outer.height}
innerPosition: (${innerPos.x}, ${innerPos.y})
outerPosition: (${outerPos.x}, ${outerPos.y})
scaleFactor: ${scale}

Expected behavior:
• Resize window → innerSize/outerSize should change
• Drag window → positions should change
• outerSize >= innerSize (includes window decorations)
• scaleFactor typically 1.0-3.0 (depends on display DPI)`;
      onMessage(manualResult);
    });
  }
</script>

<div class="flex flex-col gap-2">
  <div class="flex gap-2 flex-wrap">
    <button class="btn" onclick={runAll} disabled={running}>
      {running ? 'Running...' : 'Run All'}
    </button>
    <button class="btn" onclick={() => runCategory('auto')} disabled={running}>
      Run Auto
    </button>
    <button class="btn" onclick={() => runCategory('side-effect')} disabled={running}>
      Run Side-Effect
    </button>
    <button class="btn" onclick={async () => {
      try {
        await clearConsoleLog();
        onMessage('Console log cleared');
      } catch (e) {
        onMessage(`Failed to clear: ${e}`);
      }
    }}>
      Clear Console
    </button>
  </div>

  {#if report}
    <div class="text-sm mt-2 p-2 rd-1 bg-black/10 dark:bg-white/10">
      Total: {report.total} | Passed: {report.passed} | Failed: {report.failed} | Skipped: {report.skipped}
    </div>
  {/if}

  {#if results.length > 0}
    <div class="flex flex-col gap-1 mt-2 text-xs max-h-60 overflow-y-auto">
      {#each results as r}
        <div class="flex items-center gap-2 p-1 rd-1 {r.status === 'pass' ? 'bg-green-500/10' : r.status === 'fail' ? 'bg-red-500/10' : 'bg-gray-500/10'}">
          <span class="font-mono w-12 shrink-0">
            {r.status === 'pass' ? 'PASS' : r.status === 'fail' ? 'FAIL' : 'SKIP'}
          </span>
          <span class="flex-1 truncate">{r.name}</span>
          <span class="text-gray-500 shrink-0">{r.duration}ms</span>
        </div>
      {/each}
    </div>
  {/if}

  <div class="mt-4 pt-3 border-t-1 border-solid border-code">
    <h4 class="my-2">Manual Tests</h4>
    <p class="text-xs text-gray-500 mb-2">
      Verifies behavior that autotest can't cover (e.g., focus state must be true when user is interacting).
    </p>
    <div class="flex gap-2 flex-wrap">
      <button class="btn" onclick={manualIsFocused}>isFocused (should be true)</button>
      <button class="btn" onclick={toggleFocusWatch}>
        {focusWatchActive ? 'Stop watching focus' : 'Watch onFocusChanged'}
      </button>
      <button class="btn" onclick={manualMonitor}>currentMonitor</button>
      <button class="btn" onclick={manualAppCacheDir}>appCacheDir</button>
      <button class="btn" onclick={manualWindowDpi}>Window DPI (resize/drag to verify)</button>
    </div>
    {#if manualResult}
      <div class="mt-2 p-2 rd-1 bg-black/10 dark:bg-white/10 text-xs font-mono break-all">
        {manualResult}
      </div>
    {/if}
    {#if focusWatchActive || focusEvents.length > 0}
      <div class="mt-2 text-xs">
        <div class="font-bold mb-1">Focus events ({focusEvents.length}):</div>
        <div class="max-h-32 overflow-y-auto flex flex-col gap-1">
          {#each focusEvents as ev}
            <div class="font-mono p-1 rd-1 bg-black/5 dark:bg-white/5">{ev}</div>
          {/each}
          {#if focusEvents.length === 0 && focusWatchActive}
            <div class="text-gray-500 italic">Waiting... send app to background and bring it back.</div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
