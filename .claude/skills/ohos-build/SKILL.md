# ohos-build

编译 Tauri OpenHarmony 项目（examples/api），生成 HAP 包并签名安装到设备。

## 工作流程

整个流程分三步，依次执行：

### Step 0: 环境配置（首次执行，已有 `.env.local` 则跳过）

检查 `scripts/.env.local` 是否存在。如果不存在：
1. 询问用户 DevEco Studio 的安装路径
2. 将路径转换为 Unix 格式（如 `D:\app\DevEco-Studio` → `/d/app/DevEco-Studio`）
3. 写入 `scripts/.env.local`：
   ```bash
   DEVECO_HOME="/d/app/DevEco-Studio"
   ```

后续脚本通过 `env.sh` 加载此配置，自动导出 OHOS_HOME、JAVA_HOME、PATH 等环境变量。

### Step 1: 编译（生成未签名 HAP）

```bash
bash .claude/skills/ohos-build/scripts/build-ohos.sh
```

该脚本完成以下工作：
1. 检测并配置 DevEco Studio 环境（首次运行时自动检测或询问路径）
2. 安装前端依赖（`pnpm install`，仅 node_modules 不存在时）
3. 构建 `@tauri-apps/api`（仅 dist 不存在时）
4. 执行 `cargo tauri ohos build`，生成未签名 HAP

产物路径：`examples/api/src-tauri/gen/ohos/entry/build/default/outputs/default/entry-default-unsigned.hap`

### Step 2: 签名 + 安装 + 启动

```bash
# 自动检测设备
bash .claude/skills/ohos-build/scripts/sign-and-install.sh

# 或指定设备序列号
bash .claude/skills/ohos-build/scripts/sign-and-install.sh <DEVICE_SN>
```

该脚本完成以下工作：
1. 自动检测连接的设备（多设备时交互选择）
2. 获取设备 UDID
3. 生成 debug profile JSON（包含设备 UDID 和 bundle name）
4. 使用 hap-sign-tool.jar 签名 profile
5. 生成 app 调试证书链（end-entity + sub-CA + root-CA）
6. 签名 HAP 包
7. 卸载设备上的旧版本（`hdc shell bm uninstall`）
8. 安装已签名 HAP 到设备（`hdc install`）
9. 启动应用（`hdc shell aa start`）

产物路径：`examples/api/src-tauri/gen/ohos/.sign/entry-default-signed.hap`

## 首次使用

脚本依赖 `scripts/.env.local` 中的 DevEco Studio 路径。如果该文件不存在且自动检测失败，脚本会报错退出。按 Step 0 配置即可解决。

如需重新配置，删除 `.env.local` 即可：
```bash
rm .claude/skills/ohos-build/scripts/.env.local
```

## 环境要求

- DevEco Studio（含 OpenHarmony SDK、ohpm、hvigor、JBR）
- pnpm
- Rust + `aarch64-unknown-linux-ohos` target
- tauri-cli（OpenHarmony 分支）：`cargo install tauri-cli --git https://github.com/tauri-apps/tauri --branch feat/open-harmony`

## 脚本说明

| 脚本 | 功能 |
|------|------|
| `env.sh` | 共享环境配置，自动检测 DevEco Studio 并导出环境变量 |
| `build-ohos.sh` | 安装前端依赖 → 构建 @tauri-apps/api → cargo tauri ohos build |
| `sign-and-install.sh` | 签名 → 卸载旧版 → 安装 → 启动 |

## 自动检测项

- DevEco Studio 路径：搜索 `/d/app/DevEco-Studio`、`/c/Program Files/Huawei/DevEco Studio` 等
- 设备：通过 `hdc list targets` 获取，多设备时交互选择
- 设备 UDID：通过 `hdc shell bm get --udid` 获取
- Bundle Name：从 `gen/ohos/AppScope/app.json5` 解析

## Cargo.toml patch

workspace 根目录 `Cargo.toml` 需包含本地 openharmony-ability patch：

```toml
[patch."https://github.com/harmony-contrib/openharmony-ability.git"]
openharmony-ability = { path = "../openharmony-ability/crates/ability" }
openharmony-ability-derive = { path = "../openharmony-ability/crates/derive" }
```
