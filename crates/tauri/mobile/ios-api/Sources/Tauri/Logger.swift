// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import os.log
import UIKit

/// Wrapper class for os_log function
public class Logger {
  private static var _enabled = false
  public static var enabled: Bool {
    get {
      #if DEBUG
      return true
      #else
      return _enabled
      #endif
    }
    set {
      Logger._enabled = newValue
    }
  }

  static func log(_ items: Any..., category: String, type: OSLogType) {
    if Logger.enabled {
      var message = ""
      let last = items.count - 1
      for (index, item) in items.enumerated() {
        message += "\(item)"
        if index != last {
          message += " "
        }
      }
      let log = OSLog(subsystem: Bundle.main.bundleIdentifier ?? "-", category: category)
      os_log("%{public}@", log: log, type: type, String(message.prefix(4068)))
    }
  }

  public static func debug(_ items: Any..., category: String = "app") {
    #if DEBUG
    Logger.log(items, category: category, type: OSLogType.default)
    #else
    Logger.log(items, category: category, type: OSLogType.debug)
    #endif
  }

  public static func info(_ items: Any..., category: String = "app") {
    #if DEBUG
    Logger.log(items, category: category, type: OSLogType.default)
    #else
    Logger.log(items, category: category, type: OSLogType.info)
    #endif
  }

  public static func error(_ items: Any..., category: String = "app") {
    Logger.log(items, category: category, type: OSLogType.error)
  }
}
