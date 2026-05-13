# Tauri OpenHarmony 踩坑记录

日期: 2026-05-12

本文档记录在将 Tauri examples/api 适配 OpenHarmony 过程中遇到的所有问题及解决方案。

---

## 1. 编译工具链问题

### 1.1 ohos SDK clang wrapper 不可执行

**症状**: `error occurred in cc-rs: %1 不是有效的 Win32 应用程序`

**原因**: ohos SDK 中的 `aarch64-unknown-linux-ohos-clang` 是 Linux shell 脚本，Windows 无法直接执行。

**解决方案**: 创建 `.cmd` wrapper 文件：

```batch
@echo off
"D:\app\DevEco-Studio\sdk\default\openharmony\native\llvm\bin\clang.exe" ^
  -target aarch64-linux-ohos ^
  --sysroot="D:\app\DevEco-Studio\sdk\default\openharmony\native\sysroot" ^
  -D__MUSL__ %*
```

设置环境变量：
```bash
export CC_aarch64_unknown_linux_ohos="/path/to/ohos-clang.cmd"
```

### 1.2 `cargo tauri ohos build` 在 Windows 上卡住

**症状**: Rust 编译完成后，hvigor 打包阶段无限等待。

**原因**: `entry/hvigorfile.ts` 中的 tauriPlugin 调用 `cargo tauri ohos dev-eco-studio-script --target aarch64`，需要通过 TCP 连接回 tauri CLI 进程。Windows 上 TCP 连接被拒绝或超时。

**解决方案**: 手动分步构建（绕过 TCP 回调）：

```bash
# 1. 单独编译 Rust
source .claude/skills/ohos-build/scripts/env.sh
cargo build --target aarch64-unknown-linux-ohos --release

# 2. 拷贝 .so 到 ohos 项目
cp target/aarch64-unknown-linux-ohos/release/libapi_lib.so \
   examples/api/src-tauri/gen/ohos/entry/libs/arm64-v8a/

# 3. 临时禁用 hvigorfile.ts 中的 tauriPlugin
# 将 plugins:[tauriPlugin()] 改为 plugins:[]

# 4. 手动打包
cd examples/api/src-tauri/gen/ohos
ohpm install
hvigorw.bat assembleHap --no-daemon

# 5. 恢复 hvigorfile.ts
```

### 1.3 hvigor 环境变量缺失

**症状**: `Invalid value of 'DEVECO_SDK_HOME'` 或 `spawn java ENOENT`

**解决方案**: 确保导出以下环境变量：
```bash
export DEVECO_SDK_HOME="/d/app/DevEco-Studio/sdk"
export JAVA_HOME="/d/app/DevEco-Studio/jbr"
export NODE_HOME="/d/app/DevEco-Studio/tools/node"
export PATH="/d/app/DevEco-Studio/jbr/bin:/d/app/DevEco-Studio/tools/ohpm/bin:/d/app/DevEco-Studio/tools/hvigor/bin:/d/app/DevEco-Studio/tools/node/bin:$PATH"
```

---

## 2. HAP 签名问题

### 2.1 签名 keyAlias 错误

**症状**: 签名失败，提示 key alias 不存在。

**原因**: 网上很多教程用 `oh-app1-key-v1`，但 SDK 自带的 `OpenHarmony.p12` 中实际的 alias 不同。

**正确参数**:
```
keyAlias: "openharmony application release"
keystorePwd: 123456
keystoreFile: $OHOS_HOME/toolchains/lib/OpenHarmony.p12
signAlg: SHA256withECDSA
```

可通过 keytool 验证：
```bash
keytool -list -keystore OpenHarmony.p12 -storetype PKCS12 -storepass 123456
```

---

## 3. 设备安装问题

### 3.1 必须先卸载旧版本

**症状**: `hdc install` 失败或安装后行为异常（旧代码残留）。

**原因**: 签名证书每次生成不同，与设备上已安装版本的签名冲突。

**解决方案**: 每次安装前必须卸载：
```bash
hdc -t $DEVICE_SN shell bm uninstall -n com.tauri.api
hdc -t $DEVICE_SN install signed.hap
```

### 3.2 hdc install 路径问题

**症状**: Windows 上 `hdc install /path/to/file.hap` 报文件不存在。

**原因**: hdc 在 Windows 上对路径处理有 bug，Unix 风格路径可能被错误拼接。

**解决方案**: 使用 Windows 反斜杠路径，或先 cd 到文件所在目录再用相对路径安装。

---

## 4. 运行时崩溃问题（重点）

### 4.1 调试方法

ohos 上 Rust panic 默认只输出到 stderr，不容易看到。解决方案是设置自定义 panic hook 写入 app 沙箱：

```rust
#[cfg(target_env = "ohos")]
std::panic::set_hook(Box::new(|info| {
    let msg = format!("PANIC: {info}\n");
    let _ = std::fs::write("/data/storage/el2/base/cache/panic.log", &msg);
    eprintln!("{msg}");
}));
```

拉取日志：
```bash
hdc file recv /data/app/el2/100/base/com.tauri.api/cache/panic.log ./panic.log
```

也可查看系统崩溃日志：
```bash
hdc shell "ls /data/log/faultlog/faultlogger/ | grep tauri"
hdc shell "cat /data/log/faultlog/faultlogger/<crash-file>"
```

### 4.2 tauri-plugin-log 崩溃

**症状**: `PluginInitialization("log", "Operation not permitted (os error 1)")`

**原因**: 默认的 LogDir target 调用 `app_log_dir()` → `dirs::cache_dir()` → `$HOME/.cache`。ohos 上 `$HOME=/storage/Users/currentUser`，该路径对 app 只读。

