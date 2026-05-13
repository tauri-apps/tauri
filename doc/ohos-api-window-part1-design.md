# Tauri OHOS API 适配设计文档

> 创建时间: 2026-05-12
> 更新时间: 2026-05-12 17:28
> 目标: 适配 api-to-support.md 中列出的所有 API
> 状态: ✅ 已完成

---

## 一、需要支持的 API 清单

| 模块 | API | 实现状态 | 测试状态 |
|------|-----|----------|----------|
| @tauri-apps/api/window | getCurrentWindow() | ✅ 通用 API | ✅ pass |
| @tauri-apps/api/window | isFocused() | ✅ 通用 API | ✅ pass |
| @tauri-apps/api/window | onFocusChanged() | ✅ 通用 API | ✅ pass |
| @tauri-apps/api/window | currentMonitor() | ✅ 已适配 | ✅ pass |
| @tauri-apps/api/webview | getCurrentWebview() | ✅ 通用 API | ✅ pass |
| @tauri-apps/api/path | appCacheDir() | ✅ 已适配 | ✅ pass |

---

## 二、底层依赖调研结果

### 2.1 tao-oh (窗口/显示器)

**文件**: `tao/src/platform_impl/ohos/mod.rs`

| API | 实现 | 代码位置 |
|-----|------|----------|
| MonitorHandle | ✅ 已实现 | 第 830-870 行 |
| Window.current_monitor() | ✅ 已实现 | 第 807-810 行 |
| EventLoopWindowTarget.primary_monitor() | ✅ 已实现 | 第 482-486 行 |
| EventLoopWindowTarget.available_monitors() | ✅ 已实现 | 第 476-480 行 |

**结论**: tao 层已完整实现，无需修改。

---

### 2.2 wry-oh (WebView)

**文件**: `wry/src/ohos/mod.rs`

WebView 已实现，通过 `openharmony-ability::WebViewBuilder` 封装。

**结论**: wry 层已实现，无需修改。

---

### 2.3 openharmony-ability (原生能力)

**文件**: `openharmony-ability/crates/ability/src/app.rs`

| API | 状态 |
|-----|------|
| OpenHarmonyApp.content_rect() | ✅ 已实现 |
| OpenHarmonyApp.scale() | ✅ 已实现 |
| OpenHarmonyApp.base_path() | ✅ 已实现 |
| OpenHarmonyApp.module_name() | ✅ 已实现 |

**AbilityInitContext**:
```rust
#[napi(object)]
pub struct AbilityInitContext {
    pub base_path: Option<String>,
    pub pref_path: Option<String>,
    pub module_name: Option<String>,
}
```

---

### 2.4 tauri-runtime-wry

**build.rs**:
```rust
let mobile = target_os == "ios" || target_os == "android" || target_env == "ohos";
alias("desktop", !mobile);
alias("mobile", mobile);
```

**结论**: OHOS 属于 mobile 平台，runtime 层已支持。

---

### 2.5 @ohos-rs/ability (ArkTS 层)

**版本升级**: 0.4.0-beta.0 → 0.4.0-beta.7

| 版本 | 类 | init 参数 | basePath |
|------|-----|-----------|----------|
| 0.4.0-beta.0 | RustAbility | 无参数 | ❌ 空值 |
| 0.4.0-beta.7 | NativeAbility | AbilityInitContext | ✅ 动态获取 |

**NativeAbility 关键代码** (`native_ability/src/main/ets/ability/NativeAbility.ets`):
```typescript
protected createInitContext(moduleName: string): AbilityInitContext {
  const context = this.context as common.UIAbilityContext;
  return {
    basePath: context?.filesDir ?? "",
    prefPath: context?.filesDir ?? "",
    moduleName,
    // ...
  };
}

// onCreate 中调用
const lifecycle: ApplicationLifecycle = module.init(this.createInitContext(moduleName));
```

---

## 三、currentMonitor 适配设计

### 3.1 调用链路

