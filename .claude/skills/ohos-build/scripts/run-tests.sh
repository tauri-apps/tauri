#!/bin/bash
# Tauri OpenHarmony 自动化测试脚本
# 编译(autotest) → 签名安装 → 启动 → 等待 → 拉取报告 → 分析

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/env.sh"

DEVICE_SN="${DEVICE_SN:-$1}"
OHOS_PROJECT="$PROJECT_ROOT/examples/api/src-tauri/gen/ohos"
APP_JSON="$OHOS_PROJECT/AppScope/app.json5"
BUNDLE_NAME=$(grep -o '"bundleName"[[:space:]]*:[[:space:]]*"[^"]*"' "$APP_JSON" | head -1 | sed 's/.*"bundleName"[[:space:]]*:[[:space:]]*"//;s/"//')
REPORT_DEVICE_PATH="/data/app/el2/100/base/$BUNDLE_NAME/cache/test-report.json"
REPORT_LOCAL="$PROJECT_ROOT/examples/api/test-report.json"
REPORT_LOCAL_WIN=$(echo "$REPORT_LOCAL" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
WAIT_SECONDS="${WAIT_SECONDS:-15}"

echo "=== Tauri OpenHarmony Auto Test ==="
echo "Bundle: $BUNDLE_NAME"
echo "Device: ${DEVICE_SN:-auto-detect}"
echo ""

# Step 1: Build with VITE_AUTOTEST=true
echo ">>> Step 1: Building (autotest mode)..."
export VITE_AUTOTEST=true
bash "$SCRIPT_DIR/build-ohos.sh"

# Step 2: Sign & Install
echo ""
echo ">>> Step 2: Sign & Install..."
if [ -n "$DEVICE_SN" ]; then
    bash "$SCRIPT_DIR/sign-and-install.sh" "$DEVICE_SN"
else
    bash "$SCRIPT_DIR/sign-and-install.sh"
fi

# Step 3: Wait for tests to complete
echo ""
echo ">>> Step 3: Waiting ${WAIT_SECONDS}s for tests to complete..."
sleep "$WAIT_SECONDS"

# Step 4: Pull report (use cmd.exe to avoid Git Bash path mangling)
echo ""
echo ">>> Step 4: Pulling test report..."
rm -f "$REPORT_LOCAL"

if [ -n "$DEVICE_SN" ]; then
    cmd.exe /c "hdc -t $DEVICE_SN file recv $REPORT_DEVICE_PATH $REPORT_LOCAL_WIN" 2>&1 | tr -d '\r'
else
    cmd.exe /c "hdc file recv $REPORT_DEVICE_PATH $REPORT_LOCAL_WIN" 2>&1 | tr -d '\r'
fi

if [ ! -f "$REPORT_LOCAL" ]; then
    echo "ERROR: Failed to pull test report from device."
    echo "Expected at: $REPORT_DEVICE_PATH"
    echo ""
    echo "Try increasing WAIT_SECONDS (current: $WAIT_SECONDS)"
    echo "Or check if the app started correctly on device."
    exit 1
fi

# Step 5: Analyze report
echo ""
echo "=== Test Report ==="
echo ""

python3 - "$REPORT_LOCAL" << 'PYTHON' 2>/dev/null || python - "$REPORT_LOCAL" << 'PYTHON'
import json, sys

with open(sys.argv[1]) as f:
    report = json.load(f)

print(f"Timestamp: {report['timestamp']}")
print(f"Total: {report['total']}, Passed: {report['passed']}, Failed: {report['failed']}, Skipped: {report['skipped']}")
print("")

passed = []
failed = []
skipped = []

for r in report['results']:
    if r['status'] == 'pass':
        passed.append(r)
    elif r['status'] == 'fail':
        failed.append(r)
    else:
        skipped.append(r)

if failed:
    print("--- FAILED ---")
    for r in failed:
        print(f"  FAIL: {r['name']} ({r['duration']}ms)")
        if r.get('error'):
            print(f"        Error: {r['error']}")
    print("")

if passed:
    print(f"--- PASSED ({len(passed)}) ---")
    for r in passed:
        print(f"  PASS: {r['name']} ({r['duration']}ms)")
    print("")

if skipped:
    print(f"--- SKIPPED ({len(skipped)}) ---")
    for r in skipped:
        print(f"  SKIP: {r['name']}")
    print("")

# Summary
print("=" * 50)
if report['failed'] == 0:
    print("ALL TESTS PASSED!")
else:
    print(f"FAILURES: {report['failed']} test(s) failed.")
    print("")
    print("Failed APIs (not yet adapted for OpenHarmony):")
    for r in failed:
        print(f"  - {r['name']}: {r.get('error', 'unknown')}")

sys.exit(0 if report['failed'] == 0 else 1)
PYTHON
