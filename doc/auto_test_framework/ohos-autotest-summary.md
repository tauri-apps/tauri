# Tauri OpenHarmony 自动化前端测试方案

## 概述

为 Tauri OpenHarmony 适配工作建立了一套自动化前端测试框架，用于验证 Tauri API 和官方 plugins 在 ohos 平台上的可用性。

## 完成的工作

### 1. 测试引擎 (`src/lib/test-runner.ts`)

实现了一个轻量级测试运行器：
- 定义 TestCase 接口（name, category, fn）
- 支持三种测试类别：auto（自动验证）、side-effect（有副作用但可验证）、manual（需人工确认）
- **新增 5 秒超时机制**：防止未实现的 API 卡死测试流程
- 自动跳过 manual 类测试
- 收集测试结果并生成 JSON 报告

### 2. 核心 API 测试 (`src/lib/tests/core.ts`)

覆盖 `@tauri-apps/api` 的所有前端函数：
- app: getVersion
- core: invoke, Channel, Resource
- event: emit+listen, once (含 UnlistenFn 验证)
- window: getCurrentWindow, isFocused, onFocusChanged, currentMonitor
- webview: getCurrentWebview
- path: appCacheDir
- 全局对象: `__TAURI_INTERNALS__`, `__TAURI__`

### 3. Plugin 测试 (`src/lib/tests/plugins.ts`)

覆盖所有官方 plugins 的前端 API：
- plugin-os: platform
- plugin-log: trace, debug, info, warn, error
- plugin-http: fetch
- plugin-fs: mkdir, writeFile, stat, readFile, exists, readDir, removeFile, removeDir
- plugin-autostart: enable, isEnabled, disable
- plugin-clipboard-manager: writeText+readText, writeImage
- plugin-dialog: open, save, confirm, message (manual)
- plugin-shell: open (manual)
- plugin-process: relaunch (manual)
- plugin-notification: sendNotification (manual)
- plugin-updater: check (manual)

### 4. TestRunner UI (`src/views/TestRunner.svelte`)

在 demo app 中新增 Tests 页面：
- Run All / Run Auto / Run Side-Effect 按钮
- 实时显示测试进度和结果
- 显示通过/失败/跳过统计

### 5. 自动测试触发机制

两种触发方式：
- **URL param**: `?autotest=true` — 用于 Windows dev 模式
- **Vite env**: `VITE_AUTOTEST=true` — 用于 ohos 生产构建

检测到后自动执行所有 auto + side-effect 类测试，完成后调用 `write_test_report` Rust command 将 JSON 报告写入设备缓存目录。

### 6. 测试报告持久化

- Rust command `write_test_report` 将 JSON 报告写入设备
- ohos 路径：`/data/app/el2/100/base/com.tauri.api/cache/test-report.json`
- 通过 `hdc file recv` 拉取报告到主机

### 7. ohos-build Skill

完整的构建流程脚本：
- `env.sh`：环境变量配置（CC、JAVA_HOME、linker 等）
- `build-ohos.sh`：前端构建 → Rust 编译 → hvigorw 打包
- `sign-and-install.sh`：签名 → 卸载旧版 → 安装 → 启动
- `run-tests.sh`：一键测试流程

## 技术要点

### 超时机制

为防止未实现的 API 永久阻塞，测试引擎添加 5 秒超时：
```typescript
const TEST_TIMEOUT_MS = 5000;
await withTimeout(test.fn(), TEST_TIMEOUT_MS);
```

### hvigor TCP 回调问题

`cargo tauri ohos build` 内部的 tauriPlugin 需要 TCP 回调，Windows 上会失败。
解决方案：手动禁用 `hvigorfile.ts` 中的 tauriPlugin，直接调用 hvigorw.bat。

### Rust Linker 配置

通过环境变量配置 ohos clang linker：
```bash
export CC_aarch64_unknown_linux_ohos="D:\app\DevEco-Studio\sdk\default\openharmony\native\llvm\bin\clang.exe"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER="..."
```

## 修改的文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `examples/api/src/lib/test-runner.ts` | 新增 | 测试引擎（含超时机制） |
| `examples/api/src/lib/tests/core.ts` | 新增 | 核心 API 测试 |
| `examples/api/src/lib/tests/plugins.ts` | 新增 | Plugin 测试 |
| `examples/api/src/views/TestRunner.svelte` | 新增 | 测试 UI |
| `examples/api/src/App.svelte` | 修改 | 注册 Tests view + autotest 触发 |
| `examples/api/src-tauri/src/cmd.rs` | 修改 | 添加 write_test_report command |
| `examples/api/src-tauri/src/lib.rs` | 修改 | 注册 plugins + command |
| `examples/api/src-tauri/Cargo.toml` | 修改 | 添加 plugin 依赖 |
| `examples/api/package.json` | 修改 | 添加 plugin JS 依赖 |
| `examples/api/src-tauri/capabilities/run-app.json` | 修改 | 添加权限 |
| `.claude/skills/ohos-build/*` | 新增/修改 | 构建脚本和 skill 文档 |

## 验证策略

1. **Windows 先行验证**：所有测试在 Windows desktop 上通过，证明测试用例正确
2. **ohos 对比验证**：同一套测试部署到 ohos 设备，失败项即为尚未适配的 API

## 后续工作

- 修复测试报告写入路径问题（当前报告未能正确写入）
- 完善自动化脚本，使 run-tests.sh 能一键完成全部流程
- 分析 ohos 测试结果，确定需要适配的 API