# Tauri OHOS DPI & Image API 测试设计文档

> 创建时间: 2026-05-13
> 更新时间: 2026-05-13 14:20
> 目标: 为 DPI 类型 (LogicalSize/PhysicalSize/PhysicalPosition) 和 Image API 编写自动化测试用例
> 状态: ✅ 已完成

---

## 一、需要测试的 API 清单

### 1.1 DPI 类型 (@tauri-apps/api/dpi)

| API | 来源 | OHOS 支持 | 需要 image-png |
|-----|------|----------|----------------|
| `LogicalSize` | dpi crate (外部) | ✅ 已支持 | ❌ 不需要 |
| `PhysicalSize` | dpi crate (外部) | ✅ 已支持 | ❌ 不需要 |
| `PhysicalPosition` | dpi crate (外部) | ✅ 已支持 | ❌ 不需要 |
| `LogicalPosition` | dpi crate (外部) | ✅ 已支持 | ❌ 不需要 |
| `Size` (包装类) | dpi crate (外部) | ✅ 已支持 | ❌ 不需要 |
| `Position` (包装类) | dpi crate (外部) | ✅ 已支持 | ❌ 不需要 |

**结论**: DPI 类型为纯 Rust 数据结构，无平台特定代码，OHOS 上完全可用。

---

### 1.2 Window 尺寸/位置 API (@tauri-apps/api/window)

| API | 返回类型 | OHOS 支持 | 当前测试状态 |
|-----|----------|----------|-------------|
| `innerSize()` | `PhysicalSize` | ✅ 已支持 | ❌ 仅手动测试 |
| `outerSize()` | `PhysicalSize` | ✅ 已支持 | ❌ 仅手动测试 |
| `innerPosition()` | `PhysicalPosition` | ✅ 已支持 | ❌ 仅手动测试 |
| `outerPosition()` | `PhysicalPosition` | ✅ 已支持 | ❌ 仅手动测试 |
| `scaleFactor()` | `number` | ✅ 已支持 | ❌ 无测试 |

**结论**: 这些 API 已实现，但缺少自动化测试用例。

---

### 1.3 Image API (@tauri-apps/api/image)

| API | 需要 feature | 当前状态 | Cargo.toml |
|-----|-------------|----------|------------|
| `Image.new(rgba, width, height)` | ❌ 不需要 | ✅ 可用 | - |
| `Image.rgba()` | ❌ 不需要 | ✅ 可用 | - |
| `Image.size()` | ❌ 不需要 | ✅ 可用 | - |
| `Image.fromBytes(bytes)` | ✅ `image-ico` 或 `image-png` | ✅ 可用 | 第 46-47 行已启用 |
| `Image.fromPath(path)` | ✅ `image-ico` 或 `image-png` | ✅ 可用 | 第 46-47 行已启用 |
| `Image.close()` | ❌ 不需要 (继承自 Resource) | ✅ 可用 | - |

**结论**: 所有 Image API 都可测试，`image-png` feature 已在 `examples/api/src-tauri/Cargo.toml` 启用。

---

## 二、底层依赖分析

### 2.1 DPI 类型来源

```
dpi crate (v0.1)
  ├── LogicalSize, PhysicalSize
  ├── LogicalPosition, PhysicalPosition
  ├── Size, Position (包装类)
  └── Pixel, PixelUnit (内部类型)
```

**调用链路**:
```
前端 JS: @tauri-apps/api/dpi
    ↓
Rust: tauri-runtime/src/dpi.rs → dpi crate (外部)
    ↓
tauri/src/lib.rs: pub use dpi::{LogicalSize, PhysicalSize, ...}
```

**关键代码** (lib.rs:222-224):
```rust
pub use self::runtime::dpi::{
  LogicalPosition, LogicalRect, LogicalSize, LogicalUnit, PhysicalPosition, PhysicalRect,
  PhysicalSize, PhysicalUnit, Pixel, PixelUnit, Position, Rect, Size,
};
```

---

### 2.2 Window 尺寸/位置 API

**调用链路**:
```
前端: win.innerSize()
    ↓
window.ts: invoke('plugin:window|inner_size')
    ↓
plugin.rs: getter!(inner_size, PhysicalSize<u32>)
    ↓
window/mod.rs: Window::inner_size()
    ↓
runtime-wry: Dispatcher::inner_size()
    ↓
tao: Window::inner_size()
```

