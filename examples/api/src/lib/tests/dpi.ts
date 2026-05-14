import type { TestCase } from '../test-runner';

function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

export const dpiTests: TestCase[] = [
  {
    name: '@tauri-apps/api/dpi.PhysicalSize.constructor',
    category: 'auto',
    async fn() {
      const { PhysicalSize } = await import('@tauri-apps/api/dpi');
      const size = new PhysicalSize(100, 200);
      assert(size.width === 100, `width mismatch: ${size.width}`);
      assert(size.height === 200, `height mismatch: ${size.height}`);
      assert(size.type === 'Physical', `type mismatch: ${size.type}`);
    },
  },
  {
    name: '@tauri-apps/api/dpi.PhysicalSize.toLogical',
    category: 'auto',
    async fn() {
      const { PhysicalSize, LogicalSize } = await import('@tauri-apps/api/dpi');
      const physical = new PhysicalSize(100, 200);
      const logical = physical.toLogical(2.0);
      assert(logical.width === 50, `expected 50, got ${logical.width}`);
      assert(logical.height === 100, `expected 100, got ${logical.height}`);
      assert(logical instanceof LogicalSize, 'should return LogicalSize');
    },
  },
  {
    name: '@tauri-apps/api/dpi.LogicalSize.constructor',
    category: 'auto',
    async fn() {
      const { LogicalSize } = await import('@tauri-apps/api/dpi');
      const size = new LogicalSize(50, 100);
      assert(size.width === 50, `width mismatch: ${size.width}`);
      assert(size.height === 100, `height mismatch: ${size.height}`);
      assert(size.type === 'Logical', `type mismatch: ${size.type}`);
    },
  },
  {
    name: '@tauri-apps/api/dpi.LogicalSize.toPhysical',
    category: 'auto',
    async fn() {
      const { LogicalSize, PhysicalSize } = await import('@tauri-apps/api/dpi');
      const logical = new LogicalSize(50, 100);
      const physical = logical.toPhysical(2.0);
      assert(physical.width === 100, `expected 100, got ${physical.width}`);
      assert(physical.height === 200, `expected 200, got ${physical.height}`);
      assert(physical instanceof PhysicalSize, 'should return PhysicalSize');
    },
  },
  {
    name: '@tauri-apps/api/dpi.PhysicalPosition.constructor+toLogical',
    category: 'auto',
    async fn() {
      const { PhysicalPosition, LogicalPosition } = await import('@tauri-apps/api/dpi');
      const physical = new PhysicalPosition(100, 200);
      assert(physical.x === 100, `x mismatch: ${physical.x}`);
      assert(physical.y === 200, `y mismatch: ${physical.y}`);
      assert(physical.type === 'Physical', `type mismatch: ${physical.type}`);
      const logical = physical.toLogical(2.0);
      assert(logical.x === 50, `expected 50, got ${logical.x}`);
      assert(logical.y === 100, `expected 100, got ${logical.y}`);
      assert(logical instanceof LogicalPosition, 'should return LogicalPosition');
    },
  },
  {
    name: '@tauri-apps/api/dpi.LogicalPosition.constructor+toPhysical',
    category: 'auto',
    async fn() {
      const { LogicalPosition, PhysicalPosition } = await import('@tauri-apps/api/dpi');
      const logical = new LogicalPosition(50, 100);
      assert(logical.x === 50, `x mismatch: ${logical.x}`);
      assert(logical.y === 100, `y mismatch: ${logical.y}`);
      assert(logical.type === 'Logical', `type mismatch: ${logical.type}`);
      const physical = logical.toPhysical(2.0);
      assert(physical.x === 100, `expected 100, got ${physical.x}`);
      assert(physical.y === 200, `expected 200, got ${physical.y}`);
      assert(physical instanceof PhysicalPosition, 'should return PhysicalPosition');
    },
  },
];