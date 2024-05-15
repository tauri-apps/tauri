#!/usr/bin/env bash

OUT_DIR=./out
CONFIGURATION=debug
SCHEME_NAME="Tauri"
FRAMEWORK_NAME="Tauri"

DEVICE_ARCHIVE_PATH="${OUT_DIR}/${CONFIGURATION}/${FRAMEWORK_NAME}-iphoneos.xcarchive"

xcodebuild archive \
  -scheme ${SCHEME_NAME} \
  -archivePath ${DEVICE_ARCHIVE_PATH} \
  -sdk iphoneos \
  -destination "generic/platform=iOS,arch=arm64" \
  BUILD_LIBRARY_FOR_DISTRIBUTION=YES \
  SKIP_INSTALL=NO \
  OTHER_SWIFT_FLAGS="-no-verify-emitted-module-interface"
