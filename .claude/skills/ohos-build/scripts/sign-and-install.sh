#!/bin/bash
# Tauri OpenHarmony HAP 签名 + 安装脚本
# 自动检测设备、bundle name，完成签名并安装启动

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/env.sh"

TOOLCHAINS="$DEVECO_HOME/sdk/default/openharmony/toolchains"
SIGN_LIB="$TOOLCHAINS/lib"
HAP_SIGN_TOOL="$SIGN_LIB/hap-sign-tool.jar"
KEYSTORE="$SIGN_LIB/OpenHarmony.p12"
PROFILE_CERT="$SIGN_LIB/OpenHarmonyProfileDebug.pem"
STORE_PWD="123456"

# 转换为 Windows 格式路径（供 java 使用）
WIN_TOOLCHAINS=$(echo "$TOOLCHAINS" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
WIN_SIGN_LIB=$(echo "$SIGN_LIB" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
WIN_HAP_SIGN_TOOL="$WIN_SIGN_LIB\\hap-sign-tool.jar"
WIN_KEYSTORE="$WIN_SIGN_LIB\\OpenHarmony.p12"
WIN_PROFILE_CERT="$WIN_SIGN_LIB\\OpenHarmonyProfileDebug.pem"

OHOS_PROJECT="$PROJECT_ROOT/examples/api/src-tauri/gen/ohos"
UNSIGNED_HAP="$OHOS_PROJECT/entry/build/default/outputs/default/entry-default-unsigned.hap"
WORK_DIR="$OHOS_PROJECT/.sign"

# ─── 检查未签名 HAP ───
if [ ! -f "$UNSIGNED_HAP" ]; then
    echo "ERROR: Unsigned HAP not found at:"
    echo "  $UNSIGNED_HAP"
    echo "Run build-ohos.sh first."
    exit 1
fi

# ─── 自动检测 bundle name ───
APP_JSON="$OHOS_PROJECT/AppScope/app.json5"
if [ -f "$APP_JSON" ]; then
    BUNDLE_NAME=$(grep -o '"bundleName"[[:space:]]*:[[:space:]]*"[^"]*"' "$APP_JSON" | head -1 | sed 's/.*"bundleName"[[:space:]]*:[[:space:]]*"//;s/"//')
fi
if [ -z "$BUNDLE_NAME" ]; then
    echo "ERROR: Cannot detect bundleName from $APP_JSON"
    exit 1
fi

# ─── 自动检测设备 ───
select_device() {
    local targets
    targets=$(hdc list targets 2>&1 | tr -d '\r' | grep -v '^\[' | grep -v '^$')
    local count=$(echo "$targets" | wc -l)

    if [ -z "$targets" ] || [ "$count" -eq 0 ]; then
        echo "ERROR: No device connected. Check hdc connection."
        exit 1
    elif [ "$count" -eq 1 ]; then
        DEVICE_SN="$targets"
    else
        echo "Multiple devices detected:"
        local i=1
        while IFS= read -r line; do
            echo "  [$i] $line"
            i=$((i+1))
        done <<< "$targets"
        echo "Select device [1-$count]:"
        read -r choice
        DEVICE_SN=$(echo "$targets" | sed -n "${choice}p")
    fi

    if [ -z "$DEVICE_SN" ]; then
        echo "ERROR: No device selected."
        exit 1
    fi
}

# 允许通过环境变量或参数指定设备
DEVICE_SN="${DEVICE_SN:-$1}"
if [ -z "$DEVICE_SN" ]; then
    select_device
fi

# ─── 获取设备 UDID ───
echo "=== Tauri OpenHarmony Sign & Install ==="
echo "Device: $DEVICE_SN"
echo "Bundle: $BUNDLE_NAME"
echo ""

DEVICE_UDID=$(hdc -t "$DEVICE_SN" shell bm get --udid 2>&1 | tr -d '\r' | grep -v '^udid' | tr -d '[:space:]')
if [ -z "$DEVICE_UDID" ]; then
    echo "ERROR: Failed to get UDID from device $DEVICE_SN"
    exit 1
fi
echo "UDID: $DEVICE_UDID"
echo ""

# ─── 创建工作目录 ───
mkdir -p "$WORK_DIR"

# ─── Step 1: 生成 debug profile JSON ───
echo ">>> Step 1: Generating debug profile..."
cat > "$WORK_DIR/debug-profile.json" << EOF
{
    "version-name": "2.0.0",
    "version-code": 2,
    "uuid": "fe686e1b-3770-4824-a938-961b140a7c98",
    "validity": {
        "not-before": 1610519532,
        "not-after": 1924959532
    },
    "type": "debug",
    "bundle-info": {
        "developer-id": "OpenHarmony",
        "development-certificate": "-----BEGIN CERTIFICATE-----\nMIICMzCCAbegAwIBAgIEaOC/zDAMBggqhkjOPQQDAwUAMGMxCzAJBgNVBAYTAkNO\nMRQwEgYDVQQKEwtPcGVuSGFybW9ueTEZMBcGA1UECxMQT3Blbkhhcm1vbnkgVGVh\nbTEjMCEGA1UEAxMaT3Blbkhhcm1vbnkgQXBwbGljYXRpb24gQ0EwHhcNMjEwMjAy\nMTIxOTMxWhcNNDkxMjMxMTIxOTMxWjBoMQswCQYDVQQGEwJDTjEUMBIGA1UEChML\nT3Blbkhhcm1vbnkxGTAXBgNVBAsTEE9wZW5IYXJtb255IFRlYW0xKDAmBgNVBAMT\nH09wZW5IYXJtb255IEFwcGxpY2F0aW9uIFJlbGVhc2UwWTATBgcqhkjOPQIBBggq\nhkjOPQMBBwNCAATbYOCQQpW5fdkYHN45v0X3AHax12jPBdEDosFRIZ1eXmxOYzSG\nJwMfsHhUU90E8lI0TXYZnNmgM1sovubeQqATo1IwUDAfBgNVHSMEGDAWgBTbhrci\nFtULoUu33SV7ufEFfaItRzAOBgNVHQ8BAf8EBAMCB4AwHQYDVR0OBBYEFPtxruhl\ncRBQsJdwcZqLu9oNUVgaMAwGCCqGSM49BAMDBQADaAAwZQIxAJta0PQ2p4DIu/ps\nLMdLCDgQ5UH1l0B4PGhBlMgdi2zf8nk9spazEQI/0XNwpft8QAIwHSuA2WelVi/o\nzAlF08DnbJrOOtOnQq5wHOPlDYB4OtUzOYJk9scotrEnJxJzGsh/\n-----END CERTIFICATE-----\n",
        "bundle-name": "$BUNDLE_NAME",
        "apl": "normal",
        "app-feature": "hos_normal_app"
    },
    "acls": {
        "allowed-acls": [""]
    },
    "permissions": {
        "restricted-permissions": [""]
    },
    "debug-info": {
        "device-ids": [
            "$DEVICE_UDID"
        ],
        "device-id-type": "udid"
    },
    "issuer": "pki_internal"
}
EOF

# ─── Step 2: 签名 profile ───
echo ">>> Step 2: Signing profile..."
java -jar "$WIN_HAP_SIGN_TOOL" sign-profile \
    -mode localSign \
    -keyAlias "openharmony application profile debug" \
    -keyPwd "$STORE_PWD" \
    -profileCertFile "$WIN_PROFILE_CERT" \
    -inFile "$WORK_DIR/debug-profile.json" \
    -signAlg SHA256withECDSA \
    -keystoreFile "$WIN_KEYSTORE" \
    -keystorePwd "$STORE_PWD" \
    -outFile "$WORK_DIR/signed-profile.p7b"

# ─── Step 3: 生成 app 证书链 ───
echo ">>> Step 3: Generating app certificate..."
WIN_KEYSTORE_ESC=$(echo "$WIN_KEYSTORE" | sed 's/\\/\\\\/g')
keytool -exportcert -keystore "$WIN_KEYSTORE" -storetype PKCS12 \
    -storepass "$STORE_PWD" -alias "openharmony application ca" \
    -rfc > "$WORK_DIR/sub-ca.cer" 2>/dev/null
keytool -exportcert -keystore "$WIN_KEYSTORE" -storetype PKCS12 \
    -storepass "$STORE_PWD" -alias "openharmony application root ca" \
    -rfc > "$WORK_DIR/root-ca.cer" 2>/dev/null

java -jar "$WIN_HAP_SIGN_TOOL" generate-app-cert \
    -keyAlias "openharmony application release" \
    -keyPwd "$STORE_PWD" \
    -issuer "C=CN,O=OpenHarmony,OU=OpenHarmony Team,CN=OpenHarmony Application CA" \
    -issuerKeyAlias "openharmony application ca" \
    -issuerKeyPwd "$STORE_PWD" \
    -subject "C=CN,O=OpenHarmony,OU=OpenHarmony Team,CN=OpenHarmony Application Release" \
    -signAlg SHA256withECDSA \
    -subCaCertFile "$WORK_DIR/sub-ca.cer" \
    -rootCaCertFile "$WORK_DIR/root-ca.cer" \
    -keystoreFile "$WIN_KEYSTORE" \
    -keystorePwd "$STORE_PWD" \
    -outFile "$WORK_DIR/app-debug-cert.cer" \
    -validity 365

# ─── Step 4: 签名 HAP ───
echo ">>> Step 4: Signing HAP..."
SIGNED_HAP="$WORK_DIR/entry-default-signed.hap"
java -jar "$WIN_HAP_SIGN_TOOL" sign-app \
    -mode localSign \
    -keyAlias "openharmony application release" \
    -keyPwd "$STORE_PWD" \
    -appCertFile "$WORK_DIR/app-debug-cert.cer" \
    -profileFile "$WORK_DIR/signed-profile.p7b" \
    -inFile "$UNSIGNED_HAP" \
    -signAlg SHA256withECDSA \
    -keystoreFile "$WIN_KEYSTORE" \
    -keystorePwd "$STORE_PWD" \
    -outFile "$SIGNED_HAP" \
    -signCode "1"

echo "    Signed: $SIGNED_HAP"

# ─── Step 5: 卸载旧版本 + 安装新版本 ───
echo ">>> Step 5: Uninstalling old bundle..."
hdc -t "$DEVICE_SN" shell bm uninstall -n "$BUNDLE_NAME" 2>&1 | tr -d '\r' || true

echo ">>> Step 6: Installing..."
WIN_HAP=$(echo "$SIGNED_HAP" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
hdc -t "$DEVICE_SN" install "$WIN_HAP" 2>&1 | tr -d '\r'

# ─── Step 7: 启动应用 ───
echo ">>> Step 7: Launching..."
hdc -t "$DEVICE_SN" shell aa start -b "$BUNDLE_NAME" -a EntryAbility 2>&1 | tr -d '\r'

echo ""
echo "=== Done ==="
