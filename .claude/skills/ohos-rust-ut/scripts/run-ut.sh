#!/bin/bash
# OHOS Rust 单元测试一键脚本
# 交叉编译 → 推送设备 → 运行 → 输出结果

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# 复用 ohos-build 的环境配置
source "$SCRIPT_DIR/../../ohos-build/scripts/env.sh"

# ─── 参数 ───
PACKAGE="${PACKAGE:-tauri}"
TEST_FILTER="${TEST_FILTER:-}"       # 可选: 只跑匹配的测试 (e.g. "path::ohos")
DEVICE_SN="${DEVICE_SN:-}"
DEVICE_DIR="${DEVICE_DIR:-/data/local/tmp}"
TARGET="aarch64-unknown-linux-ohos"

# 位置参数：第一个为 TEST_FILTER
if [ -n "$1" ] && [[ "$1" != -* ]]; then
    TEST_FILTER="$1"
    shift
fi

HDC_ARGS=""
if [ -n "$DEVICE_SN" ]; then
    HDC_ARGS="-t $DEVICE_SN"
fi

echo "=== OHOS Rust UT Runner ==="
echo "Package:       $PACKAGE"
echo "Test filter:   ${TEST_FILTER:-<all>}"
echo "Target:        $TARGET"
echo "Device:        ${DEVICE_SN:-auto}"
echo ""

# ─── Step 1: 交叉编译测试二进制 ───
echo ">>> Step 1: Cross-compiling test binary..."
cd "$PROJECT_ROOT"

# --no-run 只编译不执行；--message-format=json 便于解析产物路径
COMPILE_OUTPUT=$(cargo test \
    --target "$TARGET" \
    -p "$PACKAGE" \
    --lib \
    ${TEST_FILTER:+$TEST_FILTER} \
    --no-run \
    --message-format=json 2>&1 || true)

# 解析出 executable 路径（最后一个 profile.test=true 的 artifact）
# 注意：不能用 python - <<HEREDOC，heredoc 会占用 stdin，导致 cargo 输出读不到
PARSER_SCRIPT='
import sys, json
last_exe = None
for line in sys.stdin:
    line = line.strip()
    if not line.startswith("{"):
        continue
    try:
        obj = json.loads(line)
    except Exception:
        continue
    if obj.get("reason") != "compiler-artifact":
        continue
    exe = obj.get("executable")
    if not exe:
        continue
    profile = obj.get("profile", {})
    if profile.get("test") is True:
        last_exe = exe
print(last_exe or "")
'
BINARY=$(echo "$COMPILE_OUTPUT" | python -c "$PARSER_SCRIPT" 2>/dev/null || echo "")

if [ -z "$BINARY" ] || [ ! -f "$BINARY" ]; then
    echo "ERROR: Failed to locate compiled test binary."
    echo "Cargo output tail:"
    echo "$COMPILE_OUTPUT" | tail -20
    exit 1
fi

# Windows 路径转 Unix
if [[ "$BINARY" == *"\\"* ]]; then
    BINARY=$(echo "$BINARY" | sed 's|\\|/|g' | sed 's|^\([A-Z]\):|/\L\1|')
fi

echo "    Binary: $BINARY"
BINARY_SIZE=$(stat -c %s "$BINARY" 2>/dev/null || stat -f %z "$BINARY")
echo "    Size:   $(( BINARY_SIZE / 1024 / 1024 )) MB"
echo ""

# ─── Step 2: 推送到设备 ───
echo ">>> Step 2: Pushing to device..."
BINARY_NAME=$(basename "$BINARY")
DEVICE_BINARY="$DEVICE_DIR/$BINARY_NAME"

# Windows 路径格式供 cmd.exe hdc 使用
BINARY_WIN=$(echo "$BINARY" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')

cmd.exe /c "hdc $HDC_ARGS file send $BINARY_WIN $DEVICE_BINARY" 2>&1 | tr -d '\r' | grep -v "^$"
echo ""

# ─── Step 3: 在设备上执行 ───
echo ">>> Step 3: Running on device..."
echo ""

cmd.exe /c "hdc $HDC_ARGS shell chmod +x $DEVICE_BINARY" 2>&1 | tr -d '\r'

# 捕获输出和退出码
TEST_OUTPUT=$(cmd.exe /c "hdc $HDC_ARGS shell $DEVICE_BINARY ${TEST_FILTER} --test-threads=1 2>&1; echo __EXIT_CODE__=\$?" 2>&1 | tr -d '\r')

# 提取退出码
EXIT_CODE=$(echo "$TEST_OUTPUT" | grep -oE "__EXIT_CODE__=[0-9]+" | tail -1 | cut -d= -f2)
# 打印除标记外的输出
echo "$TEST_OUTPUT" | grep -v "^__EXIT_CODE__="

echo ""
echo "=========================================="
if [ "$EXIT_CODE" = "0" ]; then
    echo "ALL TESTS PASSED"
    exit 0
else
    echo "TESTS FAILED (exit code: ${EXIT_CODE:-unknown})"
    exit 1
fi
