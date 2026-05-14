import type { TestCase } from '../test-runner';
import { getCurrentWindow } from '@tauri-apps/api/window';

function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

export const windowDpiTests: TestCase[] = [
  {
    name: '@tauri-apps/api/window.innerSize',
    category: 'auto',
    async fn() {
      const { PhysicalSize } = await import('@tauri-apps/api/dpi');
      const win = getCurrentWindow();
      const size = await win.innerSize();
      assert(size.width > 0, `innerSize.width should be positive, got ${size.width}`);
      assert(size.height > 0, `innerSize.height should be positive, got ${size.height}`);
      assert(size instanceof PhysicalSize, `should return PhysicalSize, got ${size.constructor.name}`);
    },
  },
  {
    name: '@tauri-apps/api/window.outerSize',
    category: 'auto',
    async fn() {
      const win = getCurrentWindow();
      const inner = await win.innerSize();
      const outer = await win.outerSize();
      assert(outer.width >= inner.width, `outerSize.width (${outer.width}) should >= innerSize.width (${inner.width})`);
      assert(outer.height >= inner.height, `outerSize.height (${outer.height}) should >= innerSize.height (${inner.height})`);
    },
  },
  {
    name: '@tauri-apps/api/window.innerPosition',
    category: 'auto',
    async fn() {
      const { PhysicalPosition } = await import('@tauri-apps/api/dpi');
      const win = getCurrentWindow();
      const pos = await win.innerPosition();
      assert(typeof pos.x === 'number', `innerPosition.x should be number, got ${typeof pos.x}`);
      assert(typeof pos.y === 'number', `innerPosition.y should be number, got ${typeof pos.y}`);
      assert(pos instanceof PhysicalPosition, `should return PhysicalPosition, got ${pos.constructor.name}`);
    },
  },
  {
    name: '@tauri-apps/api/window.outerPosition',
    category: 'auto',
    async fn() {
      const win = getCurrentWindow();
      const pos = await win.outerPosition();
      assert(typeof pos.x === 'number', `outerPosition.x should be number, got ${typeof pos.x}`);
      assert(typeof pos.y === 'number', `outerPosition.y should be number, got ${typeof pos.y}`);
    },
  },
  {
    name: '@tauri-apps/api/window.scaleFactor',
    category: 'auto',
    async fn() {
      const win = getCurrentWindow();
      const factor = await win.scaleFactor();
      assert(typeof factor === 'number', `scaleFactor should be number, got ${typeof factor}`);
      assert(factor > 0, `scaleFactor should be positive, got ${factor}`);
    },
  },
];