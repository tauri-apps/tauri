/* Copyright 2019-2024 Tauri Programme within The Commons Conservancy
 * SPDX-License-Identifier: Apache-2.0
 * SPDX-License-Identifier: MIT
 *
 * Embeds Tauri's `examples/api` validation app — the same Svelte frontend the
 * Rust example uses — from C. The frontend is NOT copied here: this points the
 * asset server at the shared `examples/api/dist` directory, so the frontend
 * stays a single source of truth.
 *
 * Core APIs (app info, window, menu, tray, events — everything the frontend
 * reaches through @tauri-apps/api) are served by Tauri itself and work as-is.
 * The Communication view's custom commands (log_operation, perform_request,
 * echo, spam) are delivered over the `tauri_events_next` queue and need a host
 * pump to answer; the node / deno / python examples show that. Left
 * unregistered here, they reject cleanly with "command not found".
 *
 * Build the frontend once, then compile and run from the repo root:
 *   pnpm i && pnpm --dir examples/api build       # produces examples/api/dist
 *   cargo build -p tauri-ffi
 *   macOS:  cc bindings/c/examples/api.c -I bindings/c \
 *             -L target/debug -ltauri_ffi -o /tmp/tauri-ffi-api
 *   Linux:  cc bindings/c/examples/api.c -I bindings/c \
 *             -L target/debug -ltauri_ffi -Wl,-rpath,target/debug -o /tmp/tauri-ffi-api
 *   /tmp/tauri-ffi-api                            # run from the repo root
 */

#include <stdio.h>
#include "tauri_ffi.h"

/* Assets directory, relative to the working directory (the repo root). */
static const char *ASSETS_DIR = "examples/api/dist";

static const char *CONFIG =
    "{"
    "  \"productName\": \"tauri-ffi-api-c\","
    "  \"version\": \"1.0.0\","
    "  \"identifier\": \"com.tauri.ffi.api.c\","
    "  \"app\": {"
    "    \"withGlobalTauri\": true,"
    "    \"windows\": ["
    "      { \"label\": \"main\", \"title\": \"Tauri API - C\","
    "        \"url\": \"index.html\", \"width\": 1000, \"height\": 800,"
    "        \"minWidth\": 600, \"minHeight\": 400 }"
    "    ]"
    "  }"
    "}";

/* Capability the frontend needs, in the same file format a Rust app keeps under
 * capabilities/. The scripting bindings auto-discover a capabilities/ directory;
 * over the raw C ABI you add each capability yourself with
 * tauri_app_builder_add_capability(). Adding any capability replaces the default
 * core:default grant, so it is listed here explicitly. */
static const char *CAPABILITY =
    "{"
    "  \"identifier\": \"run-app\","
    "  \"description\": \"permissions the examples/api frontend needs\","
    "  \"windows\": [\"main\", \"main-*\"],"
    "  \"permissions\": ["
    "    \"core:default\","
    "    \"core:app:allow-app-hide\", \"core:app:allow-app-show\","
    "    \"core:app:allow-set-app-theme\", \"core:app:allow-set-dock-visibility\","
    "    \"core:app:allow-default-window-icon\","
    "    \"core:window:allow-is-enabled\", \"core:window:allow-set-enabled\","
    "    \"core:window:allow-set-theme\", \"core:window:allow-center\","
    "    \"core:window:allow-request-user-attention\","
    "    \"core:window:allow-set-resizable\", \"core:window:allow-set-maximizable\","
    "    \"core:window:allow-set-minimizable\", \"core:window:allow-set-closable\","
    "    \"core:window:allow-set-title\", \"core:window:allow-maximize\","
    "    \"core:window:allow-unmaximize\", \"core:window:allow-minimize\","
    "    \"core:window:allow-unminimize\", \"core:window:allow-show\","
    "    \"core:window:allow-hide\", \"core:window:allow-close\","
    "    \"core:window:allow-set-decorations\", \"core:window:allow-set-shadow\","
    "    \"core:window:allow-set-effects\", \"core:window:allow-set-always-on-top\","
    "    \"core:window:allow-set-always-on-bottom\","
    "    \"core:window:allow-set-content-protected\","
    "    \"core:window:allow-set-size\", \"core:window:allow-set-min-size\","
    "    \"core:window:allow-set-max-size\", \"core:window:allow-set-position\","
    "    \"core:window:allow-set-fullscreen\","
    "    \"core:window:allow-set-simple-fullscreen\","
    "    \"core:window:allow-set-focus\", \"core:window:allow-set-focusable\","
    "    \"core:window:allow-set-skip-taskbar\", \"core:window:allow-set-cursor-grab\","
    "    \"core:window:allow-set-cursor-visible\", \"core:window:allow-set-cursor-icon\","
    "    \"core:window:allow-set-cursor-position\","
    "    \"core:window:allow-set-ignore-cursor-events\","
    "    \"core:window:allow-start-dragging\", \"core:window:allow-set-progress-bar\","
    "    \"core:window:allow-set-icon\", \"core:window:allow-toggle-maximize\","
    "    \"core:webview:allow-create-webview-window\", \"core:webview:allow-print\""
    "  ]"
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

  /* Serve the shared examples/api frontend (built into examples/api/dist). */
  if (check(tauri_app_builder_set_assets_dir(builder, ASSETS_DIR), "set_assets_dir")) return 1;

  /* Grant the frontend the core permissions it uses (window, app, webview). */
  if (check(tauri_app_builder_add_capability(builder, CAPABILITY), "add_capability")) return 1;

  uint64_t app = 0;
  if (check(tauri_app_build(builder, &app), "app_build")) return 1;

  int32_t exit_code = 0;
  if (check(tauri_app_run(app, &exit_code), "app_run")) return 1;

  printf("app exited with code %d\n", exit_code);
  tauri_handle_close(app);
  return exit_code;
}
