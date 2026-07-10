/* Minimal tauri-ffi consumer: reads tauri.conf.json, opens the window it
 * declares, and runs until the app exits.
 *
 * Unlike the Node/Deno/Python bindings, a C app is compiled and linked against
 * the tauri-ffi library — it is not driven by `tauri dev`. Build it with the
 * Makefile in this directory (see README.md), then run the produced binary.
 */

#include <stdio.h>
#include <stdlib.h>
#include "tauri_ffi.h"

/* Reads a whole file into a NUL-terminated, caller-freed buffer. */
static char *read_file(const char *path) {
  FILE *file = fopen(path, "rb");
  if (!file) {
    fprintf(stderr, "could not open %s\n", path);
    return NULL;
  }
  fseek(file, 0, SEEK_END);
  long size = ftell(file);
  fseek(file, 0, SEEK_SET);
  char *buffer = malloc(size + 1);
  if (buffer && fread(buffer, 1, size, file) == (size_t)size) {
    buffer[size] = '\0';
  } else {
    free(buffer);
    buffer = NULL;
  }
  fclose(file);
  return buffer;
}

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

  char *config = read_file("tauri.conf.json");
  if (!config) return 1;

  uint64_t builder = 0;
  if (check(tauri_app_builder_new(config, &builder), "builder_new")) return 1;
  free(config);

  /* Serve a local frontend by pointing a window at `index.html` in the config
   * and uncommenting the next line (path relative to the working directory):
   * tauri_app_builder_set_assets_dir(builder, "../dist"); */

  uint64_t app = 0;
  if (check(tauri_app_build(builder, &app), "app_build")) return 1;

  int32_t exit_code = 0;
  if (check(tauri_app_run(app, &exit_code), "app_run")) return 1;

  tauri_handle_close(app);
  return exit_code;
}
