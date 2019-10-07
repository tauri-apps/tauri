#if defined(WEBVIEW_GTK)
#include "tauri-gtk-webview.h"
#elif defined(WEBVIEW_WINAPI)
#define CINTERFACE
#include "tauri-windows-webview.h"
#elif defined(WEBVIEW_COCOA)
#include "tauri-cocoa-webview.h"
#else
#error "Define one of: WEBVIEW_GTK, WEBVIEW_COCOA or WEBVIEW_WINAPI"
#endif

#define WEBVIEW_IMPLEMENTATION
#include "tauri.h"

void wrapper_webview_free(struct webview* w) {
	free(w);
}

struct webview* wrapper_webview_new(const char* title, const char* url, int width, int height, int resizable, int debug, webview_external_invoke_cb_t external_invoke_cb, void* userdata) {
	struct webview* w = (struct webview*)calloc(1, sizeof(*w));
	w->width = width;
	w->height = height;
	w->title = title;
	w->url = url;
	w->resizable = resizable;
	w->debug = debug;
	w->external_invoke_cb = external_invoke_cb;
	w->userdata = userdata;
	if (webview_init(w) != 0) {
		wrapper_webview_free(w);
		return NULL;
	}
	return w;
}

void* wrapper_webview_get_userdata(struct webview* w) {
	return w->userdata;
}

