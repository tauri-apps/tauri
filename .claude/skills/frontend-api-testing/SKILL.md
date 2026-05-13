---
name: frontend-api-testing
description: Tauri 前端 API 自动化测试开发技能。使用场景：(1) 为新的 Tauri API 或 plugin 编写前端测试用例，(2) 添加测试到 core.ts 或 plugins.ts，(3) 配置 plugin 的 JS/Rust 依赖和权限，(4) 验证测试在 Windows/ohos 平台运行，(5) 分析测试报告定位问题。
---

# Tauri 前端 API 测试开发

本技能指导 agent 为 Tauri API 和 plugins 编写增量前端自动化测试。

## 快速导航

| 任务 | 跳转 |
|------|------|
| 添加自动测试 | [添加自动测试](#添加自动测试) |
| 添加手动测试 | [添加手动测试](#添加手动测试) |
| 接入新 plugin | [接入新 plugin](#接入新-plugin) |
| 运行测试 | [运行测试](#运行测试) |
| 查看报告 | [测试报告](#测试报告) |
| 排查问题 | [常见问题](#常见问题) |

## 文件位置

```
examples/api/src/
├── lib/
│   ├── test-runner.ts          # 测试引擎（不要修改）
│   └── tests/
│       ├── core.ts             # @tauri-apps/api 测试
│       └── plugins.ts          # @tauri-apps/plugin-* 测试
├── views/
│   └── TestRunner.svelte       # Tests 视图（按钮 + 手动测试 UI）
└── App.svelte                  # autotest 触发 + 默认视图
```

## Tests 视图

打开 Tests 视图时会自动执行一次全部测试（`onMount(() => runAll())`）。视图顶部有 3 个手动触发按钮：

| 按钮 | 行为 |
|------|------|
| **Run All** | 运行所有 `auto` + `side-effect` 测试（`manual` 自动跳过） |
| **Run Auto** | 仅运行 `category: 'auto'` 测试 |
| **Run Side-Effect** | 仅运行 `category: 'side-effect'` 测试 |

测试完成后自动调用 `invoke('write_test_report', ...)` 将报告写入设备。

视图下方是手动测试按钮区域，用于验证 autotest 无法覆盖的语义（如 `isFocused` 在用户主动操作时必须为 `true`）。

### 测试类别

| 类别 | 适用场景 | 自动执行 |
|------|----------|----------|
| `auto` | 纯函数调用，有返回值可断言 | ✓ |
| `side-effect` | 有副作用但可程序验证（fs、clipboard） | ✓ |
| `manual` | 需人工确认（dialog、notification） | ✗（跳过） |

## 添加自动测试

适用于可程序化断言的 API（返回值可验证、无需用户交互）。

1. 在 `core.ts`（核心 API）或 `plugins.ts`（plugin）添加 TestCase
2. 选择 category：纯读取用 `auto`，有副作用用 `side-effect`
3. 在 `fn()` 中调用 API 并用 `assert()` 验证
4. 如果是新 plugin，先完成 [接入新 plugin](#接入新-plugin) 的配置

**核心 API**：静态 import
```typescript
import { currentMonitor } from '@tauri-apps/api/window';

{
  name: '@tauri-apps/api/window.currentMonitor',
  category: 'auto',
  async fn() {
    const monitor = await currentMonitor();
    assert(monitor !== null, 'currentMonitor returned null');
    assert(monitor.size.width > 0, 'width should be positive');
  },
},
```

**Plugin**：动态 import（避免加载失败影响其他测试）
```typescript
{
  name: '@tauri-apps/plugin-fs.mkdir',
  category: 'side-effect',
  async fn() {
    const { mkdir } = await import('@tauri-apps/plugin-fs');
    await mkdir('test-dir', { baseDir: 1 });
  },
},
```

## 添加手动测试

适用于返回值依赖用户交互状态、或需要人工观察确认的 API。触发条件：

- 返回值依赖交互状态（焦点、前后台）
- 需要人工观察（UI 弹窗、通知）
- autotest 只能验证类型/非空，无法验证语义

步骤：

1. 在 `TestRunner.svelte` 中添加 `$state` 变量保存结果
2. 编写 `async function manualXxx()` handler
3. 在 Manual Tests 区域添加 `<button>` 绑定 handler
4. 结果自动显示在按钮下方 + Console

```typescript
let myResult = $state('');

async function manualMyApi() {
  const value = await someApi();
  const ok = value === expectedValue;
  myResult = `someApi() → ${value} ${ok ? '[OK]' : '[UNEXPECTED]'}`;
  onMessage(myResult);
}
```

```svelte
<button class="btn" onclick={manualMyApi}>My API (should be X)</button>
```

按钮文案建议包含预期结果（如 `isFocused (should be true)`），方便测试人员判断。

## 接入新 plugin

除了添加 TestCase，还需配置依赖和权限。

**1. Rust 依赖** — `examples/api/src-tauri/Cargo.toml`:
```toml
tauri-plugin-xxx = { path = "../../../../plugins-workspace/plugins/xxx" }
```

若 plugin 不支持 ohos：
```toml
[target.'cfg(not(target_env = "ohos"))'.dependencies]
tauri-plugin-xxx = { path = "../../../../plugins-workspace/plugins/xxx" }
```

**2. 注册 plugin** — `examples/api/src-tauri/src/lib.rs`:
```rust
#[cfg(not(target_env = "ohos"))]
let builder = builder.plugin(tauri_plugin_xxx::init());
```

**3. JS 依赖** — `examples/api/package.json`:
```json
"@tauri-apps/plugin-xxx": "file:../../../plugins-workspace/plugins/xxx"
```

**4. 权限** — `examples/api/src-tauri/capabilities/run-app.json`:
```json
"xxx:default"
```

## 约定

### 命名规范

- 核心 API：`@tauri-apps/api/<模块>.<函数名>`
- Plugin：`@tauri-apps/plugin-<名称>.<函数名>`
- 多函数组合：`@tauri-apps/plugin-fs.mkdir+writeFile+readFile`

### 断言

```typescript
function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

assert(typeof version === 'string', `expected string, got ${typeof version}`);
assert(result === expected, `mismatch: "${result}" vs "${expected}"`);
```

## 运行测试

### Windows

```powershell
cd D:\workspace\tauri\tauri\examples\api
pnpm build
cargo tauri dev
```

打开后默认进入 Tests 视图并自动执行一轮测试。也可通过 URL 参数触发：`http://localhost:1420/?autotest=true`

### ohos 设备

使用 `ohos-build` skill 构建并运行，详见该 skill 的 SKILL.md。

## 测试报告

```json
{
  "timestamp": "2026-05-13T01:50:30Z",
  "total": 33,
  "passed": 20,
  "failed": 5,
  "skipped": 8,
  "results": [
    { "name": "@tauri-apps/api/core.invoke", "status": "pass", "duration": 14 },
    { "name": "@tauri-apps/plugin-fs.mkdir", "status": "fail", "error": "..." }
  ]
}
```

Windows：Console 面板实时输出；ohos：写入设备 `test-report.json`。

## 常见问题

**Plugin command 未注册** — 检查 `capabilities/run-app.json` 是否包含对应权限。

**HTTP scope 限制** — `plugin-http` fetch 需声明 URL scope：
```json
{ "identifier": "http:default", "allow": [{ "url": "https://www.example.com/*" }] }
```

**Plugin 在 ohos 编译失败** — Cargo.toml 用 `cfg(not(target_env = "ohos"))` 排除。测试保留，失败即为待适配项。

**动态 import 失败** — 确保 `plugins-workspace` 已执行 `pnpm build`。

## 参考资料

- [test-template.md](references/test-template.md) - 测试用例模板
