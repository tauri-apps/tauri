// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// qtshim: a flat C API over Qt Widgets and QtWebEngine, consumed from Rust (`qtshim-sys`).
//
// Sections marked [webengine] are only compiled with the `webengine` feature.
//
// Threading: every function must be called on the main thread (the one that called
// qs_app_init) except qs_dispatch and qs_job_respond, which are thread-safe.
//
// Coordinates passed through this API are Qt logical (device-independent) pixels unless
// stated otherwise; Rust converts using the window's device pixel ratio.
#pragma once
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct QsWindow QsWindow;
typedef struct QsView QsView;
typedef struct QsProfile QsProfile;
typedef struct QsJob QsJob;
typedef struct QsReply QsReply;

typedef void (*qs_fn)(void *ud);
typedef void (*qs_drop_fn)(void *ud);
typedef void (*qs_str_fn)(void *ud, const char *s, size_t len);

// ---------------------------------------------------------------------------------------------
// events

enum {
  QS_EV_WINDOW_RESIZED = 1,    // a,b = physical inner w,h (below a menubar; also sent when one is shown/hidden)
  QS_EV_WINDOW_MOVED = 2,      // a,b = physical outer x,y
  QS_EV_WINDOW_CLOSE_REQ = 3,  // user asked to close (the close is always blocked; Rust decides)
  QS_EV_WINDOW_FOCUS = 4,      // a = focused
  QS_EV_WINDOW_SCALE = 5,      // f = scale, a,b = physical inner w,h
  QS_EV_THEME = 6,             // a = 1 light, 2 dark (app wide, window = 0)

  QS_EV_VIEW_IPC = 20,         // s1 = message, s2 = page url
  QS_EV_VIEW_TITLE = 21,       // s1 = title
  QS_EV_VIEW_LOAD = 22,        // a = 0 started / 1 finished, s1 = url
  QS_EV_VIEW_NAVIGATE = 23,    // s1 = url; return 0 to block
  QS_EV_VIEW_NEW_WINDOW = 24,  // s1 = url, a,b,c,d = requested logical geometry (0 if unset)
                               // return 0 deny, 1 allow (popup window), 2 open in view reply.i
  QS_EV_VIEW_PERMISSION = 25,  // s1 = origin, a = QWebEnginePermission::PermissionType
                               // return 1 grant, 0 deny, -1 ask the user
  QS_EV_VIEW_DOWNLOAD = 26,    // s1 = url, s2 = suggested absolute path
                               // return 0 cancel, 1 accept; reply.s = destination path
  QS_EV_VIEW_DOWNLOAD_DONE = 27, // s1 = url, s2 = path, a = success
  QS_EV_VIEW_DRAG = 28,        // a = 0 enter,1 over,2 drop,3 leave; b,c = physical x,y;
                               // s1 = '\n'-joined paths; return 1 to swallow the event
  QS_EV_VIEW_CRASHED = 29,     // renderer died: a = QWebEnginePage::RenderProcessTerminationStatus,
                               // b = exit code; return 1 if handled (otherwise the page reloads)
  QS_EV_ABOUT_TO_BLOCK = 40,   // the event loop is about to wait
};

typedef struct QsEvent {
  int32_t kind;
  uint64_t window;
  uint64_t view;
  int64_t a, b, c, d;
  double f;
  const char *s1;
  size_t s1_len;
  const char *s2;
  size_t s2_len;
} QsEvent;

// Return value semantics are per event kind (see above); unknown kinds ignore it.
typedef int (*qs_event_fn)(void *ud, const QsEvent *ev, QsReply *reply);
void qs_reply_set_int(QsReply *r, int64_t v);
void qs_reply_set_str(QsReply *r, const char *s, size_t len);

// [webengine] Custom scheme request. `view` is the id of the requesting view (0 if unknown).
// `headers` is "Name: value\n" joined.
typedef void (*qs_scheme_fn)(void *ud, QsJob *job, uint64_t view, const char *method,
                             const char *url, const char *headers, const uint8_t *body,
                             size_t body_len);

// ---------------------------------------------------------------------------------------------
// application

