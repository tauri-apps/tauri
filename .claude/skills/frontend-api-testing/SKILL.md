---
name: frontend-api-testing
description: Tauri 前端 API 自动化测试开发技能。使用场景：(1) 为新的 Tauri API 或 plugin 编写前端测试用例，(2) 添加测试到 core.ts 或 plugins.ts，(3) 配置 plugin 的 JS/Rust 依赖和权限，(4) 验证测试在 Windows/ohos 平台运行，(5) 分析测试报告定位问题。
---

# Tauri 前端 API 测试开发

本技能指导 agent 为 Tauri API 和 plugins 编写增量前端自动化测试。

## 快速导航

| 任务 | 操作 |
|------|------|
| **添加新测试** | 编辑 `core.ts` 或 `plugins.ts` → 添加 TestCase |
| **添加新 plugin** | Cargo.toml + lib.rs + package.json + capabilities |
| **运行测试** | Windows: `cargo tauri dev` → Tests tab；ohos: ohos-build skill |
| **查看报告** | Windows: Console 输出；ohos: `test-report.json` |

## 文件位置

```
examples/api/src/
├── lib/
│   ├── test-runner.ts          # 测试引擎（不要修改）
│   └── tests/
│       ├── core.ts             # @tauri-apps/api 测试（在此添加）
│       └── plugins.ts          # @tauri-apps/plugin-* 测试（在此添加）
└── App.svelte                  # autotest 触发逻辑
```

## 测试用例格式

```typescript
import type { TestCase } from '../test-runner';

export const myTests: TestCase[] = [
  {
    name: '@tauri-apps/plugin-xxx.functionName',
    category: 'auto',  // 或 'side-effect' 或 'manual'
    async fn() {
      const { functionName } = await import('@tauri-apps/plugin-xxx');
      const result = await functionName();
      assert(result !== undefined, 'expected result');
    },
  },
];
```

## 测试类别选择

| 类别 | 适用场景 | 自动执行 |
|------|----------|----------|
| `auto` | 纯函数调用，有返回值可断言 | ✓ |
| `side-effect` | 有副作用但可程序验证（fs、clipboard） | ✓ |
| `manual` | 需人工确认（dialog、notification） | ✗（跳过） |

## 编写新测试步骤

### 步骤 1：添加测试用例

在 `core.ts`（核心 API）或 `plugins.ts`（plugin）添加 TestCase。

**核心 API**：静态 import
```typescript
import { getVersion } from '@tauri-apps/api/app';
```

**Plugin**：动态 import（避免加载失败影响其他测试）
```typescript
const { mkdir } = await import('@tauri-apps/plugin-fs');
```

### 步骤 2：添加 Rust 依赖（新 plugin）

`examples/api/src-tauri/Cargo.toml`:
```toml
tauri-plugin-xxx = { path = "../../../../plugins-workspace/plugins/xxx" }
```

如果 plugin 不支持 ohos：
```toml
[target.'cfg(not(target_env = "ohos"))'.dependencies]
tauri-plugin-xxx = { path = "../../../../plugins-workspace/plugins/xxx" }
```

### 步骤 3：注册 plugin

`examples/api/src-tauri/src/lib.rs`:
```rust
#[cfg(not(target_env = "ohos"))]
let builder = builder.plugin(tauri_plugin_xxx::init());
```

### 步骤 4：添加 JS 依赖

`examples/api/package.json`:
```json
"@tauri-apps/plugin-xxx": "file:../../../plugins-workspace/plugins/xxx"
```

### 步骤 5：添加权限

`examples/api/src-tauri/capabilities/run-app.json`:
```json
"xxx:default"
```

### 步骤 6：验证

```powershell
cd D:\workspace\tauri\tauri\examples\api
pnpm build
cargo tauri dev
# 点击 Tests → Run All
```

## 断言方式

使用简单 assert 函数：

```typescript
function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

assert(typeof version === 'string', `expected string, got ${typeof version}`);
assert(result === expected, `mismatch: "${result}" vs "${expected}"`);
```

## 命名规范

测试名格式：
- `@tauri-apps/api/模块.函数名`
- `@tauri-apps/plugin-名称.函数名`

多函数组合测试：`@tauri-apps/plugin-fs.mkdir+writeFile+readFile`

## 运行测试

### Windows 开发模式

```powershell
cd D:\workspace\tauri\tauri\examples\api
cargo tauri dev
# 点击 Tests → Run All
```

或 URL 参数自动触发：`http://localhost:1420/?autotest=true`

### ohos 设备

使用 `ohos-build` skill 构建并运行测试，详见该 skill 的 SKILL.md。

## 测试报告

```json
{
  "timestamp": "2026-05-11T14:30:00Z",
  "total": 25,
  "passed": 20,
  "failed": 5,
  "skipped": 0,
  "results": [
    { "name": "@tauri-apps/api/core.invoke", "status": "pass", "duration": 14 },
    { "name": "@tauri-apps/plugin-fs.mkdir", "status": "fail", "error": "Timeout after 5000ms" }
  ]
}
```

## 常见问题

### Plugin command 未注册

检查 `capabilities/run-app.json` 是否包含对应权限。

### HTTP scope 限制

`plugin-http` fetch 需声明 URL scope：
```json
{ "identifier": "http:default", "allow": [{ "url": "https://www.example.com/*" }] }
```

### Plugin 在 ohos 编译失败

Cargo.toml 用 `cfg(not(target_env = "ohos"))` 排除。测试保留，失败即为待适配项。

### 动态 import 失败

确保 `plugins-workspace` 已执行 `pnpm build`。

## 参考资料

- [test-template.md](references/test-template.md) - 测试用例模板
- [frontend-test-guide.md](../../doc/frontend-test-guide.md) - 完整指南