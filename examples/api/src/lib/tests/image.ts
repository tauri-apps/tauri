import type { TestCase } from '../test-runner';

function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

const MINIMAL_PNG = new Uint8Array([
  137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82,
  0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0, 144, 119, 83,
  222, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 0,
  0, 3, 1, 1, 0, 201, 254, 146, 239, 0, 0, 0, 0, 73, 69, 78,
  68, 174, 66, 96, 130
]);

export const imageTests: TestCase[] = [
  {
    name: '@tauri-apps/api/image.new',
    category: 'auto',
    async fn() {
      const { Image } = await import('@tauri-apps/api/image');
      const rgba = new Uint8Array([255, 0, 0, 255, 0, 255, 0, 255]);
      const img = await Image.new(rgba, 2, 1);
      assert(img.rid > 0, `Image.rid should be positive, got ${img.rid}`);
    },
  },
  {
    name: '@tauri-apps/api/image.size',
    category: 'auto',
    async fn() {
      const { Image } = await import('@tauri-apps/api/image');
      const rgba = new Uint8Array([255, 0, 0, 255]);
      const img = await Image.new(rgba, 1, 1);
      const size = await img.size();
      assert(size.width === 1, `size.width should be 1, got ${size.width}`);
      assert(size.height === 1, `size.height should be 1, got ${size.height}`);
    },
  },
  {
    name: '@tauri-apps/api/image.rgba',
    category: 'auto',
    async fn() {
      const { Image } = await import('@tauri-apps/api/image');
      const rgba = new Uint8Array([255, 0, 0, 255]);
      const img = await Image.new(rgba, 1, 1);
      const data = await img.rgba();
      assert(data.length === 4, `RGBA length should be 4, got ${data.length}`);
      assert(data[0] === 255, `RGBA[0] should be 255, got ${data[0]}`);
      assert(data[3] === 255, `RGBA[3] should be 255, got ${data[3]}`);
    },
  },
  {
    name: '@tauri-apps/api/image.fromBytes',
    category: 'auto',
    async fn() {
      const { Image } = await import('@tauri-apps/api/image');
      const img = await Image.fromBytes(MINIMAL_PNG);
      const size = await img.size();
      assert(size.width === 1, `PNG width should be 1, got ${size.width}`);
      assert(size.height === 1, `PNG height should be 1, got ${size.height}`);
    },
  },
  {
    name: '@tauri-apps/api/image.close',
    category: 'auto',
    async fn() {
      const { Image } = await import('@tauri-apps/api/image');
      const rgba = new Uint8Array([0, 0, 0, 255]);
      const img = await Image.new(rgba, 1, 1);
      await img.close();
      try {
        await img.size();
        throw new Error('size() should fail after close()');
      } catch (e: any) {
        const msg = e?.message ?? String(e);
        assert(
          !msg.includes('size() should fail after close()'),
          `expected error from backend after close, got: ${msg}`
        );
      }
    },
  },
];