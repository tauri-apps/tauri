# Tauri OHOS API 适配进度追踪

> 更新时间: 2026-05-12 17:28
> 状态: ✅ 全部完成

---

## 总体进度

| API | 状态 | 完成度 | 备注 |
|-----|------|--------|------|
| getCurrentWindow() | ✅ 完成 | 100% | 通用 API |
| getCurrentWebview() | ✅ 完成 | 100% | 通用 API |
| isFocused() | ✅ 完成 | 100% | 测试通过 |
| onFocusChanged() | ✅ 完成 | 100% | 测试通过 |
| currentMonitor() | ✅ 完成 | 100% | 测试通过 |
| appCacheDir() | ✅ 完成 | 100% | 动态获取路径，测试通过 |

**总完成度: 100%** 🎉

---

## 底层依赖状态

| 层级 | Monitor API | Path API | 状态 |
|------|-------------|----------|------|
| tao-oh | ✅ 已实现 | - | 无需修改 |
| wry-oh | - | ✅ 已实现 | 无需修改 |
| openharmony-ability | ✅ 已实现 | ✅ 已提供 | 无需修改 |
| tauri-runtime-wry | ✅ 已实现 | - | 无需修改 |
| tauri window plugin | ✅ 已修改 | - | 已完成 |
| tauri path plugin | - | ✅ 已创建 | 已完成 |
| @ohos-rs/ability | - | ✅ 升级到 0.4.0-beta.7 | 使用 NativeAbility |

---

## 任务清单

### currentMonitor 适配 [P0] ✅ 已完成

- [x] 修改 plugin.rs: 创建 `desktop_commands` 模块 (cfg: `any(desktop, target_env = "ohos")`)
- [x] 分离 `desktop_only_commands` 模块 (仅 desktop 平台 API)
- [x] 注册 `current_monitor`, `primary_monitor`, `available_monitors` 命令
- [x] 编译验证 ✅
- [x] 运行测试验证 ✅ (pass)

**实际工作量**: 0.5 天

---

### appCacheDir 适配 [P1] ✅ 已完成

- [x] 创建 `path/ohos.rs` 文件 (OHOS PathResolver 实现)
- [x] 修改 `ohos.rs`: 使用动态获取的 base_path (非硬编码)
- [x] 修改 `path/mod.rs` 条件编译 (添加 `target_env = "ohos"` 分支)
- [x] 修改 `path/plugin.rs` setup 函数 (OHOS 分支初始化)
- [x] 修改 `ohos.rs` (tauri crate): 添加 BASE_PATH 和 MODULE_NAME 静态变量
- [x] 修改 `app.rs`: 在 APP 被 take() 前保存路径信息
- [x] 升级 `@ohos-rs/ability` 到 0.4.0-beta.7
- [x] 更新模板: 使用 NativeAbility 替代 RustAbility
- [x] 编译验证 ✅
- [x] 运行测试验证 ✅ (pass)

**实际工作量**: 1 天

---

## 里程碑

| 里程碑 | 预计日期 | 状态 |
|--------|----------|------|
| 设计文档完成 | 2026-05-12 | ✅ 完成 |
| currentMonitor 可用 | 2026-05-13 | ✅ 提前完成 |
| appCacheDir 可用 | 2026-05-14 | ✅ 提前完成 |
| 所有 API 测试通过 | 2026-05-15 | ✅ 提前完成 |

---

## 每日更新日志

### 2026-05-12 (下午 - 第二轮修复)

**问题发现**:
- 初次测试 `appCacheDir` 失败，错误 `unknown path`
- 原因 1: `ohos.rs` 使用硬编码路径，不符合通用实现原则
- 原因 2: `APP` 在 `app.rs:2299` 被 `.take()` 取走，之后 `base_path()` 返回 None
- 原因 3: 旧版 `@ohos-rs/ability@0.4.0-beta.0` 的 `RustAbility.init()` 不传入 context

**解决方案**:
- 在 `crates/tauri/src/ohos.rs` 添加静态变量:
  - `BASE_PATH: OnceLock<Option<String>>` - 存储 base_path
  - `MODULE_NAME: OnceLock<Option<String>>` - 存储 module_name
- 在 `app.rs` 中 APP 被 take() 前保存路径信息
- 修改 `path/ohos.rs` 使用 `BASE_PATH` 动态获取路径
- 升级 `@ohos-rs/ability` 到 0.4.0-beta.7
- 更新模板和示例使用 `NativeAbility` (传入 AbilityInitContext)

**代码变更**:
- `crates/tauri/src/ohos.rs`: 添加 BASE_PATH, MODULE_NAME 静态变量
- `crates/tauri/src/app.rs`: 在 take() 前保存路径信息
- `crates/tauri/src/path/ohos.rs`: 使用动态路径
- `crates/tauri-cli/templates/mobile/open-harmony/entry/oh-package.json5`: 升级 ability 版本
- `crates/tauri-cli/templates/mobile/open-harmony/entry/src/main/ets/entryability/EntryAbility.ets`: 使用 NativeAbility
- `examples/api/src-tauri/gen/ohos/entry/oh-package.json5`: 升级 ability 版本
- `examples/api/src-tauri/gen/ohos/entry/src/main/ets/entryability/EntryAbility.ets`: 使用 NativeAbility

**测试结果**:
- 通过: 20/25
- currentMonitor: ✅ pass
- appCacheDir: ✅ pass (动态路径)
- 失败: 5/25 (与本轮无关)

---