```
前端: invoke('plugin:window|current_monitor')
    ↓
plugin.rs: desktop_commands::current_monitor [✅ #[cfg(any(desktop, target_env = "ohos"))]
    ↓
window/mod.rs: Window::current_monitor() [✅ 通用实现]
    ↓
runtime-wry: Dispatcher::current_monitor() [✅ 已实现]
    ↓
tao-oh: MonitorHandle::new(app) [✅ 已实现]
    ↓
openharmony-ability: app.content_rect(), app.scale() [✅ 已实现]
```

### 3.2 实际实现

**修改文件**: `tauri/crates/tauri/src/window/plugin.rs`

**改动 - 模块分离**:
```rust
// OHOS 可用的 monitor 命令
#[cfg(any(desktop, target_env = "ohos"))]
mod desktop_commands {
  use super::*;
  use crate::{command, Monitor};

  getter!(current_monitor, Option<Monitor>);
  getter!(primary_monitor, Option<Monitor>);
  getter!(available_monitors, Vec<Monitor>);
}

// 仅 desktop 可用的窗口操作命令
#[cfg(desktop)]
mod desktop_only_commands {
  // minimize, maximize, fullscreen 等
}
```

**改动 - invoke_handler 注册**:
```rust
#[cfg(any(desktop, target_env = "ohos"))] desktop_commands::current_monitor,
#[cfg(any(desktop, target_env = "ohos"))] desktop_commands::primary_monitor,
#[cfg(any(desktop, target_env = "ohos"))] desktop_commands::available_monitors,
```

---

## 四、appCacheDir 适配设计

### 4.1 问题分析

**问题 1: 硬编码路径不可用**
- 初版设计使用硬编码 `/data/storage/el2/base`
- 不符合通用实现原则，需动态获取

**问题 2: APP 被 take() 后无法访问**
- `app.rs:2299` 使用 `.take()` 取走 `OpenHarmonyApp`
- 之后 `crate::ohos::APP` 变成 `None`
- `base_path()` 返回 `None`，导致 `unknown path` 错误

**问题 3: 旧版 ability 不传入 context**
- `@ohos-rs/ability@0.4.0-beta.0` 的 `RustAbility.init()` 无参数
- `basePath` 从未传入 Rust 层

### 4.2 解决方案

#### 方案 1: 静态变量存储路径信息

**修改文件**: `tauri/crates/tauri/src/ohos.rs`

```rust
use std::sync::{Mutex, OnceLock};

pub use openharmony_ability;
pub use openharmony_ability_derive;

pub static APP: Mutex<Option<openharmony_ability::OpenHarmonyApp>> = Mutex::new(None);

/// Stores the base path for OHOS app, initialized before APP is taken.
pub static BASE_PATH: OnceLock<Option<String>> = OnceLock::new();

/// Stores the module name for OHOS app, initialized before APP is taken.
pub static MODULE_NAME: OnceLock<Option<String>> = OnceLock::new();
```

#### 方案 2: 在 APP 被 take 前保存路径

**修改文件**: `tauri/crates/tauri/src/app.rs`

```rust
#[cfg(target_env = "ohos")]
app: {
  let ohos_app = crate::ohos::APP
    .lock()
    .unwrap()
    .take()
    .expect("OpenHarmony app instance not initialized");
  // 在 APP 被 take 前保存路径信息
  crate::ohos::BASE_PATH.set(ohos_app.base_path()).ok();
  crate::ohos::MODULE_NAME.set(ohos_app.module_name()).ok();
  ohos_app
},
```

#### 方案 3: PathResolver 使用动态路径

**新建文件**: `tauri/crates/tauri/src/path/ohos.rs`

