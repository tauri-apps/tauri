// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import os.log
import UIKit
import Foundation

class StdoutRedirector {
  private var originalStdout: Int32 = -1
  private var originalStderr: Int32 = -1
  private var stdoutPipe: [Int32] = [-1, -1]
  private var stderrPipe: [Int32] = [-1, -1]
  private var stdoutReadSource: DispatchSourceRead?
  private var stderrReadSource: DispatchSourceRead?
    
  func start() {
    originalStdout = dup(STDOUT_FILENO)
    originalStderr = dup(STDERR_FILENO)
        
    guard Darwin.pipe(&stdoutPipe) == 0,
      Darwin.pipe(&stderrPipe) == 0 else {
      Logger.error("Failed to create stdout/stderr pipes")
      return
    }
        
    dup2(stdoutPipe[1], STDOUT_FILENO)
    dup2(stderrPipe[1], STDERR_FILENO)
    close(stdoutPipe[1])
    close(stderrPipe[1])
        
    stdoutReadSource = createReader(
      readPipe: stdoutPipe[0],
      writeToOriginal: originalStdout,
      label: "stdout"
    )
        
    stderrReadSource = createReader(
      readPipe: stderrPipe[0],
      writeToOriginal: originalStderr,
      label: "stderr"
    )
  }
    
  private func createReader(
    readPipe: Int32,
    writeToOriginal: Int32,
    label: String
  ) -> DispatchSourceRead {
    let source = DispatchSource.makeReadSource(
      fileDescriptor: readPipe,
      queue: .global(qos: .utility)
    )
        
    source.setEventHandler {
      let bufferSize = 4096
      var buffer = [UInt8](repeating: 0, count: bufferSize)
      let bytesRead = read(readPipe, &buffer, bufferSize)
            
      if bytesRead > 0 {
        let output = String(
          bytes: buffer[0..<bytesRead],
          encoding: .utf8
        ) ?? ""
                
        let trimmed = output.trimmingCharacters(in: .newlines)
        if !trimmed.isEmpty {
          // we're sending stderr to oslog, so we need to avoid recursive calls
          if trimmed.hasPrefix("OSLOG-") {
            // make sure the system can parse the oslogs
            write(writeToOriginal, &buffer, bytesRead)
          } else {
            Logger.info("[\(label)] \(trimmed)")
          }
        }
      }
    } 
        
    source.setCancelHandler {
      close(readPipe)
    }
        
    source.resume()
    return source
  }
}

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

  static func log(_ items: [Any], category: String, type: OSLogType) {
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
