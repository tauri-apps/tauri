# Build the app against the tauri-ffi C library.
#
# RUNTIME picks the webview runtime and must match `app > runtime` in
# tauri.conf.json (wry or cef). Point TAURI_FFI_DIR at a directory containing the
# tauri-ffi header (tauri_ffi.h) and the library for that runtime
# (libtauri_$(RUNTIME).{dylib,so} / tauri_$(RUNTIME).dll):
#
#   make TAURI_FFI_DIR=/path/to/tauri-ffi RUNTIME=wry
#
# In this repo that is `bindings/c` for the header and `target/debug` for the
# library after `cargo build -p tauri-ffi` — which emits the runtime-agnostic
# `libtauri_ffi`, so build with `RUNTIME=ffi` against a local cargo build.

RUNTIME ?= wry
TAURI_FFI_DIR ?= .
INCLUDE_DIR ?= $(TAURI_FFI_DIR)
LIB_DIR ?= $(TAURI_FFI_DIR)

CC ?= cc
CFLAGS += -I$(INCLUDE_DIR)
LDFLAGS += -L$(LIB_DIR) -ltauri_$(RUNTIME)

UNAME := $(shell uname)
ifeq ($(UNAME), Linux)
  LDFLAGS += -Wl,-rpath,$(LIB_DIR)
endif

app: main.c
	$(CC) $(CFLAGS) main.c $(LDFLAGS) -o app

run: app
	./app

clean:
	rm -f app

.PHONY: run clean