**修复**:
```rust
tauri_plugin_log::Builder::default()
    .level(log::LevelFilter::Info)
    .clear_targets()  // 关键：清除默认的 LogDir target
    .target(tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout))
    .build()
```

### 4.3 tauri-plugin-http 崩溃

**症状**: `PluginInitialization("http", "Operation not permitted (os error 1)")`

**原因**: http plugin 的 cookies 功能在初始化时调用 `app.path().app_cache_dir()` 创建 cookie 存储目录，同样走到 `dirs::cache_dir()` 返回不可写路径。

**临时修复**: 在 ohos 上排除 http plugin：
```rust
#[cfg(not(target_env = "ohos"))]
let builder = builder.plugin(tauri_plugin_http::init());
```

**长期方案**: 实现 ohos 专用路径模块（见下文 4.4）。

### 4.4 根本原因：Tauri 路径模块不支持 ohos

**核心问题**: `crates/tauri/src/path/mod.rs` 中：
```rust
#[cfg(target_os = "android")] mod android;
#[cfg(not(target_os = "android"))] mod desktop;
```

ohos 的 `target_os = "linux"`，所以走了 `desktop.rs`，而 desktop.rs 依赖 `dirs` crate 通过 `$HOME` 推导路径。

**ohos 文件系统特殊性**:
- `$HOME = /storage/Users/currentUser` — 只读，app 无写权限
- App 可写沙箱: `/data/storage/el2/base/`（内部视角）
- 外部视角: `/data/app/el2/100/base/com.tauri.api/`
- 子目录: `files/`, `cache/`, `preferences/` 等

**长期方案**: 需要在 tauri 中新增 ohos 路径实现：
```rust
#[cfg(target_env = "ohos")] mod ohos;
#[cfg(all(not(target_os = "android"), not(target_env = "ohos")))] mod desktop;
```

ohos 模块应通过 ArkTS Context API（或硬编码沙箱路径）返回正确的目录。

---

## 5. cfg 条件编译模式

### 5.1 ohos 的 target 特征

```
target_os = "linux"
target_env = "ohos"
target_arch = "aarch64"
```

Tauri 将 ohos 归类为 `mobile`（与 android/ios 一致），因此 `#[cfg(desktop)]` 不包含 ohos。

### 5.2 常用 cfg 模式

```rust
// ohos 需要走 desktop 代码路径的场景
#[cfg(any(desktop, target_env = "ohos"))]

// 仅 android/ios 的 mobile 代码（排除 ohos）
#[cfg(any(target_os = "android", target_os = "ios"))]

// Linux 特有但排除 ohos
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
```

### 5.3 为什么不把 ohos 改为 desktop

将 ohos 改为 desktop 会导致 `muda`（菜单）和 `tray_icon`（系统托盘）crate 被引入，这些依赖 GTK，无法在 ohos 上编译。所以必须保持 ohos 为 mobile，对需要的接口逐个用 `any(desktop, target_env = "ohos")` 开启。

---

## 6. 各 Plugin 适配要点

| Plugin | 问题 | 解决方案 |
|--------|------|----------|
| **fs** | desktop module 未对 ohos 开放 | `#[cfg(any(desktop, target_env = "ohos"))]` |
| **shell** | PluginHandle 仅 android/ios 需要 | 限定 `#[cfg(any(target_os = "android", target_os = "ios"))]` |
| **clipboard-manager** | 同 fs，desktop/mobile 模块门控 | 同上模式 |
| **log** | LogDir 路径不可写 | 使用 Stdout target |
| **http** | app_cache_dir 不可写 | 暂时排除，等路径模块修复 |
| **autostart** | appimage 分支在 ohos 上无意义 | `#[cfg(all(target_os = "linux", not(target_env = "ohos")))]` |

---

## 7. 构建必须启用 prod feature

### 7.1 App 显示 "failed to request http://localhost:1420"

**症状**: App 在 ohos 设备上启动后白屏，显示无法连接 localhost:1420。

**原因**: `cargo build --release` 不会自动启用 `custom-protocol` feature。Tauri 通过 `#[cfg(dev)]` 判断是否使用 devUrl，而 `dev = !custom_protocol`（见 `crates/tauri/build.rs:256`）。不启用 `custom-protocol` 时，即使是 release 构建也会尝试连接 devUrl。

**解决方案**: 编译时必须加 `--features prod`：
```bash
cargo build --target aarch64-unknown-linux-ohos --release --features prod
```

`Cargo.toml` 中的定义：
```toml
[features]
prod = ["tauri/custom-protocol"]
```

### 7.2 测试报告写入 permission denied

**症状**: 测试运行完成但报告保存失败，`permission denied (os error 13)`。

**原因**: ohos 上 `/data/app/el2/100/base/com.tauri.api/cache` 是外部视角路径，app 进程内部无法直接写入。App 内部可写的沙箱路径是 `/data/storage/el2/base/`。

**解决方案**: Rust 代码中使用内部视角路径：
```rust
#[cfg(target_env = "ohos")]
let dir = std::path::PathBuf::from("/data/storage/el2/base/cache");
```

对应的外部路径（用于 `hdc file recv`）：
```
/data/app/el2/100/base/com.tauri.api/cache/test-report.json
```

---

## 8. 环境要求清单

- DevEco Studio（含 OpenHarmony SDK、ohpm、hvigor、JBR）
- pnpm
- Rust + `aarch64-unknown-linux-ohos` target
- tauri-cli（OpenHarmony 分支）
- hdc（设备连接工具，SDK 自带）
- 设备已开启开发者模式和 USB 调试
