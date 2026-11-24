// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Shared internals of the qtshim C++ sources (not part of the C API).
#pragma once
#include "qtshim.h"

#include <QString>
#include <QByteArray>
#include <QCoreApplication>
#include <QMetaObject>
#include <QPointer>
#include <QThread>

#include <cmath>
#include <cstdint>
#include <utility>

class QScreen;

struct QsReply {
  int64_t i = 0;
  QByteArray s;
  bool hasS = false;
};

namespace qs {

// The Rust event handler (set by qs_set_event_handler).
extern qs_event_fn g_event_fn;
extern void *g_event_ud;

inline bool onMainThread() {
  auto *app = QCoreApplication::instance();
  return app && QThread::currentThread() == app->thread();
}

template <typename F> void runOnMain(F &&f) {
  if (onMainThread())
    f();
  else
    QMetaObject::invokeMethod(QCoreApplication::instance(), std::forward<F>(f),
                              Qt::QueuedConnection);
}

inline bool inEventLoop() { return QThread::currentThread()->loopLevel() > 0; }

// How many calls from C++ into Rust are on the stack (see CallbackScope).
extern int g_callbackDepth;

// Marks a call into Rust (an event, a menu action, a scheme request, a closure). Rust may destroy
// any object meanwhile, including the one whose signal or virtual function is running.
struct CallbackScope {
  CallbackScope() { ++g_callbackDepth; }
  ~CallbackScope() { --g_callbackDepth; }
  CallbackScope(const CallbackScope &) = delete;
  CallbackScope &operator=(const CallbackScope &) = delete;
};

// Whether a Qt object may be deleted right away. Not inside the event loop (deleteLater is
// processed there) and not inside a call into Rust: the object may be the sender of the signal
// being handled, or be used by the C++ frames below. Otherwise deleteLater; outside any event
// loop such objects are deleted at the next event loop run or at qs_app_shutdown.
inline bool canDeleteNow() { return !inEventLoop() && g_callbackDepth == 0; }

// Delete `o` now if canDeleteNow(), else as soon as possible: deleteLater inside the event loop;
// outside it (e.g. with only qs_app_process_events) a queued call, which processEvents runs.
inline void deleteSoon(QObject *o) {
  if (canDeleteNow()) {
    delete o;
  } else if (inEventLoop()) {
    o->deleteLater();
  } else {
    QPointer<QObject> p(o);
    QMetaObject::invokeMethod(
        QCoreApplication::instance(), [p] { if (p) deleteSoon(p.data()); }, Qt::QueuedConnection);
  }
}

// Builder for an outgoing event; keeps the string payloads alive for the call.
struct Ev {
  QsEvent e{};
  QByteArray s1, s2;
  Ev(int kind, quint64 window = 0, quint64 view = 0) {
    e.kind = kind;
    e.window = window;
    e.view = view;
  }
  int fire(QsReply *reply = nullptr) {
    if (!g_event_fn)
      return -1;
    e.s1 = s1.constData();
    e.s1_len = static_cast<size_t>(s1.size());
    e.s2 = s2.constData();
    e.s2_len = static_cast<size_t>(s2.size());
    QsReply tmp;
    CallbackScope scope;
    return g_event_fn(g_event_ud, &e, reply ? reply : &tmp);
  }
};

inline int64_t phys(double v, double scale) {
  return static_cast<int64_t>(std::lround(v * scale));
}

// A Rust closure handed across the FFI boundary. Dropped exactly once.
struct Slot {
  qs_str_fn f = nullptr;
  void *ud = nullptr;
  qs_drop_fn drop = nullptr;
  Slot(qs_str_fn f, void *ud, qs_drop_fn drop) : f(f), ud(ud), drop(drop) {}
  Slot(const Slot &) = delete;
  Slot &operator=(const Slot &) = delete;
  ~Slot() {
    if (drop && ud)
      drop(ud);
  }
};

void fillScreen(QScreen *s, QsScreen *out);

// Run at qs_app_shutdown, latest registered first, before the QApplication is deleted.
void registerShutdownHook(void (*hook)());

// Delete every top-level widget now (and whatever was already deleteLater'd). Only for teardown,
// outside the event loop.
void deleteTopLevelWidgets();

} // namespace qs
