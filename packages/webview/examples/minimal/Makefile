TARGET = minimal

CFLAGS ?= -std=c99 -Wall -Wextra -pedantic -I../..

TARGET_OS ?= $(OS)
ifeq ($(TARGET_OS),Windows_NT)
	TARGET=minimal.exe
	WEBVIEW_CFLAGS := -DWEBVIEW_WINAPI=1
	WEBVIEW_LDFLAGS := -lole32 -lcomctl32 -loleaut32 -luuid -mwindows
else ifeq ($(shell uname -s),Linux)
	WEBVIEW_CFLAGS := -DWEBVIEW_GTK=1 $(shell pkg-config --cflags gtk+-3.0 webkit2gtk-4.0)
	WEBVIEW_LDFLAGS := $(shell pkg-config --libs gtk+-3.0 webkit2gtk-4.0)
else ifeq ($(shell uname -s),Darwin)
	WEBVIEW_CFLAGS := -DWEBVIEW_COCOA=1
	WEBVIEW_LDFLAGS := -framework WebKit
endif

$(TARGET): main.c
	$(CC) $(CFLAGS) $(WEBVIEW_CFLAGS) main.c $(LDFLAGS) $(WEBVIEW_LDFLAGS) -o $@

clean:
	rm -f $(TARGET)

.PHONY: clean
