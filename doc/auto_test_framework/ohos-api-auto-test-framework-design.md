# Plan: Tauri OpenHarmony API 自动化测试方案

## Context

我们在给 Tauri 适配 OpenHarmony。`examples/api` 是一个 Svelte demo app，通过按钮手动测试各种 Tauri API。现在需要一个自动化测试方案来验证 API 在 ohos 上是否正常工作。

当前 demo 只测试了核心 `@tauri-apps/api`（invoke、events、channels、window），但 `api_list.md` 列出了大量需要支持的 plugin（http、fs、dialog、os、shell、clipboard、log、notification、process 等），这些 plugin 尚未集成到 demo 中。

## 方案设计

### 整体架构

在 examples/api app 中新增一个 **TestRunner** view，作为自动化测试入口：

```
┌─────────────────────────────────────────┐
│  App (Svelte)                           │
│  ┌─────────┐  ┌──────────────────────┐  │
│  │ Sidebar │  │ TestRunner View       │  │
│  │ ...     │  │  [Run All] [Run Cat]  │  │
│  │ Tests ← │  │  ✓ core.invoke       │  │
│  │         │  │  ✓ event.emit        │  │
│  │         │  │  ✗ fs.writeFile      │  │
│  │         │  │  ⊘ dialog.open (skip)│  │
│  └─────────┘  └──────────────────────┘  │
│  ┌──────────────────────────────────────┐│
│  │ Console (existing onMessage)         ││
│  └──────────────────────────────────────┘│
└─────────────────────────────────────────┘
```

### 测试分类

| 类别 | 说明 | 验证方式 |
|------|------|----------|
| **auto** | 可全自动验证（有明确返回值） | 断言返回值类型/内容 |
| **side-effect** | 有副作用但可程序验证 | 执行后检查状态（如 fs: write → read → compare） |
| **manual** | 需要人工确认（UI 弹窗、视觉效果） | 标记为 skip，单独手动测试 |

### 测试用例设计

**auto 类（可全自动）：**
- `@tauri-apps/api/app`: getVersion() → 返回非空字符串
- `@tauri-apps/api/core`: invoke('echo', {message}) → 返回相同 message
- `@tauri-apps/api/event`: emit() + listen() → listener 收到 payload
- `@tauri-apps/api/window`: getCurrentWindow() → 返回 window 对象
- `@tauri-apps/api/webview`: getCurrentWebview() → 返回 webview 对象
- `@tauri-apps/api/path`: appCacheDir() → 返回非空路径字符串
- `@tauri-apps/plugin-os`: platform() → 返回 "ohos" 或类似值
- `@tauri-apps/plugin-log`: info/warn/error() → 不抛异常即 pass
- `@tauri-apps/plugin-http`: fetch(公开URL) → 返回 status 200

**side-effect 类（自动验证但有副作用）：**
- `@tauri-apps/plugin-fs`: mkdir → writeFile → stat → readFile → compare → removeFile → removeDir
- `@tauri-apps/plugin-autostart`: enable() → isEnabled() === true → disable() → isEnabled() === false
- `@tauri-apps/plugin-clipboard-manager`: writeImage() → 不抛异常

**manual 类（跳过自动测试）：**
- `@tauri-apps/plugin-dialog`: open(), save(), confirm(), message() — 需要 UI 交互
- `@tauri-apps/plugin-shell`: open() — 打开外部应用
- `@tauri-apps/plugin-process`: relaunch() — 会重启 app
- `@tauri-apps/plugin-updater`: check() — 需要服务端配合
- `@tauri-apps/plugin-notification`: 需要系统权限和视觉确认

### 结果输出

测试结果需要从设备传回主机，方案：

1. **前端生成 JSON 报告**，通过 Rust command 写入设备文件系统
2. **主机用 `hdc file recv` 拉取报告**
3. 报告格式：
   ```json
   {
     "timestamp": "2026-05-11T20:30:00",
     "total": 20, "passed": 18, "failed": 1, "skipped": 1,
     "results": [
       {"name": "core.invoke", "category": "auto", "status": "pass", "duration": 12},
       {"name": "fs.writeFile", "category": "side-effect", "status": "fail", "error": "Permission denied"},
       {"name": "dialog.open", "category": "manual", "status": "skip"}
     ]
   }
   ```

### 自动触发机制

为了让测试无需手动点击即可运行：
- App 启动时检查 URL query param: `?autotest=true`
- 如果检测到，自动切换到 TestRunner view 并执行所有 auto + side-effect 测试
- 测试完成后将报告写入设备文件

这样 CI/脚本可以：
```bash
# 安装 app 后启动并自动运行测试
hdc shell aa start -b com.tauri.api -a EntryAbility --uri "?autotest=true"
sleep 10
hdc file recv /data/app/el2/100/base/com.tauri.api/files/test-report.json ./report.json
```

## 实施步骤

### Phase 1: 基础框架

1. **新增 `src/views/TestRunner.svelte`** — 测试运行器 UI
2. **新增 `src/lib/test-runner.ts`** — 测试引擎（定义测试、执行、收集结果）
3. **新增 `src/lib/tests/core.ts`** — 核心 API 测试用例
4. **在 App.svelte 中注册 TestRunner view**
5. **新增 Rust command `write_test_report`** — 将 JSON 报告写入设备文件
6. **更新 capabilities** — 添加新 command 权限

