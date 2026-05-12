# 测试用例模板

## 核心 API 测试模板

```typescript
// 在 core.ts 中添加

import type { TestCase } from '../test-runner';
import { functionName } from '@tauri-apps/api/module';

function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

// 添加到 coreTests 数组
{
  name: '@tauri-apps/api/module.functionName',
  category: 'auto',
  async fn() {
    const result = await functionName();
    assert(typeof result === 'string', `expected string, got ${typeof result}`);
    assert(result.length > 0, `expected non-empty result, got "${result}"`);
  },
},
```

## Plugin 测试模板

```typescript
// 在 plugins.ts 中添加

import type { TestCase } from '../test-runner';

function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

// 添加到 pluginTests 数组
{
  name: '@tauri-apps/plugin-xxx.functionName',
  category: 'auto',  // 或 'side-effect'
  async fn() {
    const { functionName } = await import('@tauri-apps/plugin-xxx');
    const result = await functionName({ param: 'value' });
    assert(result !== undefined, 'expected result');
    assert(result.status === 'success', `expected success, got ${result.status}`);
  },
},
```

## Side-effect 测试模板

```typescript
// 有副作用但可验证的测试

{
  name: '@tauri-apps/plugin-xxx.write+read+delete',
  category: 'side-effect',
  async fn() {
    const { write, read, delete } = await import('@tauri-apps/plugin-xxx');
    
    const testData = `test-${Date.now()}`;
    await write(testData);
    
    const result = await read();
    assert(result === testData, `mismatch: "${result}" vs "${testData}"`);
    
    await delete();
  },
},
```

## Manual 测试模板

```typescript
// 需人工确认的测试

{
  name: '@tauri-apps/plugin-dialog.message',
  category: 'manual',
  async fn() {
    // manual 测试在 autotest 模式下自动跳过
    // 在手动测试时执行
  },
},
```

## 多函数组合测试模板

```typescript
{
  name: '@tauri-apps/plugin-fs.mkdir+writeFile+stat+readFile+exists+remove',
  category: 'side-effect',
  async fn() {
    const { mkdir, writeFile, stat, readFile, exists, remove } = await import('@tauri-apps/plugin-fs');
    const { appCacheDir } = await import('@tauri-apps/api/path');

    const base = await appCacheDir();
    const testDir = `${base}/test-${Date.now()}`;
    const testFile = `${testDir}/file.txt`;
    const content = new TextEncoder().encode('test content');

    // 创建目录
    await mkdir(testDir, { recursive: true });
    
    // 写文件
    await writeFile(testFile, content);
    
    // 验证
    const info = await stat(testFile);
    assert(info.size === content.length, `size mismatch: ${info.size} vs ${content.length}`);
    
    // 读取
    const read = await readFile(testFile);
    const decoded = new TextDecoder().decode(read);
    assert(decoded === 'test content', `content mismatch: "${decoded}"`);
    
    // 清理
    await remove(testFile);
    await remove(testDir, { recursive: true });
    
    const afterRemove = await exists(testFile);
    assert(afterRemove === false, 'file still exists after remove');
  },
},
```

## 常见断言模式

| 断言类型 | 示例 |
|----------|------|
| 类型检查 | `assert(typeof x === 'string', ...)` |
| 非空检查 | `assert(x !== undefined && x !== null, ...)` |
| 值匹配 | `assert(x === expected, ...)` |
| 长度检查 | `assert(x.length > 0, ...)` |
| 数组长度 | `assert(arr.length === n, ...)` |
| 存在检查 | `assert(await exists(path) === true, ...)` |

## 新 Plugin 完整配置清单

### 1. Cargo.toml
```toml
tauri-plugin-xxx = { path = "../../../../plugins-workspace/plugins/xxx" }
```

### 2. lib.rs
```rust
#[cfg(not(target_env = "ohos"))]
let builder = builder.plugin(tauri_plugin_xxx::init());
```

### 3. package.json
```json
"@tauri-apps/plugin-xxx": "file:../../../plugins-workspace/plugins/xxx"
```

### 4. capabilities/run-app.json
```json
"xxx:default"
```

### 5. plugins.ts
```typescript
{
  name: '@tauri-apps/plugin-xxx.functionName',
  category: 'auto',
  async fn() {
    const { functionName } = await import('@tauri-apps/plugin-xxx');
    // ...
  },
},
```

### 6. 验证
```powershell
cd examples/api
pnpm build
cargo tauri dev
# Tests → Run All
```