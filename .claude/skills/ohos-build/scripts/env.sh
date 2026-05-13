#!/bin/bash
# env.sh — 共享环境配置，自动检测 DevEco Studio 路径
# 被 build-ohos.sh 和 sign-and-install.sh source

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_LOCAL="$SCRIPT_DIR/.env.local"

# ─── 加载已有配置 ───
if [ -f "$ENV_LOCAL" ]; then
    source "$ENV_LOCAL"
fi

# ─── 自动检测 DevEco Studio (Git Bash 路径格式) ───
detect_deveco_home() {
    local candidates=(
        "/d/app/DevEco-Studio"
        "/d/app/DevEco Studio"
        "/c/Program Files/Huawei/DevEco Studio"
        "/c/Program Files (x86)/Huawei/DevEco Studio"
        "$HOME/DevEco-Studio"
    )
    for path in "${candidates[@]}"; do
        if [ -d "$path/sdk/default/openharmony" ]; then
            echo "$path"
            return 0
        fi
    done
    return 1
}

if [ -z "$DEVECO_HOME" ]; then
    DEVECO_HOME=$(detect_deveco_home)
    if [ -z "$DEVECO_HOME" ]; then
        echo "ERROR: DevEco Studio not found."
        echo "Please create $ENV_LOCAL with content:"
        echo '  DEVECO_HOME="/path/to/DevEco-Studio"'
        exit 1
    fi
    # 保存配置
    echo "DEVECO_HOME=\"$DEVECO_HOME\"" > "$ENV_LOCAL"
    echo "Saved DevEco Studio path to $ENV_LOCAL"
fi

# ─── 验证路径有效性 ───
if [ ! -d "$DEVECO_HOME/sdk/default/openharmony" ]; then
    echo "ERROR: DEVECO_HOME=$DEVECO_HOME is invalid (sdk not found)"
    echo "Delete $ENV_LOCAL and re-run to reconfigure."
    exit 1
fi

# ─── 导出环境变量 ───
export DEVECO_HOME
export OHOS_HOME="$DEVECO_HOME/sdk/default/openharmony"
export JAVA_HOME="$DEVECO_HOME/jbr"
# Windows 格式路径，供 cargo-mobile2、clang.exe 等使用
export DEV_ECO_STUDIO_INSTALL_PATH=$(echo "$DEVECO_HOME" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
export PATH="$DEVECO_HOME/jbr/bin:$PATH:$DEVECO_HOME/tools/hvigor/bin:$DEVECO_HOME/tools/ohpm/bin:$OHOS_HOME/toolchains"

# ─── 设置 ohos clang 编译器 (供 ring 等 native crate 使用) ───
OHOS_CLANG=$(echo "$OHOS_HOME/native/llvm/bin/clang.exe" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
OHOS_SYSROOT=$(echo "$OHOS_HOME/native/sysroot" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
OHOS_AR=$(echo "$OHOS_HOME/native/llvm/bin/llvm-ar.exe" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
export CC_aarch64_unknown_linux_ohos="$OHOS_CLANG"
export CFLAGS_aarch64_unknown_linux_ohos="--target=aarch64-linux-ohos --sysroot=$OHOS_SYSROOT -D__MUSL__"
export AR_aarch64_unknown_linux_ohos="$OHOS_AR"

# ─── Rust linker 配置 ───
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER="$OHOS_CLANG"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_RUSTFLAGS="-C link-arg=--target=aarch64-linux-ohos -C link-arg=--sysroot=$OHOS_SYSROOT -C link-arg=-D__MUSL__"

# ─── 推导项目根目录（skill 在 .claude/skills/ohos-build/scripts/ 下）───
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
export PROJECT_ROOT