### Phase 2: 集成 Plugins

Plugins 使用本地仓库：`D:\workspace\tauri\plugins-workspace`

1. **Rust 依赖**（`examples/api/src-tauri/Cargo.toml`）使用相对路径：
   ```toml
   tauri-plugin-fs = { path = "../../../../plugins-workspace/plugins/fs" }
   tauri-plugin-http = { path = "../../../../plugins-workspace/plugins/http" }
   tauri-plugin-dialog = { path = "../../../../plugins-workspace/plugins/dialog" }
   tauri-plugin-os = { path = "../../../../plugins-workspace/plugins/os" }
   tauri-plugin-shell = { path = "../../../../plugins-workspace/plugins/shell" }
   tauri-plugin-clipboard-manager = { path = "../../../../plugins-workspace/plugins/clipboard-manager" }
   tauri-plugin-log = { path = "../../../../plugins-workspace/plugins/log" }
   tauri-plugin-notification = { path = "../../../../plugins-workspace/plugins/notification" }
   tauri-plugin-process = { path = "../../../../plugins-workspace/plugins/process" }
   tauri-plugin-autostart = { path = "../../../../plugins-workspace/plugins/autostart" }
   ```

2. **JS 依赖**（`examples/api/package.json`）— 使用 link 协议指向本地 plugin 目录
3. **注册 plugins**（`src-tauri/src/lib.rs`）
4. **添加 capabilities 权限**（`src-tauri/capabilities/run-app.json`）
5. **新增 `src/lib/tests/plugins.ts`** — 各 plugin 测试用例

### Phase 3: 自动触发 + 报告拉取

1. **实现 autotest query param 检测**
2. **新增 ohos-build skill 脚本 `run-tests.sh`** — 安装 → 启动(autotest) → 等待 → 拉取报告
3. **报告解析和展示**

## 关键文件

| 文件 | 作用 |
|------|------|
| `src/views/TestRunner.svelte` | 新增，测试 UI |
| `src/lib/test-runner.ts` | 新增，测试引擎 |
| `src/lib/tests/core.ts` | 新增，核心 API 测试 |
| `src/lib/tests/plugins.ts` | 新增（Phase 2），plugin 测试 |
| `src/App.svelte` | 修改，注册 TestRunner view |
| `src-tauri/src/cmd.rs` | 修改，添加 write_test_report command |
| `src-tauri/src/lib.rs` | 修改，注册新 command + plugins |
| `src-tauri/Cargo.toml` | 修改，添加 plugin 依赖 |
| `package.json` | 修改，添加 plugin 前端依赖 |
| `src-tauri/capabilities/run-app.json` | 修改，添加权限 |

## 验证方式

**双平台验证策略：**

1. **Windows desktop 验证（基准）：** `cargo tauri dev` 在 Windows 上运行，所有测试应全部通过。这证明测试用例本身是正确的。
2. **OpenHarmony 设备验证（目标）：** 部署到 ohos 设备运行同一套测试，失败的即为 ohos 尚未适配的 API。

具体步骤：
1. 编写完测试后，先 `cargo tauri dev` 在 Windows 上运行 app
2. 点击 TestRunner → Run All，确认全部 pass
3. `build-ohos.sh` → `sign-and-install.sh` 部署到设备
4. 在设备上运行测试，对比 Windows 结果，失败项即为 ohos 待适配清单
5. Phase 3 完成后：`run-tests.sh` 一键执行，自动输出报告 JSON 并对比

## 实现完成状态（2026-05-12）

### Phase 1-3 全部完成

| Phase | 内容 | 状态 |
|-------|------|------|
| **Phase 1** | TestRunner UI、test-runner.ts、tests/core.ts、write_test_report | ✓ 完成 |
| **Phase 2** | Plugin 集成、tests/plugins.ts、capabilities 配置 | ✓ 完成 |
| **Phase 3** | autotest 触发、run-tests.sh、报告拉取 | ✓ 完成 |

### 关键修复

1. **custom-protocol feature**: 必须在 Cargo.toml 中启用 `"custom-protocol"`，否则 tauri 认为是 dev 模式，尝试连接 localhost:1420
2. **panic hook**: 使用内部沙箱路径 `/data/storage/el2/base/cache`
3. **hvigorw**: JAVA_HOME/bin 必须加入 PATH
4. **hdc file recv**: 使用 cmd.exe 避免 Git Bash 路径问题

### 测试数量

- core.ts: 17 个测试（app、core、event、window、webview、path、global objects）
- plugins.ts: 15 个测试（os、log、http、fs、autostart、clipboard、manual 类）

### 相关文档

| 文档 | 内容 |
|------|------|
| `doc/frontend-test-guide.md` | 测试编写指南 |
| `doc/ohos-autotest-summary.md` | ohos 测试总结 |
| `doc/ohos-troubleshooting.md` | 故障排查指南 |
| `.claude/skills/ohos-build/SKILL.md` | 构建流程说明 |
| `.claude/skills/frontend-api-testing/SKILL.md` | 测试开发技能 |