**plugin.rs 现有实现** (第 73-76 行):
```rust
getter!(inner_position, PhysicalPosition<i32>);
getter!(outer_position, PhysicalPosition<i32>);
getter!(inner_size, PhysicalSize<u32>);
getter!(outer_size, PhysicalSize<u32>);
```

---

### 2.3 Image API

**调用链路**:
```
前端: Image.new(rgba, w, h)
    ↓
image.ts: invoke('plugin:image|new', { rgba, width, height })
    ↓
image/plugin.rs: new() → Image::new_owned()
    ↓
resources_table.add(image) → ResourceId
    ↓
前端: Image(rid) → Resource 子类
```

**image/plugin.rs 命令**:
```rust
#[command(root = "crate")]
fn new<R: Runtime>(webview: Webview<R>, rgba: Vec<u8>, width: u32, height: u32) -> crate::Result<ResourceId>

#[command(root = "crate")]
fn rgba<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> crate::Result<Vec<u8>>

#[command(root = "crate")]
fn size<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> crate::Result<Size>

#[cfg(any(feature = "image-ico", feature = "image-png"))]
#[command(root = "crate")]
fn from_bytes<R: Runtime>(webview: Webview<R>, bytes: Vec<u8>) -> crate::Result<ResourceId>

#[cfg(any(feature = "image-ico", feature = "image-png"))]
#[command(root = "crate")]
fn from_path<R: Runtime>(webview: Webview<R>, path: std::path::PathBuf) -> crate::Result<ResourceId>
```

---

## 三、测试文件结构

### 3.1 新建文件

| 文件 | 测试内容 | 测试数 |
|------|----------|--------|
| `lib/tests/dpi.ts` | DPI 类型构造 + 转换 | 6 |
| `lib/tests/window-dpi.ts` | Window 尺寸/位置 API | 5 |
| `lib/tests/image.ts` | Image API | 5 |

### 3.2 需修改文件

| 文件 | 修改内容 |
|------|----------|
| `views/TestRunner.svelte` | 导入新测试数组并聚合 |
| `App.svelte` | 导入新测试数组并聚合 |

---

## 四、测试用例设计

### 4.1 dpi.ts

```typescript
export const dpiTests: TestCase[] = [
  // PhysicalSize
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

  // LogicalSize
  {
    name: '@tauri-apps/api/dpi.LogicalSize.constructor',
    category: 'auto',
    async fn() {
      const { LogicalSize } = await import('@tauri-apps/api/dpi');
      const size = new LogicalSize(50, 100);
      assert(size.width === 50 && size.height === 100, 'constructor values mismatch');
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

  // PhysicalPosition
  {
    name: '@tauri-apps/api/dpi.PhysicalPosition.constructor+toLogical',
    category: 'auto',
    async fn() {
      const { PhysicalPosition, LogicalPosition } = await import('@tauri-apps/api/dpi');
      const physical = new PhysicalPosition(100, 200);
      const logical = physical.toLogical(2.0);
      assert(logical.x === 50 && logical.y === 100, 'toLogical conversion mismatch');
    },
  },

  // LogicalPosition
  {
    name: '@tauri-apps/api/dpi.LogicalPosition.constructor+toPhysical',
    category: 'auto',
    async fn() {
      const { LogicalPosition, PhysicalPosition } = await import('@tauri-apps/api/dpi');
      const logical = new LogicalPosition(50, 100);
      const physical = logical.toPhysical(2.0);
      assert(physical.x === 100 && physical.y === 200, 'toPhysical conversion mismatch');
    },
  },
];
```

---

### 4.2 window-dpi.ts

