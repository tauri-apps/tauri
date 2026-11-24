// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Private Qt classes that need moc.
#pragma once
#include <QObject>
#include <QString>
#include <functional>

class QsIpcBridge : public QObject {
  Q_OBJECT
public:
  explicit QsIpcBridge(std::function<void(const QString &)> sink, QObject *parent = nullptr)
      : QObject(parent), m_sink(std::move(sink)) {}

  Q_INVOKABLE void postMessage(const QString &message) {
    if (m_sink)
      m_sink(message);
  }

  void detach() { m_sink = nullptr; }

private:
  std::function<void(const QString &)> m_sink;
};
