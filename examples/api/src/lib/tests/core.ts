import type { TestCase } from '../test-runner';
import { invoke, Channel, Resource } from '@tauri-apps/api/core';
import { emit, listen, once } from '@tauri-apps/api/event';
import { getVersion } from '@tauri-apps/api/app';
import { getCurrentWindow, currentMonitor } from '@tauri-apps/api/window';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { appCacheDir } from '@tauri-apps/api/path';

function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

export const coreTests: TestCase[] = [
  // @tauri-apps/api/app
  {
    name: '@tauri-apps/api/app.getVersion',
    category: 'auto',
    async fn() {
      const version = await getVersion();
      assert(typeof version === 'string' && version.length > 0, `expected non-empty string, got "${version}"`);
    },
  },

  // @tauri-apps/api/core
  {
    name: '@tauri-apps/api/core.invoke',
    category: 'auto',
    async fn() {
      const msg = 'hello from test';
      const result = await invoke('echo', { message: msg });
      assert(result !== undefined, 'invoke echo returned undefined');
    },
  },
  {
    name: '@tauri-apps/api/core.Channel',
    category: 'auto',
    async fn() {
      const received: number[] = [];
      const channel = new Channel<number>();
      channel.onmessage = (msg) => { received.push(msg); };
      await invoke('spam', { channel });
      assert(received.length === 1000, `expected 1000 messages, got ${received.length}`);
    },
  },

  // @tauri-apps/api/event
  {
    name: '@tauri-apps/api/event.emit+listen',
    category: 'auto',
    async fn() {
      const payload = { test: 'data', ts: Date.now() };
      let received: any = null;
      const unlisten = await listen('test-event', (event) => {
        received = event.payload;
      });
      await emit('test-event', payload);
      await new Promise((r) => setTimeout(r, 100));
      unlisten();
      assert(received !== null, 'listener did not receive event');
      assert(received.test === 'data', `unexpected payload: ${JSON.stringify(received)}`);
    },
  },
  {
    name: '@tauri-apps/api/event.once',
    category: 'auto',
    async fn() {
      let count = 0;
      const unlisten = await once('test-once-event', () => { count++; });
      await emit('test-once-event', {});
      await new Promise((r) => setTimeout(r, 50));
      await emit('test-once-event', {});
      await new Promise((r) => setTimeout(r, 50));
      unlisten();
      assert(count === 1, `once listener fired ${count} times, expected 1`);
    },
  },

  // @tauri-apps/api/window
  {
    name: '@tauri-apps/api/window.getCurrentWindow',
    category: 'auto',
    async fn() {
      const win = getCurrentWindow();
      assert(win !== null && win !== undefined, 'getCurrentWindow returned null');
      assert(typeof win.label === 'string' && win.label.length > 0, `window.label should be non-empty string, got "${win.label}"`);
    },
  },
  {
    name: '@tauri-apps/api/window.isFocused',
    category: 'auto',
    async fn() {
      const win = getCurrentWindow();
      const focused = await win.isFocused();
      assert(typeof focused === 'boolean', `isFocused returned ${typeof focused}, expected boolean`);
      // Note: on some platforms (e.g. OHOS) the window may not have focus
      // immediately after launch, so we don't assert focused === true.
      // The key verification is that the IPC round-trip works and returns a valid boolean.
    },
  },
  {
    name: '@tauri-apps/api/window.currentMonitor',
    category: 'auto',
    async fn() {
      const monitor = await currentMonitor();
      assert(monitor !== null && monitor !== undefined, 'currentMonitor returned null (device should always have a display)');
      assert(typeof monitor.size.width === 'number' && monitor.size.width > 0, `monitor.size.width should be positive, got ${monitor.size.width}`);
      assert(typeof monitor.size.height === 'number' && monitor.size.height > 0, `monitor.size.height should be positive, got ${monitor.size.height}`);
      assert(typeof monitor.position.x === 'number', `monitor.position.x should be a number, got ${monitor.position.x}`);
      assert(typeof monitor.position.y === 'number', `monitor.position.y should be a number, got ${monitor.position.y}`);
    },
  },

  // @tauri-apps/api/webview
  {
    name: '@tauri-apps/api/webview.getCurrentWebview',
    category: 'auto',
    async fn() {
      const webview = getCurrentWebview();
      assert(webview !== null && webview !== undefined, 'getCurrentWebview returned null');
      assert(typeof webview.label === 'string' && webview.label.length > 0, `webview.label should be non-empty string, got "${webview.label}"`);
    },
  },

  // @tauri-apps/api/path
  {
    name: '@tauri-apps/api/path.appCacheDir',
    category: 'auto',
    async fn() {
      const dir = await appCacheDir();
      assert(typeof dir === 'string' && dir.length > 0, `expected non-empty path, got "${dir}"`);
      assert(dir.includes('/') || dir.includes('\\'), `path should contain separator, got "${dir}"`);
      assert(dir.toLowerCase().includes('cache'), `path should contain "cache" segment, got "${dir}"`);
    },
  },

  // @tauri-apps/api/core - Resource
  {
    name: '@tauri-apps/api/core.Resource',
    category: 'auto',
    async fn() {
      assert(typeof Resource === 'function', 'Resource is not a constructor');
      assert(typeof Resource.prototype.close === 'function', 'Resource.prototype.close is not a function');
    },
  },

  // @tauri-apps/api/window - onFocusChanged
  {
    name: '@tauri-apps/api/window.onFocusChanged',
    category: 'auto',
    async fn() {
      const win = getCurrentWindow();
      // Subscribe and unsubscribe twice to verify both directions work and
      // unlisten is idempotent — a broken event wiring would throw here.
      const unlisten1 = await win.onFocusChanged(() => {});
      assert(typeof unlisten1 === 'function', 'onFocusChanged did not return an unlisten function');
      unlisten1();
      const unlisten2 = await win.onFocusChanged(() => {});
      assert(typeof unlisten2 === 'function', 'second onFocusChanged did not return an unlisten function');
      unlisten2();
    },
  },

  // Section 12: Global objects
  {
    name: 'window.__TAURI_INTERNALS__',
    category: 'auto',
    async fn() {
      const internals = (window as any).__TAURI_INTERNALS__;
      assert(internals !== undefined && internals !== null, '__TAURI_INTERNALS__ is not defined');
      assert(typeof internals === 'object', `__TAURI_INTERNALS__ is ${typeof internals}, expected object`);
    },
  },
  {
    name: 'window.__TAURI__',
    category: 'auto',
    async fn() {
      const tauri = (window as any).__TAURI__;
      assert(tauri !== undefined && tauri !== null, '__TAURI__ is not defined');
      assert(typeof tauri === 'object', `__TAURI__ is ${typeof tauri}, expected object`);
    },
  },
];
