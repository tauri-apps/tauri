#!/bin/bash
# Tauri OpenHarmony Build Script (手动流程)
# 编译 Rust + 前端，生成未签名 HAP

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/env.sh"

API_DIR="$PROJECT_ROOT/examples/api"
SRC_TAURI="$API_DIR/src-tauri"
OHOS_PROJECT="$SRC_TAURI/gen/ohos"
UNSIGNED_HAP="$OHOS_PROJECT/entry/build/default/outputs/default/entry-default-unsigned.hap"
SO_FILE="$PROJECT_ROOT/target/aarch64-unknown-linux-ohos/release/libapi_lib.so"

echo "=== Tauri OpenHarmony Build ==="
echo "DEVECO_HOME=$DEVECO_HOME"
echo "PROJECT_ROOT=$PROJECT_ROOT"
echo ""

# ─── 设置 Windows 格式环境变量 ───
DEVECO_SDK_HOME_WIN=$(echo "$DEVECO_HOME/sdk" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
JAVA_HOME_WIN=$(echo "$DEVECO_HOME/jbr" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
HVIGORW_BAT_WIN=$(echo "$DEVECO_HOME/tools/hvigor/bin/hvigorw.bat" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
OHOS_PROJECT_WIN=$(echo "$OHOS_PROJECT" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')

export DEVECO_SDK_HOME="$DEVECO_SDK_HOME_WIN"
export JAVA_HOME="$JAVA_HOME_WIN"

echo "JAVA_HOME=$JAVA_HOME"
echo "DEVECO_SDK_HOME=$DEVECO_SDK_HOME"
echo "HVIGORW_BAT=$HVIGORW_BAT_WIN"

# ─── Step 1: 安装前端依赖 ───
if [ ! -d "$API_DIR/node_modules" ]; then
    echo ""
    echo ">>> Step 1: Installing frontend dependencies..."
    (cd "$API_DIR" && pnpm install)
fi

# ─── Step 2: 构建 @tauri-apps/api ───
if [ ! -d "$PROJECT_ROOT/packages/api/dist" ]; then
    echo ""
    echo ">>> Step 2: Building @tauri-apps/api..."
    (cd "$PROJECT_ROOT" && pnpm build:api)
fi

# ─── Step 3: 前端构建 ───
echo ""
echo ">>> Step 3: Building frontend (VITE_AUTOTEST=${VITE_AUTOTEST:-false})..."
export VITE_AUTOTEST="${VITE_AUTOTEST:-false}"
(cd "$API_DIR" && pnpm build)

# ─── Step 4: Rust 编译 ───
echo ""
echo ">>> Step 4: Compiling Rust (aarch64-unknown-linux-ohos release)..."
rm -f "$SO_FILE"
(cd "$SRC_TAURI" && cargo build --target aarch64-unknown-linux-ohos --release --features prod)

if [ ! -f "$SO_FILE" ]; then
    echo "ERROR: Rust compilation failed - .so not found"
    exit 1
fi
echo "    Generated: $SO_FILE"

# ─── Step 5: 拷贝 .so 到 ohos 项目 ───
echo ""
echo ">>> Step 5: Copying .so to ohos project..."
mkdir -p "$OHOS_PROJECT/entry/libs/arm64-v8a"
cp "$SO_FILE" "$OHOS_PROJECT/entry/libs/arm64-v8a/libapi_lib.so"

# ─── Step 6: hvigorw 打包 ───
echo ""
echo ">>> Step 6: Running hvigorw assembleHap..."
echo "    NOTE: Before running this, you must manually disable tauriPlugin in hvigorfile.ts"
echo "    See SKILL.md for instructions."
rm -f "$UNSIGNED_HAP"

# 使用 cmd.exe 设置环境变量并调用 hvigorw.bat（PATH 必须包含 jbr/bin 否则 spawn java ENOENT）
JAVA_BIN_WIN=$(echo "$DEVECO_HOME/jbr/bin" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
cmd.exe /c "set DEVECO_SDK_HOME=$DEVECO_SDK_HOME_WIN&& set JAVA_HOME=$JAVA_HOME_WIN&& set PATH=$JAVA_BIN_WIN;%PATH%&& cd /d $OHOS_PROJECT_WIN&& \"$HVIGORW_BAT_WIN\" assembleHap --no-daemon"

# ─── 验证产物 ───
if [ ! -f "$UNSIGNED_HAP" ]; then
    echo "ERROR: Build failed - unsigned HAP not found at:"
    echo "  $UNSIGNED_HAP"
    exit 1
fi

echo ""
echo "=== Build Complete ==="
echo "HAP: $UNSIGNED_HAP"