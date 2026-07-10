# Build the app against the tauri-ffi C library.
#
# Point TAURI_FFI_DIR at a directory containing the tauri-ffi header
# (tauri_ffi.h) and the built library (libtauri_ffi.{dylib,so} / tauri_ffi.dll):
#
#   make TAURI_FFI_DIR=/path/to/tauri-ffi
#
# In this repo that is `bindings/c` for the header and `target/debug` for the
# library after `cargo build -p tauri-ffi`.

TAURI_FFI_DIR ?= .
INCLUDE_DIR ?= $(TAURI_FFI_DIR)
LIB_DIR ?= $(TAURI_FFI_DIR)

CC ?= cc
CFLAGS += -I$(INCLUDE_DIR)
LDFLAGS += -L$(LIB_DIR) -ltauri_ffi

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
