# 前端自动化测试编写指南

本文档说明如何为 Tauri API 和 plugins 编写前端自动化测试，供后续 agent 或开发者参考。

## 文件结构

```
examples/api/src/
├── lib/
│   ├── test-runner.ts          # 测试引擎（含超时机制）
│   └── tests/
│       ├── core.ts             # @tauri-apps/api 核心测试
│       └── plugins.ts          # @tauri-apps/plugin-* 测试
├── views/
│   └── TestRunner.svelte       # 测试 UI
└── App.svelte                  # 注册 view + autotest 触发
```

## 测试用例格式

每个测试用例是一个 `TestCase` 对象：

```typescript
import type { TestCase } from '../test-runner';

export const myTests: TestCase[] = [
  {
    name: '@tauri-apps/plugin-xxx.functionName',
    category: 'auto',
    async fn() {
      const result = await someApiCall();
      assert(result !== undefined, 'expected result');
    },
  },
];
```

## 测试类别

| 类别 | 说明 | 何时使用 |
|------|------|----------|
| `auto` | 纯函数调用，有明确返回值可断言 | getVersion(), platform(), invoke() |
| `side-effect` | 有副作用但可程序验证 | fs 读写、clipboard 读写、autostart |
| `manual` | 需要人工确认（UI 弹窗、系统行为） | dialog, notification, process.relaunch |

- `auto` 和 `side-effect` 在 autotest 模式下自动执行
- `manual` 被跳过（status = skip）

## 超时机制

测试引擎内置 5 秒超时，防止未实现的 API 卡死：

```typescript
const TEST_TIMEOUT_MS = 5000;
await withTimeout(test.fn(), TEST_TIMEOUT_MS);
```

超时后测试标记为 fail，error 显示 "Timeout after 5000ms"。

## 断言方式

使用简单的 assert 函数：

```typescript
function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

assert(typeof version === 'string', `expected string, got ${typeof version}`);
assert(result === expected, `mismatch: "${result}" vs "${expected}"`);
```

## 命名规范

测试名必须清晰体现被测 API：

```
@tauri-apps/api/模块.函数名
@tauri-apps/plugin-名称.函数名
```

## 编写新测试的步骤

### 1. 添加测试用例

在 `core.ts` 或 `plugins.ts` 中添加 TestCase。

### 2. 添加 import

```typescript
// 核心 API：静态 import
import { getVersion } from '@tauri-apps/api/app';

// Plugin：动态 import（避免加载失败影响其他测试）
const { functionName } = await import('@tauri-apps/plugin-xxx');
```

### 3. 添加 Rust 依赖（如果是新 plugin）

`Cargo.toml`:
```toml
tauri-plugin-xxx = { path = "../../../../plugins-workspace/plugins/xxx" }
```

如果 plugin 不支持 ohos，用 cfg gate：
```toml
[target.'cfg(not(target_env = "ohos"))'.dependencies]
tauri-plugin-xxx = { path = "..." }
```

### 4. 注册 plugin（Rust 侧）

`lib.rs`:
```rust
#[cfg(not(target_env = "ohos"))]
let builder = builder.plugin(tauri_plugin_xxx::init());
```

### 5. 添加 JS 依赖

`package.json`:
```json
"@tauri-apps/plugin-xxx": "file:../../../plugins-workspace/plugins/xxx"
```

### 6. 添加权限

`capabilities/run-app.json`:
```json
"xxx:default"
```

### 7. 验证

```bash
# Windows 验证
cd examples/api && pnpm build && cargo tauri dev
# 点击 Tests → Run All
```

## 运行测试

### Windows 开发模式

```powershell
cd D:\workspace\tauri\tauri\examples\api
cargo tauri dev
# 在 app 中点击 Tests → Run All
```

或添加 URL 参数自动触发：
```
http://localhost:1420/?autotest=true
```

### ohos 设备测试

使用 ohos-build skill：

```powershell
# 设置 autotest 标志
$env:VITE_AUTOTEST="true"

# 禁用 hvigorfile.ts 中的 tauriPlugin（见 SKILL.md）
# 运行构建脚本
& "C:\Program Files (x86)\Git\bin\bash.exe" "D:\workspace\tauri\tauri\.claude\skills\ohos-build\scripts\build-ohos.sh"

# 签名安装
& "C:\Program Files (x86)\Git\bin\bash.exe" "D:\workspace\tauri\tauri\.claude\skills\ohos-build\scripts\sign-and-install.sh"

# 拉取报告
hdc file recv /data/app/el2/100/base/com.tauri.api/cache/test-report.json examples/api/test-report.json
```

## 测试报告格式

```json
{
  "timestamp": "2026-05-11T14:30:00Z",
  "total": 25,
  "passed": 20,
  "failed": 5,
  "skipped": 0,
  "results": [
    {
      "name": "@tauri-apps/api/core.invoke",
      "category": "auto",
      "status": "pass",
      "duration": 14
    },
    {
      "name": "@tauri-apps/plugin-fs.mkdir+...",
      "category": "side-effect",
      "status": "fail",
      "duration": 5004,
      "error": "Timeout after 5000ms"
    }
  ]
}
```

## 常见问题

### Plugin command 被 removeUnusedCommands 移除

检查 `capabilities/run-app.json` 是否包含对应权限。

### HTTP scope 限制

`plugin-http` fetch 需要在 capabilities 中声明 URL：
```json
{
  "identifier": "http:default",
  "allow": [{ "url": "https://www.example.com/*" }]
}
```

### Plugin 在 ohos 上编译失败

Cargo.toml 中用 `cfg(not(target_env = "ohos"))` 排除，lib.rs 中条件注册。测试保留，失败即为待适配项。

### 动态 import 失败

确保 `plugins-workspace` 已执行 `pnpm build`，`package.json` 使用 `file:` 协议。