// Create the QApplication. `app_id` (may be NULL) becomes the application name and the
// Wayland app_id / desktop file name. Returns "<dispatcher> on <platform>".
const char *qs_app_init(const char *app_id);
void qs_set_event_handler(qs_event_fn f, void *ud);
// [webengine]
void qs_set_scheme_handler(qs_scheme_fn f, void *ud);
// [webengine] flags: bit0 secure, bit1 cors, bit2 fetch api, bit3 local access, bit4 service workers,
//        bit5 csp bypass. Only effective before the web engine starts (first profile).
// Returns 0 if it was too late.
int qs_register_scheme(const char *name, uint32_t flags);
// [webengine] Append Chromium command line flags. Only effective before the web engine starts.
void qs_engine_add_flags(const char *flags);
int qs_engine_started(void);
// [webengine] "<QtWebEngine version> (Chromium <version>)", static string.
const char *qs_engine_version(void);
// [webengine] type: 0 none/system, 1 http, 2 socks5. Only effective before the web engine starts.
void qs_set_proxy(int type, const char *host, uint16_t port);
int qs_app_exec(void);
void qs_app_quit(int code);
void qs_app_process_events(void);
void qs_app_shutdown(void);
// Run `f(ud)` on the main thread from the event loop. Does nothing without a QApplication; calls
// still queued when it's destroyed never run (the caller owns `ud` and must free it then).
void qs_dispatch(qs_fn f, void *ud);
// The same, for the main thread itself.
void qs_post(qs_fn f, void *ud);
// 0 unknown, 1 light, 2 dark
int qs_app_color_scheme(void);
void qs_app_set_color_scheme(int scheme);
// Free a string returned by this API.
void qs_free(void *p);

typedef struct QsScreen {
  char name[128];
  int32_t x, y, w, h;      // logical geometry
  int32_t ax, ay, aw, ah;  // logical available geometry
  double scale;
  int32_t primary;
} QsScreen;
// Fills up to `max` screens, returns the total count.
int qs_screens(QsScreen *out, int max);
// Logical cursor position + scale of the screen under it. Returns 0 if unknown.
int qs_cursor_pos(int32_t *x, int32_t *y, double *scale);
// kind: 1 wayland (wl_display*), 2 x11 (Display*). Returns 0 if unavailable.
int qs_native_display(int *kind, void **display);
// Unity LauncherEntry (badge / progress). count < 0 hides it, progress < 0 hides it.
void qs_launcher_update(const char *desktop_file, int64_t count, double progress);

// ---------------------------------------------------------------------------------------------
// windows [webengine]

enum {
  QS_WIN_TRANSPARENT = 1,
};
QsWindow *qs_window_new(uint64_t id, uint32_t flags);
void qs_window_destroy(QsWindow *w);
void qs_window_set_title(QsWindow *w, const char *title);
char *qs_window_title(QsWindow *w); // free with qs_free
void qs_window_set_visible(QsWindow *w, int visible);
// out: inner x,y,w,h then outer x,y,w,h (logical)
void qs_window_geometry(QsWindow *w, int32_t out[8]);
double qs_window_scale(QsWindow *w);
void qs_window_screen(QsWindow *w, QsScreen *out);
void qs_window_resize(QsWindow *w, int32_t width, int32_t height);
void qs_window_move(QsWindow *w, int32_t x, int32_t y);
// < 0 means unset
void qs_window_set_min_size(QsWindow *w, int32_t width, int32_t height);
void qs_window_set_max_size(QsWindow *w, int32_t width, int32_t height);
void qs_window_set_resizable(QsWindow *w, int resizable);

enum {
  QS_STATE_MINIMIZED = 1,
  QS_STATE_MAXIMIZED = 2,
  QS_STATE_FULLSCREEN = 4,
  QS_STATE_ACTIVE = 8,
  QS_STATE_VISIBLE = 16,
  QS_STATE_ENABLED = 32,
  QS_STATE_DECORATED = 64,
  QS_STATE_ON_TOP = 128,
  QS_STATE_RESIZABLE = 256,
  QS_STATE_CLOSABLE = 512,
  QS_STATE_MINIMIZABLE = 1024,
  QS_STATE_MAXIMIZABLE = 2048,
};
uint32_t qs_window_state(QsWindow *w);
// which: one of MINIMIZED, MAXIMIZED, FULLSCREEN
void qs_window_set_state(QsWindow *w, uint32_t which, int on);

enum {
  QS_FLAG_DECORATIONS = 1,
  QS_FLAG_ON_TOP = 2,
  QS_FLAG_ON_BOTTOM = 3,
  QS_FLAG_FOCUSABLE = 4,
  QS_FLAG_CLOSABLE = 5,
  QS_FLAG_MINIMIZABLE = 6,
  QS_FLAG_MAXIMIZABLE = 7,
  QS_FLAG_IGNORE_CURSOR = 8,
  QS_FLAG_SKIP_TASKBAR = 9,
};
void qs_window_set_flag(QsWindow *w, uint32_t flag, int on);
void qs_window_set_enabled(QsWindow *w, int enabled);
void qs_window_focus(QsWindow *w);
void qs_window_center(QsWindow *w);
void qs_window_set_icon(QsWindow *w, const uint8_t *rgba, int32_t width, int32_t height);
void qs_window_set_background(QsWindow *w, int has_color, uint8_t r, uint8_t g, uint8_t b,
                              uint8_t a);