### 2026-05-12 (下午 - 第一轮实现)

**完成**:
- 实现 currentMonitor 适配 (修改 window/plugin.rs)
- 实现 appCacheDir 适配 (创建 path/ohos.rs, 修改 mod.rs, plugin.rs)
- 编译验证通过
- 运行测试: appCacheDir 失败 (unknown path)

**代码变更**:
- `crates/tauri/src/window/plugin.rs`:
  - 新增 `desktop_commands` 模块 (OHOS 可用: current_monitor, primary_monitor, available_monitors)
  - 新增 `desktop_only_commands` 模块 (仅 desktop: minimize, maximize 等窗口操作)
- `crates/tauri/src/path/ohos.rs`: 新建 OHOS PathResolver 实现 (初版硬编码)
- `crates/tauri/src/path/mod.rs`: 添加 OHOS 条件编译分支
- `crates/tauri/src/path/plugin.rs`: 添加 OHOS 初始化分支

---

### 2026-05-12 (上午)

**完成**:
- 运行 run-tests.sh 获取测试报告 (通过 18/25)
- 分析 api-to-support.md 需求 (6 个 API)
- 调研 tao-oh 源码 (MonitorHandle 已实现)
- 调研 wry-oh 源码 (WebView 已实现)
- 调研 openharmony-ability 源码 (路径信息已提供)
- 调研 tauri-runtime-wry 源码 (mobile 已包含 OHOS)
- 调研 tauri path 模块 (缺少 OHOS 实现)
- 编写设计文档
- 编写进度文档

---

## 技术要点

### 路径动态获取机制

```
ArkTS: NativeAbility.onCreate()
    ↓
context.filesDir → AbilityInitContext.basePath
    ↓
Rust: openharmony_ability::OpenHarmonyApp.set_init_context()
    ↓
tauri::ohos::BASE_PATH.set(app.base_path())  [APP 被 take 前]
    ↓
PathResolver::base_path() → BASE_PATH.get()
    ↓
返回动态路径 (如 /data/storage/el2/base/files)
```

### 关键代码片段

**ohos.rs**:
```rust
pub static BASE_PATH: OnceLock<Option<String>> = OnceLock::new();
pub static MODULE_NAME: OnceLock<Option<String>> = OnceLock::new();
```

**app.rs**:
```rust
#[cfg(target_env = "ohos")]
app: {
  let ohos_app = crate::ohos::APP.lock().unwrap().take()...;
  crate::ohos::BASE_PATH.set(ohos_app.base_path()).ok();
  crate::ohos::MODULE_NAME.set(ohos_app.module_name()).ok();
  ohos_app
},
```

**path/ohos.rs**:
```rust
fn base_path(&self) -> Result<PathBuf> {
  crate::ohos::BASE_PATH
    .get()
    .and_then(|p| p.as_ref())
    .map(|p| PathBuf::from(p))
    .ok_or(Error::UnknownPath)
}
```

---

## 风险与注意事项

| 风险 | 影响 | 缓解措施 | 状态 |
|------|------|----------|------|
| desktop commands 中部分 API 在 OHOS 无意义 | 可能有 warn 日志 | 仅支持需要的 API | ✅ 已分离 |
| 硬编码路径不可用 | 通用性问题 | 动态获取路径 | ✅ 已解决 |
| APP 被 take 后无法访问 | base_path 返回 None | 提前保存到静态变量 | ✅ 已解决 |
| 旧版 ability 不传入 context | basePath 为空 | 升级到 NativeAbility | ✅ 已解决 |

---

## 遗留问题 (非本轮范围)

| 问题 | 影响 | 状态 |
|------|------|------|
| core.Channel IPC 通道消息丢失 | 消息数量不匹配 | ❌ 未解决 |
| tauriPlugin crash | 需禁用 hvigorfile.ts | ⚠️ 临时方案 |
| localStorage null | WebView DOM 存储 | ❌ 未解决 |
| plugin-http fetch 失败 | HTTP 请求问题 | ❌ 未解决 |

---

## 文件修改汇总

| 文件 | 修改类型 | 说明 |
|------|----------|------|
| `crates/tauri/src/window/plugin.rs` | 修改 | 分离 desktop_commands 和 desktop_only_commands |
| `crates/tauri/src/path/ohos.rs` | 新建 + 修改 | PathResolver (动态路径) |
| `crates/tauri/src/path/mod.rs` | 修改 | OHOS 条件编译 |
| `crates/tauri/src/path/plugin.rs` | 修改 | OHOS setup 分支 |
| `crates/tauri/src/ohos.rs` | 修改 | 添加 BASE_PATH, MODULE_NAME |
| `crates/tauri/src/app.rs` | 修改 | 保存路径信息 |
| `crates/tauri-cli/templates/.../oh-package.json5` | 修改 | ability 版本升级 |
| `crates/tauri-cli/templates/.../EntryAbility.ets` | 修改 | 使用 NativeAbility |
| `examples/api/.../oh-package.json5` | 修改 | ability 版本升级 |
| `examples/api/.../EntryAbility.ets` | 修改 | 使用 NativeAbility |

---

## 下一步建议

本轮适配已全部完成。后续可考虑:

1. **修复 core.Channel IPC 问题** - 需排查底层通信机制
2. **适配其他 Tauri API** - 如 dialog, notification 等 plugin
3. **完善 path API** - 其他路径类型已实现但未测试
4. **清理编译 warnings** - 添加文档注释
5. **将 NativeAbility 更新写入 CLI 模板** - 已完成，需提交