// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// A Node-API addon exposing the one tauri-ffi call that cannot go through
// koffi: the blocking `tauri_app_run`.
//
// koffi executes every call on a private stack it allocates itself (its
// `sync_stack_size` option), not the calling thread's real stack. WebKit's
// JavaScriptCore requires the opposite: it asserts that its stack lies within
// the bounds pthread reports for the current thread, because it scans exactly
// that range conservatively for GC roots. Running the event loop under a koffi
// call therefore aborts in `sanitizeStackForVM` as soon as a webview creates
// its JS context — and the assert is load-bearing, since scanning the wrong
// range would silently miss roots instead of crashing. A Node-API call runs on
// the thread's real stack, so the loop is safe here.
//
// Only the run loop needs this. Every other binding call either never reaches
// JSC, or is proxied onto the event loop thread by Tauri and so runs on this
// stack anyway. Runtimes with their own JS engine (cef) are unaffected, but go
// through here too so there is one path to reason about.

#include <node_api.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <dlfcn.h>
#endif

typedef int32_t (*tauri_app_run_fn)(uint64_t app, int32_t *out_exit_code);

static void throw_last_error(napi_env env, const char *what, const char *path) {
  char message[1024];
#ifdef _WIN32
  snprintf(message, sizeof(message), "%s (%s): error %lu", what, path,
           (unsigned long)GetLastError());
#else
  const char *reason = dlerror();
  snprintf(message, sizeof(message), "%s (%s): %s", what, path,
           reason ? reason : "unknown error");
#endif
  napi_throw_error(env, NULL, message);
}

/// Resolves `tauri_app_run` from the library at `path`.
///
/// The library is already loaded — the bindings dlopen it through koffi to make
/// every other call — so this only takes a second reference to the same image,
/// which matters: the app handle `run` is given lives in that instance's state.
static tauri_app_run_fn resolve_app_run(napi_env env, const char *path) {
#ifdef _WIN32
  // LoadLibraryA interprets `path` in the active ANSI code page; koffi loaded
  // this very file through the wide API, so convert the UTF-8 path to UTF-16 and
  // do the same, or a non-ASCII install path would fail to load here.
  int wlen = MultiByteToWideChar(CP_UTF8, 0, path, -1, NULL, 0);
  wchar_t *wpath = wlen > 0 ? (wchar_t *)malloc((size_t)wlen * sizeof(wchar_t)) : NULL;
  if (!wpath || MultiByteToWideChar(CP_UTF8, 0, path, -1, wpath, wlen) == 0) {
    free(wpath);
    throw_last_error(env, "failed to load the tauri-ffi library", path);
    return NULL;
  }
  HMODULE handle = LoadLibraryW(wpath);
  free(wpath);
  if (!handle) {
    throw_last_error(env, "failed to load the tauri-ffi library", path);
    return NULL;
  }
  tauri_app_run_fn run = (tauri_app_run_fn)GetProcAddress(handle, "tauri_app_run");
  if (!run) {
    throw_last_error(env, "tauri_app_run not found in", path);
    FreeLibrary(handle);  // release the reference this lookup took
    return NULL;
  }
#else
  void *handle = dlopen(path, RTLD_LAZY | RTLD_GLOBAL);
  if (!handle) {
    throw_last_error(env, "failed to load the tauri-ffi library", path);
    return NULL;
  }
  tauri_app_run_fn run = (tauri_app_run_fn)dlsym(handle, "tauri_app_run");
  if (!run) {
    throw_last_error(env, "tauri_app_run not found in", path);
    dlclose(handle);  // release the reference this lookup took
    return NULL;
  }
#endif
  return run;
}

/// run(libPath: string, app: number | bigint) -> { status, exitCode }
///
/// Blocks the calling thread until the app exits, exactly like the koffi call
/// it replaces — the caller is expected to be the parked main thread.
static napi_value run(napi_env env, napi_callback_info info) {
  size_t argc = 2;
  napi_value argv[2];
  if (napi_get_cb_info(env, info, &argc, argv, NULL, NULL) != napi_ok) {
    return NULL;
  }
  if (argc < 2) {
    napi_throw_type_error(env, NULL, "run(libPath, app) takes two arguments");
    return NULL;
  }

  // Size the buffer to the actual path length rather than a fixed cap —
  // napi_get_value_string_utf8 silently truncates on overflow, and a truncated
  // path would load (or fail to load) the wrong file.
  size_t path_len = 0;
  if (napi_get_value_string_utf8(env, argv[0], NULL, 0, &path_len) != napi_ok) {
    napi_throw_type_error(env, NULL, "libPath must be a string");
    return NULL;
  }
  char *path = (char *)malloc(path_len + 1);
  if (!path) {
    napi_throw_error(env, NULL, "out of memory allocating the library path");
    return NULL;
  }
  if (napi_get_value_string_utf8(env, argv[0], path, path_len + 1, NULL) != napi_ok) {
    free(path);
    napi_throw_type_error(env, NULL, "libPath must be a string");
    return NULL;
  }

  // The handle is a u64 in the ABI, but the registry hands out small counters,
  // so it arrives as a plain number; accept a bigint too rather than depend on
  // how the caller's FFI decoded it.
  uint64_t app = 0;
  napi_valuetype type;
  if (napi_typeof(env, argv[1], &type) != napi_ok) {
    free(path);
    return NULL;
  }
  if (type == napi_bigint) {
    bool lossless = false;
    if (napi_get_value_bigint_uint64(env, argv[1], &app, &lossless) != napi_ok || !lossless) {
      free(path);
      napi_throw_type_error(env, NULL, "app handle does not fit in a uint64");
      return NULL;
    }
  } else {
    int64_t signed_app = 0;
    if (napi_get_value_int64(env, argv[1], &signed_app) != napi_ok || signed_app < 0) {
      free(path);
      napi_throw_type_error(env, NULL, "app handle must be a non-negative number or bigint");
      return NULL;
    }
    app = (uint64_t)signed_app;
  }

  tauri_app_run_fn app_run = resolve_app_run(env, path);
  free(path);  // no longer needed; the resolved image stays referenced
  if (!app_run) {
    return NULL;  // resolve_app_run threw
  }

  int32_t exit_code = 0;
  int32_t status = app_run(app, &exit_code);  // blocks until the app exits

  napi_value result, status_value, exit_code_value;
  if (napi_create_object(env, &result) != napi_ok ||
      napi_create_int32(env, status, &status_value) != napi_ok ||
      napi_create_int32(env, exit_code, &exit_code_value) != napi_ok ||
      napi_set_named_property(env, result, "status", status_value) != napi_ok ||
      napi_set_named_property(env, result, "exitCode", exit_code_value) != napi_ok) {
    return NULL;
  }
  return result;
}

NAPI_MODULE_INIT() {
  napi_value run_fn;
  if (napi_create_function(env, "run", NAPI_AUTO_LENGTH, run, NULL, &run_fn) != napi_ok ||
      napi_set_named_property(env, exports, "run", run_fn) != napi_ok) {
    return NULL;
  }
  return exports;
}
