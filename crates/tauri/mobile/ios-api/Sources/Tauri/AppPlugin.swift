// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import UIKit

struct SetEnabledArgs: Decodable {
  let enabled: Bool
}

/// The native side of Tauri's built-in `app` plugin.
class AppPlugin: Plugin {
  private var privacyScreenEnabled = false
  // The covers currently shown, one per window. Only accessed on the main thread.
  private var privacyCovers: [UIView] = []

  override init() {
    super.init()
    let center = NotificationCenter.default
    center.addObserver(
      self, selector: #selector(appWillResignActive),
      name: UIApplication.willResignActiveNotification, object: nil)
    center.addObserver(
      self, selector: #selector(appDidBecomeActive),
      name: UIApplication.didBecomeActiveNotification, object: nil)
  }

  deinit {
    NotificationCenter.default.removeObserver(self)
  }

  @objc public func openSettings(_ invoke: Invoke) {
    guard let url = URL(string: UIApplication.openSettingsURLString) else {
      invoke.reject("invalid settings URL")
      return
    }
    DispatchQueue.main.async {
      UIApplication.shared.open(url, options: [:]) { success in
        if success {
          invoke.resolve()
        } else {
          invoke.reject("failed to open the app settings")
        }
      }
    }
  }

  @objc public func setKeepScreenOn(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(SetEnabledArgs.self)
    DispatchQueue.main.async {
      UIApplication.shared.isIdleTimerDisabled = args.enabled
      invoke.resolve()
    }
  }

  @objc public func setPrivacyScreen(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(SetEnabledArgs.self)
    DispatchQueue.main.async {
      self.privacyScreenEnabled = args.enabled
      if !args.enabled {
        self.removePrivacyCovers()
      } else if UIApplication.shared.applicationState != .active {
        self.showPrivacyCovers()
      }
      invoke.resolve()
    }
  }

  // The app switcher snapshot is taken after the app resigns active, so covering the windows
  // here hides their content from it.
  @objc private func appWillResignActive() {
    if privacyScreenEnabled {
      showPrivacyCovers()
    }
  }

  @objc private func appDidBecomeActive() {
    removePrivacyCovers()
  }

  private func showPrivacyCovers() {
    guard privacyCovers.isEmpty else {
      return
    }
    for window in allWindows() {
      let style: UIBlurEffect.Style
      if #available(iOS 13.0, *) {
        style = .systemMaterial
      } else {
        style = .light
      }
      let cover = UIVisualEffectView(effect: UIBlurEffect(style: style))
      cover.frame = window.bounds
      cover.autoresizingMask = [.flexibleWidth, .flexibleHeight]
      window.addSubview(cover)
      privacyCovers.append(cover)
    }
  }

  private func removePrivacyCovers() {
    for cover in privacyCovers {
      cover.removeFromSuperview()
    }
    privacyCovers.removeAll()
  }

  private func allWindows() -> [UIWindow] {
    if #available(iOS 13.0, *) {
      return UIApplication.shared.connectedScenes
        .compactMap { $0 as? UIWindowScene }
        .flatMap { $0.windows }
    }
    return UIApplication.shared.windows
  }
}

@_cdecl("init_plugin_app")
public func initAppPlugin() -> Plugin {
  return AppPlugin()
}