// shape: Qt::CursorShape value
void qs_window_set_cursor(QsWindow *w, int shape);
void qs_window_set_cursor_visible(QsWindow *w, int visible);
void qs_window_set_cursor_grab(QsWindow *w, int grab);
void qs_window_set_cursor_pos(QsWindow *w, int32_t x, int32_t y);
int qs_window_start_move(QsWindow *w);
// edges: Qt::Edges bitmask (Top=1, Left=2, Right=4, Bottom=8)
int qs_window_start_resize(QsWindow *w, uint32_t edges);
void qs_window_alert(QsWindow *w, int critical);
// Fullscreen on the screen containing the logical point.
void qs_window_fullscreen_on(QsWindow *w, int32_t x, int32_t y);
void qs_window_set_transient_for(QsWindow *w, QsWindow *parent);
// kind: 1 wayland (wl_surface*), 2 x11 (Window id in *handle). Returns 0 if unavailable.
int qs_window_native(QsWindow *w, int *kind, void **handle);
// The window's QWidget* (for muda's Qt backend / tauri's `qt_window`).
void *qs_window_qwidget(QsWindow *w);
// Logical size of the area holding the webviews (the window minus its menubar, if any).
void qs_window_content_size(QsWindow *w, int32_t *width, int32_t *height);
// Debugging: move the mouse (no buttons) to logical (x, y) in the window through Qt's input path,
// like a real pointer, and return the shape of the application's override cursor (a
// Qt::CursorShape), -1 if there's none, or -2 if unsupported (no QPA headers or no native window).
int qs_debug_window_hover(QsWindow *w, int32_t x, int32_t y);
// Make `w` transient for the window containing `parent` (any QWidget*).
void qs_window_set_transient_for_widget(QsWindow *w, void *parent);


// ---------------------------------------------------------------------------------------------
// profiles (one per data directory / incognito; starts the web engine on first use) [webengine]

QsProfile *qs_profile_get(const char *storage_dir, int incognito);
void qs_profile_set_user_agent(QsProfile *p, const char *ua);
// Calls f once per cookie with its Set-Cookie representation.
void qs_profile_cookies(QsProfile *p, qs_str_fn f, void *ud);
void qs_profile_set_cookie(QsProfile *p, const char *set_cookie, const char *origin);
void qs_profile_delete_cookie(QsProfile *p, const char *set_cookie, const char *origin);
void qs_profile_clear_data(QsProfile *p);

// ---------------------------------------------------------------------------------------------
// views (QWebEngineView children of a window) [webengine]

enum {
  QS_VIEW_DEVTOOLS = 1,
  QS_VIEW_TRANSPARENT = 2,
  QS_VIEW_FILL = 4,           // follow the window's client area
  QS_VIEW_NO_JAVASCRIPT = 8,
  QS_VIEW_CLIPBOARD = 16,
  QS_VIEW_DRAG_HANDLER = 32,  // report file drags as QS_EV_VIEW_DRAG
  QS_VIEW_FOCUS = 64,
};
QsView *qs_view_new(uint64_t id, QsWindow *parent, QsProfile *profile, uint32_t flags);
void qs_view_destroy(QsView *v);
void qs_view_reparent(QsView *v, QsWindow *parent);
void qs_view_set_fill(QsView *v, int fill);
void qs_view_set_bounds(QsView *v, int32_t x, int32_t y, int32_t w, int32_t h);
void qs_view_bounds(QsView *v, int32_t out[4]);
void qs_view_set_visible(QsView *v, int visible);
void qs_view_focus(QsView *v);
// headers: "Name: value\n" joined, may be NULL
void qs_view_load_url(QsView *v, const char *url, const char *headers);
void qs_view_load_html(QsView *v, const char *html, const char *base_url);
void qs_view_reload(QsView *v);
void qs_view_back(QsView *v);
void qs_view_forward(QsView *v);
int qs_view_can_go_back(QsView *v);
int qs_view_can_go_forward(QsView *v);
void qs_view_set_zoom(QsView *v, double factor);
void qs_view_set_background(QsView *v, uint8_t r, uint8_t g, uint8_t b, uint8_t a);
void qs_view_add_init_script(QsView *v, const char *js, int main_frame_only);
// Evaluate JS; callback (if any) receives the JSON-serialised result.
void qs_view_eval(QsView *v, const char *js, qs_str_fn cb, void *ud, qs_drop_fn drop);
char *qs_view_url(QsView *v); // free with qs_free
void qs_view_open_devtools(QsView *v);
void qs_view_close_devtools(QsView *v);
int qs_view_is_devtools_open(QsView *v);
void qs_view_print(QsView *v);
// Raw pointers for advanced users: QWebEngineView* / QWebEnginePage*.
void *qs_view_qt_view(QsView *v);
void *qs_view_qt_page(QsView *v);

