// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Application, event channel, screens and other app-wide pieces (always built).
#include "internal.h"

#include <QAbstractEventDispatcher>
#include <QApplication>
#include <QCursor>
#include <QDBusConnection>
#include <QDBusMessage>
#include <QGuiApplication>
#include <QList>
#include <QPointer>
#include <QScreen>
#include <QStyleHints>
#include <QVariantMap>
#include <QWidget>

#include <cstdlib>
#include <cstring>

using namespace qs;

namespace qs {
qs_event_fn g_event_fn = nullptr;
void *g_event_ud = nullptr;

static QList<void (*)()> &shutdownHooks() {
  static QList<void (*)()> hooks;
  return hooks;
}

int g_callbackDepth = 0;

void registerShutdownHook(void (*hook)()) {
  if (!shutdownHooks().contains(hook))
    shutdownHooks().append(hook);
}

void deleteTopLevelWidgets() {
  QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
  // The list also holds windows owned by other widgets (e.g. a menubar's QMenus, which are popup
  // windows), and deleting their parent deletes them too; QPointer skips those.
  QList<QPointer<QWidget>> widgets;
  for (QWidget *w : QApplication::topLevelWidgets())
    widgets.append(w);
  for (const QPointer<QWidget> &w : std::as_const(widgets))
    delete w.data();
  QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
}
} // namespace qs

void qs::fillScreen(QScreen *s, QsScreen *out) {
  std::memset(out, 0, sizeof(QsScreen));
  if (!s)
    return;
  const QByteArray n = s->name().toUtf8();
  std::strncpy(out->name, n.constData(), sizeof(out->name) - 1);
  const QRect g = s->geometry(), a = s->availableGeometry();
  out->x = g.x(); out->y = g.y(); out->w = g.width(); out->h = g.height();
  out->ax = a.x(); out->ay = a.y(); out->aw = a.width(); out->ah = a.height();
  out->scale = s->devicePixelRatio();
  out->primary = s == QGuiApplication::primaryScreen();
}

extern "C" {

const char *qs_app_init(const char *app_id) {
  static int argc = 1;
  static char arg0[] = "wry-qt";
  static char *argv[] = {arg0, nullptr};
  if (!QCoreApplication::instance()) {
    QCoreApplication::setAttribute(Qt::AA_ShareOpenGLContexts);
    if (app_id && *app_id) {
      QCoreApplication::setApplicationName(QString::fromUtf8(app_id));
      QGuiApplication::setDesktopFileName(QString::fromUtf8(app_id));
    }
    new QApplication(argc, argv);
    QApplication::setQuitOnLastWindowClosed(false);
    QObject::connect(QAbstractEventDispatcher::instance(), &QAbstractEventDispatcher::aboutToBlock,
                     qApp, [] { Ev(QS_EV_ABOUT_TO_BLOCK).fire(); });
    QObject::connect(QGuiApplication::styleHints(), &QStyleHints::colorSchemeChanged, qApp,
                     [](Qt::ColorScheme scheme) {
                       Ev ev(QS_EV_THEME);
                       ev.e.a = scheme == Qt::ColorScheme::Dark ? 2 : 1;
                       ev.fire();
                     });
  }
  static QByteArray name;
  name = QByteArray(QAbstractEventDispatcher::instance()->metaObject()->className()) + " on " +
         QGuiApplication::platformName().toUtf8();
  return name.constData();
}

void qs_set_event_handler(qs_event_fn f, void *ud) {
  g_event_fn = f;
  g_event_ud = ud;
}

void qs_reply_set_int(QsReply *r, int64_t v) { r->i = v; }
void qs_reply_set_str(QsReply *r, const char *s, size_t len) {
  r->s = QByteArray(s, static_cast<qsizetype>(len));
  r->hasS = true;
}

int qs_app_exec(void) { return QApplication::exec(); }
void qs_app_quit(int code) { QCoreApplication::exit(code); }
void qs_app_process_events(void) { QCoreApplication::processEvents(); }

void qs_app_shutdown(void) {
  if (!QCoreApplication::instance())
    return;
  for (auto it = shutdownHooks().crbegin(); it != shutdownHooks().crend(); ++it)
    (*it)();
  shutdownHooks().clear();
  deleteTopLevelWidgets();
  g_event_fn = nullptr;
  delete qApp;
}

void qs_dispatch(qs_fn f, void *ud) {
  if (QCoreApplication *app = QCoreApplication::instance())
    QMetaObject::invokeMethod(
        app,
        [f, ud] {
          CallbackScope scope;
          f(ud);
        },
        Qt::QueuedConnection);
}

void qs_post(qs_fn f, void *ud) { qs_dispatch(f, ud); }

int qs_app_color_scheme(void) {
  switch (QGuiApplication::styleHints()->colorScheme()) {
  case Qt::ColorScheme::Light: return 1;
  case Qt::ColorScheme::Dark: return 2;
  default: return 0;
  }
}
void qs_app_set_color_scheme(int scheme) {
  if (scheme == 1)
    QGuiApplication::styleHints()->setColorScheme(Qt::ColorScheme::Light);
  else if (scheme == 2)
    QGuiApplication::styleHints()->setColorScheme(Qt::ColorScheme::Dark);
  else
    QGuiApplication::styleHints()->unsetColorScheme();
}


int qs_screens(QsScreen *out, int max) {
  const auto screens = QGuiApplication::screens();
  for (int i = 0; i < screens.size() && i < max; ++i)
    fillScreen(screens[i], &out[i]);
  return static_cast<int>(screens.size());
}

int qs_cursor_pos(int32_t *x, int32_t *y, double *scale) {
  const QPoint p = QCursor::pos();
  *x = p.x();
  *y = p.y();
  QScreen *s = QGuiApplication::screenAt(p);
  *scale = s ? s->devicePixelRatio() : 1.0;
  return 1;
}

int qs_native_display(int *kind, void **display) {
#if QT_CONFIG(wayland)
  if (auto *w = qApp->nativeInterface<QNativeInterface::QWaylandApplication>()) {
    *kind = 1;
    *display = w->display();
    return 1;
  }
#endif
#if QT_CONFIG(xcb)
  if (auto *x = qApp->nativeInterface<QNativeInterface::QX11Application>()) {
    *kind = 2;
    *display = x->display();
    return 1;
  }
#endif
  return 0;
}

void qs_launcher_update(const char *desktop_file, int64_t count, double progress) {
  QString app = QString::fromUtf8(desktop_file ? desktop_file : "");
  if (app.isEmpty())
    app = QGuiApplication::desktopFileName();
  if (!app.endsWith(QLatin1String(".desktop")))
    app += QLatin1String(".desktop");
  QVariantMap props;
  props.insert(QStringLiteral("count-visible"), count >= 0);
  if (count >= 0)
    props.insert(QStringLiteral("count"), static_cast<qlonglong>(count));
  props.insert(QStringLiteral("progress-visible"), progress >= 0);
  if (progress >= 0)
    props.insert(QStringLiteral("progress"), progress);
  QDBusMessage msg = QDBusMessage::createSignal(
      QStringLiteral("/"), QStringLiteral("com.canonical.Unity.LauncherEntry"),
      QStringLiteral("Update"));
  msg << QStringLiteral("application://") + app << props;
  QDBusConnection::sessionBus().send(msg);
}


void qs_free(void *p) { std::free(p); }

} // extern "C"
