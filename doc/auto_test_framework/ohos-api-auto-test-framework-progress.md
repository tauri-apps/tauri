# OpenHarmony Plugin 适配进展

日期: 2026-05-12

## 本轮工作总结

### 目标

让 Tauri `examples/api` 项目（含多个 plugin）能在 OpenHarmony (ohos) 目标上编译通过，为后续设备端测试打基础。

### 设计决策

**最终方案**: 保持 ohos 为 `mobile` 分类（与 android/ios 一致），对 PC 上需要的接口通过 `#[cfg(any(desktop, target_env = "ohos"))]` 选择性编译。

**原因**: 将 ohos 改为 desktop 会导致 `muda`（菜单）和 `tray_icon`（系统托盘）crate 被引入，这些依赖 GTK，无法在 ohos 上编译。

### 修改的文件

| Plugin | 文件 | 修改内容 |
|--------|------|----------|
| **fs** | `plugins/fs/src/lib.rs` | desktop module/export/manage 改为 `any(desktop, target_env = "ohos")` |
| **shell** | `plugins/shell/src/lib.rs` | PluginHandle/mobile_plugin_handle 限定为 android/ios；open() 方法分 desktop+ohos 和 android+ios |
| **clipboard-manager** | `plugins/clipboard-manager/src/lib.rs` | desktop/mobile 模块、export、init、cleanup 按同样模式调整 |
| **clipboard-manager** | `plugins/clipboard-manager/src/error.rs` | PluginInvoke variant 限定 android/ios；arboard::Error 转换限定 desktop+ohos |
| **clipboard-manager** | `plugins/clipboard-manager/src/commands.rs` | write_text 的两个版本分别用 desktop+ohos 和 not(desktop)+not(ohos) |
| **log** | `plugins/log/src/lib.rs` | Stdout/Stderr match arms 改为 `any(desktop, target_env = "ohos")` |
| **autostart** | `plugins/autostart/src/lib.rs` | appimage 分支限定 `all(linux, not(ohos))`；ohos 单独走 current_exe 路径 |

### cfg 模式总结

```rust
// 需要 desktop 代码路径的 ohos 接口
#[cfg(any(desktop, target_env = "ohos"))]

// 仅 android/ios 的 mobile 代码（排除 ohos）
#[cfg(any(target_os = "android", target_os = "ios"))]

// Linux 特有但排除 ohos
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
```

### 编译验证结果

| 目标 | 结果 |
|------|------|
| Windows desktop (`cargo check`) | 通过 |
| aarch64-unknown-linux-ohos (`cargo build --target`) | 通过 |
| hvigor assembleHap (打包 HAP) | 通过 |
| 签名 (hap-sign-tool.jar) | 通过 |
| 安装到设备 (hdc install) | 通过 |
| 运行 | 崩溃 (SIGABRT，Tauri 初始化阶段) |

### 设备端运行崩溃分析与修复

#### 调试方法

通过自定义 panic hook 将 panic 信息写入 app 沙箱目录：
```rust
#[cfg(target_env = "ohos")]
std::panic::set_hook(Box::new(|info| {
    let msg = format!("PANIC: {info}\n");
    let _ = std::fs::write("/data/storage/el2/base/cache/panic.log", &msg);
    eprintln!("{msg}");
}));
```

然后通过 `hdc file recv` 拉取 panic.log 查看具体错误。

#### 崩溃原因定位

通过逐个禁用 plugin 的方式缩小范围，最终定位到两个 plugin：

1. **tauri-plugin-log**: `PluginInitialization("log", "Operation not permitted (os error 1)")`
   - 原因: 默认 LogDir target 调用 `app_log_dir()` → `dirs::cache_dir()` → `$HOME/.cache`
   - ohos 上 `$HOME=/storage/Users/currentUser`，该路径对 app 只读
   - 修复: 使用 `.clear_targets().target(TargetKind::Stdout)` 只输出到标准输出

2. **tauri-plugin-http**: `PluginInitialization("http", "Operation not permitted (os error 1)")`
   - 原因: cookies 功能调用 `app.path().app_cache_dir()` → 同样走 `dirs::cache_dir()`
   - 修复: 暂时在 ohos 上排除 http plugin（`#[cfg(not(target_env = "ohos"))]`）

#### 根本原因

Tauri 的路径模块 (`crates/tauri/src/path/mod.rs`) 对 ohos 使用了 `desktop.rs` 实现（因为 `target_os != "android"`），而 desktop.rs 依赖 `dirs` crate，该 crate 通过 `$HOME` 环境变量推导路径。

ohos 环境特殊性：
- `$HOME = /storage/Users/currentUser`（只读，app 无写权限）
- App 实际可写沙箱: `/data/storage/el2/base/`（对应外部路径 `/data/app/el2/100/base/com.tauri.api/`）

**长期方案**: 需要为 ohos 实现专用的路径模块（类似 `android.rs`），使用 ohos 的 Context API 获取正确的沙箱路径。

#### 最终可用 plugin 列表

| Plugin | ohos 状态 | 备注 |
|--------|-----------|------|
| tauri-plugin-log | 可用 | 必须用 Stdout target |
| tauri-plugin-fs | 可用 | |
| tauri-plugin-os | 可用 | |
| tauri-plugin-shell | 可用 | |
| tauri-plugin-process | 可用 | |
| tauri-plugin-sample | 排除 | ohos 无 mobile plugin handle |
| tauri-plugin-notification | 排除 | 需要 ohos 权限适配 |
| tauri-plugin-dialog | 排除 | 需要 ohos UI 适配 |
| tauri-plugin-http | 排除 | app_cache_dir 不可写 |
| tauri-plugin-clipboard-manager | 排除 | 需要进一步测试 |
| tauri-plugin-autostart | 排除 | 依赖 desktop 特性 |