// ---------------------------------------------------------------------------------------------
// custom scheme responses [webengine]

// Thread-safe. Consumes the job. headers: "Name: value\n" joined (may be NULL).
// QtWebEngine cannot report arbitrary status codes: 3xx + Location redirects, an empty 404
// fails with "not found", everything else is served. For a 206 pass the Content-Range start
// and total size so Qt (which applies the Range header itself) sees the full resource.
void qs_job_respond(QsJob *job, int status, const char *content_type, const char *headers,
                    const uint8_t *data, size_t len, int64_t range_start, int64_t range_total);

// ---------------------------------------------------------------------------------------------
// menus

// A QPointer<QWidget>: tells whether a widget has been destroyed. qs_widget_get returns the
// widget or NULL. Free with qs_widget_untrack.
// `ptr` if it is a live QWidget of this application, NULL otherwise. Never dereferences `ptr`, so
// any value may be passed.
void *qs_widget_find(void *ptr);
void *qs_widget_track(void *widget);
void *qs_widget_get(void *tracker);
void qs_widget_untrack(void *tracker);

// The QMenuBar this API installed on `widget` (QMainWindow::setMenuBar, or the top of the widget's
// layout via QLayout::setMenuBar; a QVBoxLayout is created if it has none). With `create` it is
// created if missing. Returns NULL if there is none.
void *qs_menubar_for(void *widget, int create);
void qs_menubar_remove(void *widget);
void qs_menubar_set_visible(void *widget, int visible);
int qs_menubar_is_visible(void *widget);

// "Containers" are QMenuBar* or QMenu*.
void qs_container_clear(void *container);
// Returns the new QMenu*. `rgba` (may be NULL) is a w*h RGBA icon.
void *qs_container_add_menu(void *container, const char *text, int enabled, const uint8_t *rgba,
                            int w, int h);
// `shortcut` (QKeySequence portable text) is registered; `shortcut_hint` is only displayed.
// `on_click(ud)` runs when triggered and `drop(ud)` when the action is destroyed (both optional).
void qs_container_add_action(void *container, const char *text, int enabled, int checkable,
                             int checked, const char *shortcut, const char *shortcut_hint,
                             const uint8_t *rgba, int w, int h, qs_fn on_click, void *ud,
                             qs_drop_fn drop);
void qs_container_add_separator(void *container);

// A context menu (QMenu*) parented to `widget`; it deletes itself when closed.
void *qs_popup_menu_new(void *widget);
// Show it at (x, y) relative to the window's top-left corner (`physical`: physical pixels), or at
// the cursor without `has_pos`. Doesn't block.
void qs_popup_menu_show(void *menu, void *widget, int has_pos, double x, double y, int physical);

enum {
  QS_ACTION_COPY = 1,
  QS_ACTION_CUT,
  QS_ACTION_PASTE,
  QS_ACTION_SELECT_ALL,
  QS_ACTION_UNDO,
  QS_ACTION_REDO,
  QS_ACTION_DELETE,
  QS_ACTION_MINIMIZE,
  QS_ACTION_MAXIMIZE,
  QS_ACTION_FULLSCREEN,
  QS_ACTION_HIDE,
  QS_ACTION_CLOSE_WINDOW,
  QS_ACTION_QUIT,
};
// Perform a standard action on the focused widget / active window.
void qs_standard_action(int kind);
// A non-blocking About box. `html` is rich text; `rgba` (may be NULL) is a w*h RGBA icon.
void qs_show_about(const char *title, const char *html, const uint8_t *rgba, int w, int h);

// Debugging helpers: dump the menubar of `widget` as an indented tree (free with qs_free), and
// trigger the item at `path` ("File/Save"). Return value of trigger: 1 if found.
char *qs_debug_menubar(void *widget);
int qs_debug_trigger(void *widget, const char *path);
// Dump the last context menu Qt showed (empty if none since the previous call) and close it if
// it's still open. Recording starts with the first call to a qs_debug_* function.
char *qs_debug_take_popup(void);

#ifdef __cplusplus
}
#endif