```rust
use super::{Error, Result};
use crate::{AppHandle, Runtime};
use std::path::{Path, PathBuf};

/// The path resolver is a helper class for general and application-specific path APIs on OpenHarmony.
pub struct PathResolver<R: Runtime>(pub(crate) AppHandle<R>);

impl<R: Runtime> Clone for PathResolver<R> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<R: Runtime> PathResolver<R> {
  fn base_path(&self) -> Result<PathBuf> {
    crate::ohos::BASE_PATH
      .get()
      .and_then(|p| p.as_ref())
      .map(|p| PathBuf::from(p))
      .ok_or(Error::UnknownPath)
  }

  pub fn file_name(&self, path: &str) -> Option<String> {
    Path::new(path)
      .file_name()
      .map(|name| name.to_string_lossy().into_owned())
  }

  pub fn audio_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files").join("Audio"))
  }

  pub fn cache_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("cache"))
  }

  pub fn app_cache_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("cache"))
  }

  pub fn resource_dir(&self) -> Result<PathBuf> {
    let module_name = crate::ohos::MODULE_NAME
      .get()
      .and_then(|m| m.as_ref())
      .map(|m| m.clone())
      .unwrap_or_else(|| "entry".to_string());
    Ok(PathBuf::from("/data/storage/el1/base").join(module_name).join("assets"))
  }

  // ... 其他路径方法
}
```

#### 方案 4: 升级 @ohos-rs/ability

**模板修改**: `crates/tauri-cli/templates/mobile/open-harmony/entry/oh-package.json5`
```json
{
  "dependencies": {
    "@ohos-rs/ability": "0.4.0-beta.7"
  }
}
```

**模板修改**: `entry/src/main/ets/entryability/EntryAbility.ets`
```typescript
import { NativeAbility } from '@ohos-rs/ability'

export default class EntryAbility extends NativeAbility {
  public moduleName: string = "{{app.lib-name}}"
  public defaultPage: boolean = true;
  public mode: 'xcomponent' | 'webview' = 'webview'
}
```

### 4.3 路径动态获取机制

```
ArkTS: NativeAbility.onCreate()
    ↓
context.filesDir → AbilityInitContext.basePath
    ↓
Rust: openharmony_ability::AbilityInitContext::from_object()
    ↓
OpenHarmonyApp.set_init_context(init_context)
    ↓
tauri-macros: ability derive → init(context) 函数
    ↓
app.rs: BASE_PATH.set(app.base_path()) [APP 被 take 前]
    ↓
PathResolver::base_path() → BASE_PATH.get()
    ↓
返回动态路径 (如 /data/storage/el2/base/files)
```

---

## 五、文件修改清单

| 文件 | 修改内容 | 影响范围 |
|------|----------|----------|
| `crates/tauri/src/window/plugin.rs` | 分离 desktop_commands 和 desktop_only_commands | currentMonitor API |
| `crates/tauri/src/path/ohos.rs` | 新建，动态路径实现 | appCacheDir API |
| `crates/tauri/src/path/mod.rs` | OHOS 条件编译 | PathResolver |
| `crates/tauri/src/path/plugin.rs` | OHOS setup 分支 | PathPlugin 初始化 |
| `crates/tauri/src/ohos.rs` | 添加 BASE_PATH, MODULE_NAME | 路径存储 |
| `crates/tauri/src/app.rs` | 保存路径信息 | 路径初始化 |
| `crates/tauri-cli/templates/.../oh-package.json5` | ability 版本升级 | NativeAbility |
| `crates/tauri-cli/templates/.../EntryAbility.ets` | 使用 NativeAbility | context 传入 |

---

## 六、测试验证

```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/run-tests.sh
```

**测试结果** (2026-05-12):
- 通过: 20/25
- currentMonitor: ✅ pass
- appCacheDir: ✅ pass (动态路径)
- 失败: 5/25 (与本轮无关: core.Channel, plugin-http 等)

---

## 七、已知问题（非本次范围）

| 问题 | 影响 | 备注 |
|------|------|------|
| core.Channel IPC | 消息数量不匹配 | 需排查底层通信 |
| tauriPlugin crash | 需禁用 hvigorfile.ts | 临时方案 |
| localStorage null | WebView DOM 存储 | 非本次范围 |
| plugin-http fetch | HTTP 请求失败 | 非本次范围 |

---

## 八、设计原则

1. **不硬编码路径**: 所有路径应动态获取，适应不同设备和应用配置
2. **通用实现**: 代码应适用于所有 OHOS 环境，不依赖特定路径
3. **静态变量存储**: 在 APP 被 take 前保存必要信息
4. **使用最新 ability**: NativeAbility 正确传入 AbilityInitContext