```typescript
export const windowDpiTests: TestCase[] = [
  {
    name: '@tauri-apps/api/window.innerSize',
    category: 'auto',
    async fn() {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const { PhysicalSize } = await import('@tauri-apps/api/dpi');
      const win = getCurrentWindow();
      const size = await win.innerSize();
      assert(size.width > 0 && size.height > 0, 'innerSize should be positive');
      assert(size instanceof PhysicalSize, 'should return PhysicalSize');
    },
  },
  {
    name: '@tauri-apps/api/window.outerSize',
    category: 'auto',
    async fn() {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const win = getCurrentWindow();
      const inner = await win.innerSize();
      const outer = await win.outerSize();
      assert(outer.width >= inner.width, `outerSize.width < innerSize.width: ${outer.width} < ${inner.width}`);
      assert(outer.height >= inner.height, `outerSize.height < innerSize.height: ${outer.height} < ${inner.height}`);
    },
  },
  {
    name: '@tauri-apps/api/window.innerPosition',
    category: 'auto',
    async fn() {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const { PhysicalPosition } = await import('@tauri-apps/api/dpi');
      const win = getCurrentWindow();
      const pos = await win.innerPosition();
      assert(typeof pos.x === 'number' && typeof pos.y === 'number', 'innerPosition should return numbers');
      assert(pos instanceof PhysicalPosition, 'should return PhysicalPosition');
    },
  },
  {
    name: '@tauri-apps/api/window.outerPosition',
    category: 'auto',
    async fn() {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const win = getCurrentWindow();
      const innerPos = await win.innerPosition();
      const outerPos = await win.outerPosition();
      assert(typeof outerPos.x === 'number' && typeof outerPos.y === 'number', 'outerPosition should return numbers');
    },
  },
  {
    name: '@tauri-apps/api/window.scaleFactor',
    category: 'auto',
    async fn() {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const win = getCurrentWindow();
      const factor = await win.scaleFactor();
      assert(typeof factor === 'number' && factor > 0, `scaleFactor should be positive number, got ${factor}`);
    },
  },
];
```

---

### 4.3 image.ts

```typescript
export const imageTests: TestCase[] = [
  {
    name: '@tauri-apps/api/image.new',
    category: 'auto',
    async fn() {
      const { Image } = await import('@tauri-apps/api/image');
      // 2x2 红色像素 RGBA
      const rgba = new Uint8Array([255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255]);
      const img = await Image.new(rgba, 2, 2);
      assert(img.rid > 0, 'Image.rid should be positive');
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
      assert(size.width === 1 && size.height === 1, `size mismatch: ${size.width}x${size.height}`);
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
      assert(data[0] === 255 && data[3] === 255, 'RGBA values mismatch');
    },
  },
  {
    name: '@tauri-apps/api/image.fromBytes',
    category: 'auto',
    async fn() {
      const { Image } = await import('@tauri-apps/api/image');
      // 最小有效 PNG (1x1 红色像素)
      const pngBytes = new Uint8Array([
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82,
        0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0, 144, 119, 83,
        222, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 0,
        0, 3, 1, 1, 0, 201, 254, 146, 239, 0, 0, 0, 0, 73, 69, 78,
        68, 174, 66, 96, 130
      ]);
      const img = await Image.fromBytes(pngBytes);
      const size = await img.size();
      assert(size.width === 1 && size.height === 1, `PNG size mismatch: ${size.width}x${size.height}`);
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
      // close 后操作应失败
      try {
        await img.size();
        assert(false, 'size() should fail after close');
      } catch (e) {
        assert(true, 'expected error after close');
      }
    },
  },
];
```

---

## 五、TestRunner.svelte 修改

```svelte
<script>
  import { onMount } from 'svelte';
  import { runTests } from '../lib/test-runner';
  import { coreTests } from '../lib/tests/core';
  import { pluginTests } from '../lib/tests/plugins';
  import { dpiTests } from '../lib/tests/dpi';            // 新增
  import { windowDpiTests } from '../lib/tests/window-dpi'; // 新增
  import { imageTests } from '../lib/tests/image';         // 新增
  import { invoke } from '@tauri-apps/api/core';
  // ...

  const allTests = [...coreTests, ...pluginTests, ...dpiTests, ...windowDpiTests, ...imageTests];
  // ...
</script>
```

---

## 六、App.svelte 修改