### 下一步

1. **实现 ohos 路径模块**: 在 `crates/tauri/src/path/` 新增 ohos 实现，使用正确的沙箱路径
2. **逐步恢复 plugin**: 路径问题修复后，http 等 plugin 应能正常工作
3. **前端测试**: 运行 TestRunner 自动化测试验证 API 可用性
4. **适配更多 plugin**: notification、dialog 等需要 ohos 原生 API 对接

## 自动化测试框架搭建（2026-05-12）

### 目标

为 ohos 适配工作建立自动化前端测试框架，验证 Tauri API 在 ohos 平台上的可用性。

### 完成的组件

1. **test-runner.ts**: 测试引擎，含 5 秒超时机制防止卡死
2. **tests/core.ts**: 核心 API 测试用例（17 个测试）
3. **tests/plugins.ts**: Plugin 测试用例（15 个测试）
4. **TestRunner.svelte**: 测试 UI，显示进度和结果
5. **write_test_report command**: Rust 端报告持久化到设备文件系统
6. **ohos-build skill**: 完整构建脚本套件（env.sh, build-ohos.sh, sign-and-install.sh, run-tests.sh）
7. **frontend-test-guide.md**: 测试编写指南文档
8. **frontend-api-testing skill**: 测试开发指导技能

### 技术要点

- **超时机制**: 5 秒超时，防止未实现 API 永久阻塞
- **hvigor TCP 问题**: 手动禁用 tauriPlugin，直接调用 hvigorw.bat
- **Rust linker**: 通过环境变量配置 ohos clang
- **测试报告路径**: `/data/app/el2/100/base/com.tauri.api/cache/test-report.json`
- **autotest 触发**: `VITE_AUTOTEST=true` 环境变量或 URL 参数 `?autotest=true`
- **custom-protocol**: Cargo.toml 必须启用此 feature，否则 app 会尝试连接 devUrl

### 构建流程关键修复

1. **custom-protocol feature**: Cargo.toml 中添加 `"custom-protocol"` feature，避免 app 在设备上尝试连接 localhost:1420
2. **hvigorw JAVA_HOME**: cmd.exe 调用时必须将 JAVA_HOME/bin 加入 PATH
3. **hdc file recv**: 使用 cmd.exe 而非 Git Bash，避免路径格式问题
4. **panic hook 路径**: 使用内部沙箱路径 `/data/storage/el2/base/cache`（而非外部路径）

### 当前状态

- 构建流程：完整可用 ✓
- 签名安装：完整可用 ✓
- 测试报告写入：待调试（报告文件未能正确生成）
- 自动化脚本 run-tests.sh：已创建，需配合手动操作 hvigorfile.ts

### Git Commits

| Commit | 内容 |
|--------|------|
| `a331a9e6f` | feat: 添加测试框架、修复构建管道、添加文档和 skill |
| `292bd7d97` | fix: 修复 panic hook 路径，移除 debug prints |

### 待解决

1. 确认测试报告为何未能写入设备（可能 autotest 未触发或权限问题）
2. 将 hvigorfile.ts 的禁用/恢复操作自动化或文档化
3. 分析 ohos 测试结果，确定需要适配的 API

## 构建环境备忘

### Windows 上交叉编译 ohos 所需环境变量

```bash
export OHOS_HOME="/d/app/DevEco-Studio/sdk/default/openharmony"
export DEVECO_SDK_HOME="/d/app/DevEco-Studio/sdk"
export DEV_ECO_STUDIO_INSTALL_PATH="D:\\app\\DevEco-Studio"
export JAVA_HOME="/d/app/DevEco-Studio/jbr"
export NODE_HOME="/d/app/DevEco-Studio/tools/node"
export PATH="/d/app/DevEco-Studio/jbr/bin:/d/app/DevEco-Studio/tools/ohpm/bin:/d/app/DevEco-Studio/tools/hvigor/bin:/d/app/DevEco-Studio/tools/node/bin:$PATH"
```

### 手动构建流程（绕过 cargo tauri ohos build 的 TCP 回调问题）

```bash
# 1. Rust 编译
export CC_aarch64_unknown_linux_ohos="path/to/ohos-clang.cmd"
cargo build --target aarch64-unknown-linux-ohos --release

# 2. 拷贝 so
cp target/aarch64-unknown-linux-ohos/release/libapi_lib.so \
   src-tauri/gen/ohos/entry/libs/arm64-v8a/

# 3. 临时禁用 hvigorfile.ts 中的 tauriPlugin（避免 TCP 回调）
# 4. ohpm install && hvigorw assembleHap
# 5. 签名
java -jar hap-sign-tool.jar sign-app \
  -keyAlias "openharmony application release" \
  -keystoreFile OpenHarmony.p12 -keystorePwd 123456 \
  -appCertFile app-debug-cert.cer \
  -profileFile signed-profile.p7b \
  -inFile unsigned.hap -outFile signed.hap \
  -signAlg SHA256withECDSA -mode localSign

# 6. 卸载旧版 + 安装
hdc shell bm uninstall -n com.tauri.api
hdc install signed.hap

# 7. 启动
hdc shell aa start -a EntryAbility -b com.tauri.api
```

### 已知问题

1. `cargo tauri ohos build` 在 Windows 上会卡住——hvigor 通过 TCP 回调 `cargo tauri ohos dev-eco-studio-script`，但连接被拒绝
2. ohos SDK 中的 `aarch64-unknown-linux-ohos-clang` 是 shell 脚本，Windows 上不能直接执行，需要用 `.cmd` wrapper 或直接用 `clang.exe --target=aarch64-linux-ohos --sysroot=...`
3. App 运行时崩溃，需要进一步调试
