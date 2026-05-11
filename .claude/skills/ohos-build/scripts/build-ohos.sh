#!/bin/bash
# Tauri OpenHarmony Build Script
# 编译 examples/api/src-tauri 项目，生成未签名 HAP 包

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/env.sh"

API_DIR="$PROJECT_ROOT/examples/api"

echo "=== Tauri OpenHarmony Build ==="
echo "DEVECO_HOME=$DEVECO_HOME"
echo "PROJECT_ROOT=$PROJECT_ROOT"
echo ""

# 1. 安装前端依赖（如果 node_modules 不存在）
if [ ! -d "$API_DIR/node_modules" ]; then
    echo ">>> Installing frontend dependencies..."
    (cd "$API_DIR" && pnpm install)
fi

# 2. 构建 @tauri-apps/api（如果 dist 不存在）
if [ ! -d "$PROJECT_ROOT/packages/api/dist" ]; then
    echo ">>> Building @tauri-apps/api..."
    (cd "$PROJECT_ROOT" && pnpm build:api)
fi

# 3. 执行 cargo tauri ohos build
echo ">>> Running cargo tauri ohos build..."
(cd "$API_DIR" && cargo tauri ohos build)

echo ""
echo "=== Build Complete ==="
echo "HAP: $API_DIR/src-tauri/gen/ohos/entry/build/default/outputs/default/entry-default-unsigned.hap"
