/* Copyright 2019-2024 Tauri Programme within The Commons Conservancy
 * SPDX-License-Identifier: Apache-2.0
 * SPDX-License-Identifier: MIT
 *
 * Minimal tauri-ffi consumer: opens a window and runs until it is closed.
 *
 * Build (from the repo root, after `cargo build -p tauri-ffi`):
 *   macOS:  cc bindings/c/examples/hello.c -I bindings/c \
 *             -L target/debug -ltauri_ffi -o /tmp/tauri-ffi-hello
 *   Linux:  cc bindings/c/examples/hello.c -I bindings/c \
 *             -L target/debug -ltauri_ffi -Wl,-rpath,target/debug -o /tmp/tauri-ffi-hello
 */

#include <stdio.h>
#include "tauri_ffi.h"

static const char *CONFIG =
    "{"
    "  \"productName\": \"tauri-ffi-c-hello\","
    "  \"version\": \"0.1.0\","
    "  \"identifier\": \"com.tauri.ffi.chello\","
    "  \"app\": {"
    "    \"windows\": ["
    "      { \"label\": \"main\", \"title\": \"Tauri FFI - C\","
    "        \"url\": \"https://tauri.app\", \"width\": 800, \"height\": 600 }"
    "    ]"
    "  }"
    "}";

static int check(int32_t code, const char *what) {
  if (code != TAURI_OK) {
    const char *message = tauri_last_error_message();
    fprintf(stderr, "%s failed (%d): %s\n", what, code, message ? message : "");
    return 1;
  }
  return 0;
}

int main(void) {
  printf("tauri-ffi %s (abi %u)\n", tauri_ffi_version(), tauri_ffi_abi_version());

  uint64_t builder = 0;
  if (check(tauri_app_builder_new(CONFIG, &builder), "builder_new")) return 1;

  uint64_t app = 0;
  if (check(tauri_app_build(builder, &app), "app_build")) return 1;

  int32_t exit_code = 0;
  if (check(tauri_app_run(app, &exit_code), "app_run")) return 1;

  printf("app exited with code %d\n", exit_code);
  tauri_handle_close(app);
  return exit_code;
}
