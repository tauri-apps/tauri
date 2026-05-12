# ohos-build

编译 Tauri OpenHarmony 项目（examples/api），生成 HAP 包并签名安装到设备。支持自动化前端测试。

## 环境要求

- **运行环境**: 使用 **Git Bash** 运行脚本（路径格式 `/d/app/...`）
  - Git Bash 位于: `C:\Program Files (x86)\Git\bin\bash.exe`
- DevEco Studio（含 OpenHarmony SDK、ohpm、hvigor、JBR）
- pnpm
- Rust + `aarch64-unknown-linux-ohos` target
- hdc（设备连接工具，SDK 自带）

## 一键测试流程

最简方式（需要先手动禁用 tauriPlugin）：

```bash
# 1. 禁用 hvigorfile.ts 中的 tauriPlugin（见下方说明）
# 2. 运行一键测试
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/run-tests.sh
# 3. 恢复 hvigorfile.ts
```

## 分步手动流程

### 步骤 1: 禁用 hvigorfile.ts 中的 tauriPlugin

编辑 `examples/api/src-tauri/gen/ohos/entry/hvigorfile.ts`，将：
```typescript
plugins:[tauriPlugin()]
```
改为：
```typescript
plugins:[]
```

原因：tauriPlugin 需要 TCP 回调 tauri CLI 进程，Windows 上连接会失败。

### 步骤 2: 构建

```bash
export VITE_AUTOTEST=true
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/build-ohos.sh
```

### 步骤 3: 签名安装

```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/sign-and-install.sh
```

### 步骤 4: 拉取报告

测试启动后约 10-15 秒即可完成。使用 cmd.exe 调用 hdc 避免 Git Bash 路径转义：

```bash
cmd.exe /c "hdc -t DEVICE_SN file recv /data/app/el2/100/base/com.tauri.api/cache/test-report.json D:\workspace\tauri\tauri\examples\api\test-report.json"
```

### 步骤 5: 恢复 hvigorfile.ts

将 `plugins:[]` 改回 `plugins:[tauriPlugin()]`。

## 脚本说明

| 脚本 | 功能 |
|------|------|
| `env.sh` | 共享环境配置，自动检测 DevEco Studio，导出 CC/linker/JAVA_HOME 等 |
| `build-ohos.sh` | 前端构建 → Rust 编译(--features prod) → 拷贝 .so → hvigorw 打包 |
| `sign-and-install.sh` | 生成 debug profile → 签名 → 卸载旧版 → 安装 → 启动 |
| `run-tests.sh` | 一键流程：build(autotest) → sign+install → 等待 → 拉取报告 → 分析 |

## 关键注意事项

### 1. 必须启用 prod feature

Rust 编译必须加 `--features prod`，否则 app 会尝试连接 `http://localhost:1420`（devUrl）而不是加载打包好的前端文件。

原因：Tauri 通过 `custom-protocol` feature 控制 `#[cfg(dev)]`，不启用时即使 release 构建也走 dev 路径。

`build-ohos.sh` 已包含此 flag。

### 2. hdc 路径转义问题

Git Bash 会把 `/data/...` 开头的路径转换为 `C:/Program Files (x86)/Git/data/...`。所有涉及设备路径的 hdc 命令必须通过 `cmd.exe /c "hdc ..."` 调用。

### 3. hvigorw 需要 java 在 PATH 中

cmd.exe 调用 hvigorw 时必须把 `JAVA_HOME/bin` 加入 PATH，否则报 `spawn java ENOENT`。`build-ohos.sh` 已处理。

### 4. 每次安装前必须卸载旧版本

签名证书每次生成不同，与设备上已安装版本冲突。`sign-and-install.sh` 会自动处理。

### 5. ohos 文件系统路径

| 视角 | 路径 | 用途 |
|------|------|------|
| App 内部（Rust 写入） | `/data/storage/el2/base/cache/` | `write_test_report` command |
| 外部（hdc 拉取） | `/data/app/el2/100/base/com.tauri.api/cache/` | `hdc file recv` |

## 自动检测项

- DevEco Studio 路径：自动检测 `/d/app/DevEco-Studio` 等常见位置
- 设备：通过 `hdc list targets` 获取
- 设备 UDID：通过 `hdc shell bm get --udid` 获取
- Bundle Name：从 `gen/ohos/AppScope/app.json5` 解析

## 首次使用

脚本依赖 `scripts/.env.local` 中的 DevEco Studio 路径。如果自动检测失败，手动创建：

```bash
echo 'DEVECO_HOME="/d/app/DevEco-Studio"' > .claude/skills/ohos-build/scripts/.env.local
```

## 测试报告格式

```json
{
  "timestamp": "2026-05-12T05:20:47.253Z",
  "total": 25, "passed": 18, "failed": 7, "skipped": 0,
  "results": [
    {"name": "@tauri-apps/api/core.invoke", "category": "auto", "status": "pass", "duration": 10},
    {"name": "@tauri-apps/plugin-fs.mkdir+...", "category": "side-effect", "status": "fail", "error": "Operation not permitted"}
  ]
}
```
