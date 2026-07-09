// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Low-level koffi bindings to the tauri-ffi C ABI. Pure JS — no node-gyp, no
// per-platform native glue. Hand-written for the M0 spike; generated from the
// API manifest from M3 (see /ffi-bindings-plan.md §6).

import { existsSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import koffi from 'koffi'

export const CODES = {
  OK: 0,
  GENERIC: -1,
  INVALID_HANDLE: -2,
  TIMEOUT: -3,
  CLOSED: -4,
  INVALID_ARG: -5,
  PANIC: -7
}

export function libraryPath() {
  if (process.env.TAURI_FFI_LIB) return process.env.TAURI_FFI_LIB
  const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..')
  const name = { darwin: 'libtauri_ffi.dylib', linux: 'libtauri_ffi.so', win32: 'tauri_ffi.dll' }[
    process.platform
  ]
  if (!name) throw new Error(`unsupported platform: ${process.platform}`)
  for (const profile of ['debug', 'release']) {
    const candidate = path.join(repoRoot, 'target', profile, name)
    if (existsSync(candidate)) return candidate
  }
  throw new Error(
    'tauri_ffi library not found — run `cargo build -p tauri-ffi` or set TAURI_FFI_LIB'
  )
}

export function open(libPath = libraryPath()) {
  const lib = koffi.load(libPath)

  const api = {
    version: lib.func('const char *tauri_ffi_version()'),
    abiVersion: lib.func('uint32_t tauri_ffi_abi_version()'),
    lastError: lib.func('const char *tauri_last_error_message()'),
    stringFree: lib.func('void tauri_string_free(void *str)'),
    handleClose: lib.func('int32_t tauri_handle_close(uint64_t handle)'),

    builderNew: lib.func('int32_t tauri_app_builder_new(const char *config, _Out_ uint64_t *out)'),
    builderSetAssetsDir: lib.func(
      'int32_t tauri_app_builder_set_assets_dir(uint64_t builder, const char *path)'
    ),
    builderRegisterCommand: lib.func(
      'int32_t tauri_app_builder_register_command(uint64_t builder, const char *name)'
    ),
    builderAddCapability: lib.func(
      'int32_t tauri_app_builder_add_capability(uint64_t builder, const char *capability)'
    ),
    appBuild: lib.func('int32_t tauri_app_build(uint64_t builder, _Out_ uint64_t *out)'),

    appRun: lib.func('int32_t tauri_app_run(uint64_t app, _Out_ int32_t *out_code)'),
    appExit: lib.func('int32_t tauri_app_exit(uint64_t app, int32_t code)'),

    eventsNext: lib.func(
      'int32_t tauri_events_next(uint64_t app, uint32_t timeout_ms, _Out_ void **out_json)'
    ),

    emit: lib.func('int32_t tauri_app_emit(uint64_t app, const char *event, const char *payload)'),
    emitTo: lib.func(
      'int32_t tauri_app_emit_to(uint64_t app, const char *label, const char *event, const char *payload)'
    ),
    listen: lib.func(
      'int32_t tauri_app_listen(uint64_t app, const char *event, _Out_ uint32_t *out_id)'
    ),
    unlisten: lib.func('int32_t tauri_app_unlisten(uint64_t app, uint32_t id)'),

    webviewEval: lib.func(
      'int32_t tauri_webview_eval(uint64_t app, const char *label, const char *js)'
    ),

    invokeResolve: lib.func('int32_t tauri_invoke_resolve(uint64_t resolver, const char *json)'),
    invokeReject: lib.func('int32_t tauri_invoke_reject(uint64_t resolver, const char *json)')
  }

  function check(code, what = 'tauri-ffi call') {
    if (code !== CODES.OK) {
      throw new Error(`${what} failed (${code}): ${api.lastError() ?? 'unknown error'}`)
    }
    return code
  }

  // Async so the worker's event loop stays free while we block on the queue —
  // koffi runs the call on the libuv thread pool and resolves a promise.
  function eventsNext(app, timeoutMs) {
    return new Promise((resolve, reject) => {
      const out = [null]
      api.eventsNext.async(app, timeoutMs, out, (err, code) => {
        if (err) return reject(err)
        if (code !== CODES.OK) return resolve({ code, json: null })
        const json = koffi.decode(out[0], 'char', -1)
        api.stringFree(out[0])
        resolve({ code, json })
      })
    })
  }

  return { api, check, eventsNext, libPath }
}