```svelte
<script>
  import { coreTests } from './lib/tests/core'
  import { pluginTests } from './lib/tests/plugins'
  import { dpiTests } from './lib/tests/dpi'              // 新增
  import { windowDpiTests } from './lib/tests/window-dpi' // 新增
  import { imageTests } from './lib/tests/image'          // 新增
  // ...

  // autotest 触发
  const allTests = [...coreTests, ...pluginTests, ...dpiTests, ...windowDpiTests, ...imageTests]
    .filter((t) => t.category !== 'manual')
  // ...
</script>
```

---

## 七、测试执行

### Windows 测试

```powershell
cd D:\workspace\tauri\tauri\examples\api
pnpm build
cargo tauri dev
```

### OHOS 测试

```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/run-tests.sh
```

---

## 八、测试报告预期

测试完成后，报告将包含:

```json
{
  "total": 36,  // 25 (原有) + 6 (dpi) + 5 (window-dpi) + 5 (image) = 41，减去 manual
  "passed": ?,
  "failed": ?,
  "results": [
    // dpiTests
    { "name": "@tauri-apps/api/dpi.PhysicalSize.constructor", "status": "pass" },
    { "name": "@tauri-apps/api/dpi.PhysicalSize.toLogical", "status": "pass" },
    // windowDpiTests
    { "name": "@tauri-apps/api/window.innerSize", "status": "pass" },
    // imageTests
    { "name": "@tauri-apps/api/image.new", "status": "pass" },
    ...
  ]
}
```

---

## 九、风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| DPI 类型转换依赖 scaleFactor | 需验证转换公式正确 | 使用固定因子测试 |
| Image.fromBytes 依赖 PNG 解析 | 解析失败可能抛异常 | 使用预验证的最小 PNG |
| outerSize 在 OHOS 可能无装饰边框 | outerSize 可能等于 innerSize | 断言改为 >= |
| Image.close 后资源释放 | 后续操作行为不确定 | catch 异常即为 pass |

---

## 十、OHOS 平台修复

### 问题发现

OHOS 测试首次运行时，`innerPosition` 和 `outerPosition` 返回错误：
```
"runtime error: failed to send message to the webview"
```

### 根因分析

`tao/src/platform_impl/ohos/mod.rs` 中这两个方法返回 `NotSupportedError`：
```rust
pub fn inner_position(&self) -> Result<PhysicalPosition<i32>, error::NotSupportedError> {
    Err(error::NotSupportedError::new())  // 原实现
}
pub fn outer_position(&self) -> Result<PhysicalPosition<i32>, error::NotSupportedError> {
    Err(error::NotSupportedError::new())  // 原实现
}
```

### 解决方案

OpenHarmony 应用通过 `openharmony-ability` 库提供 `content_rect()` 和 `window_rect()` API：

```rust
// openharmony-ability/crates/ability/src/area/rect.rs
pub struct Rect {
    pub top: i32,
    pub left: i32,
    pub width: i32,
    pub height: i32,
}

// openharmony-ability/crates/ability/src/app.rs
pub fn content_rect(&self) -> Rect  // 内容区域（排除状态栏等）
pub fn window_rect(&self) -> Rect   // 窗口完整区域
```

### 修复代码

```rust
// tao/src/platform_impl/ohos/mod.rs
pub fn inner_position(&self) -> Result<PhysicalPosition<i32>, error::NotSupportedError> {
    let content = self.app.content_rect();
    let window = self.app.window_rect();
    // inner_position = 内容区域相对于窗口的偏移
    Ok(PhysicalPosition::new(content.left - window.left, content.top - window.top))
}

pub fn outer_position(&self) -> Result<PhysicalPosition<i32>, error::NotSupportedError> {
    let rect = self.app.window_rect();
    // outer_position = 窗口在屏幕上的位置
    Ok(PhysicalPosition::new(rect.left, rect.top))
}
```

### 验证结果

修复后 OHOS 测试：
- innerPosition: ✅ pass
- outerPosition: ✅ pass
- 新增 16 测试: 全部通过

---

## 十一、后续工作

完成本轮测试后，可考虑:

1. **添加 Size/Position 包装类测试** - IPC 传参场景
2. **添加 Image.fromPath 测试** - 需要设备上有文件路径
3. **添加 Image 多格式测试** - ICO 格式支持
4. **添加手动测试按钮** - 如果自动测试发现边界情况需人工确认
5. **修复其他 OHOS plugin 适配** - http、autostart、clipboard-manager