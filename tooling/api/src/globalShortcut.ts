// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Register global shortcuts.
 *
 * This package is also accessible with `window.__TAURI__.globalShortcut` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.globalShortcut`](https://tauri.app/v1/api/config/#allowlistconfig.globalshortcut) in `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "globalShortcut": {
 *         "all": true // enable all global shortcut APIs
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'
import { transformCallback } from './tauri'

export type ShortcutHandler = (shortcut: string) => void

/**
 * A keyboard shortcut that consists of an optional combination
 * of modifier keys and one key, eg: `Control+Shift+KeyX`.
 *
 * Possible Modifiers:
 *   - Option
 *   - Alt
 *   - Control
 *   - CTRL
 *   - Command
 *   - CMD
 *   - Super
 *   - Shift
 *   - CommandOrControl
 *   - CommandOrCTRL
 *   - CMDOrCTRL
 *   - CMDOrControl
 *
 * Possible Keys (Note that not all keys listed below will not work on all platforms, if you run into this, please file an issue so we could update the docs):
 *   - Backquote
 *   - Backslash
 *   - BracketLeft
 *   - BracketRight
 *   - Comma
 *   - Digit0
 *   - Digit1
 *   - Digit2
 *   - Digit3
 *   - Digit4
 *   - Digit5
 *   - Digit6
 *   - Digit7
 *   - Digit8
 *   - Digit9
 *   - Equal
 *   - IntlBackslash
 *   - IntlRo
 *   - IntlYen
 *   - KeyA
 *   - KeyB
 *   - KeyC
 *   - KeyD
 *   - KeyE
 *   - KeyF
 *   - KeyG
 *   - KeyH
 *   - KeyI
 *   - KeyJ
 *   - KeyK
 *   - KeyL
 *   - KeyM
 *   - KeyN
 *   - KeyO
 *   - KeyP
 *   - KeyQ
 *   - KeyR
 *   - KeyS
 *   - KeyT
 *   - KeyU
 *   - KeyV
 *   - KeyW
 *   - KeyX
 *   - KeyY
 *   - KeyZ
 *   - Minus
 *   - Period
 *   - Quote
 *   - Semicolon
 *   - Slash
 *   - AltLeft
 *   - AltRight
 *   - Backspace
 *   - CapsLock
 *   - ContextMenu
 *   - ControlLeft
 *   - ControlRight
 *   - Enter
 *   - MetaLeft
 *   - OSLeft
 *   - MetaRight
 *   - OSRight
 *   - ShiftLeft
 *   - ShiftRight
 *   - Space
 *   - Tab
 *   - Convert
 *   - KanaMode
 *   - Lang1
 *   - Lang2
 *   - Lang3
 *   - Lang4
 *   - Lang5
 *   - NonConvert
 *   - Delete
 *   - End
 *   - Help
 *   - Home
 *   - Insert
 *   - PageDown
 *   - PageUp
 *   - ArrowDown
 *   - ArrowLeft
 *   - ArrowRight
 *   - ArrowUp
 *   - NumLock
 *   - Numpad0
 *   - Numpad1
 *   - Numpad2
 *   - Numpad3
 *   - Numpad4
 *   - Numpad5
 *   - Numpad6
 *   - Numpad7
 *   - Numpad8
 *   - Numpad9
 *   - NumpadAdd
 *   - NumpadBackspace
 *   - NumpadClear
 *   - NumpadClearEntry
 *   - NumpadComma
 *   - NumpadDecimal
 *   - NumpadDivide
 *   - NumpadEnter
 *   - NumpadEqual
 *   - NumpadHash
 *   - NumpadMemoryAdd
 *   - NumpadMemoryClear
 *   - NumpadMemoryRecall
 *   - NumpadMemoryStore
 *   - NumpadMemorySubtract
 *   - NumpadMultiply
 *   - NumpadParenLeft
 *   - NumpadParenRight
 *   - NumpadStar
 *   - NumpadSubtract
 *   - Escape
 *   - F1
 *   - F2
 *   - F3
 *   - F4
 *   - F5
 *   - F6
 *   - F7
 *   - F8
 *   - F9
 *   - F10
 *   - F11
 *   - F12
 *   - Fn
 *   - FnLock
 *   - PrintScreen
 *   - ScrollLock
 *   - Pause
 *   - BrowserBack
 *   - BrowserFavorites
 *   - BrowserForward
 *   - BrowserHome
 *   - BrowserRefresh
 *   - BrowserSearch
 *   - BrowserStop
 *   - Eject
 *   - LaunchApp1
 *   - LaunchApp2
 *   - LaunchMail
 *   - MediaPlayPause
 *   - MediaSelect
 *   - LaunchMediaPlayer
 *   - MediaStop
 *   - MediaTrackNext
 *   - MediaTrackPrevious
 *   - Power
 *   - Sleep
 *   - AudioVolumeDown
 *   - VolumeDown
 *   - AudioVolumeMute
 *   - VolumeMute
 *   - AudioVolumeUp
 *   - VolumeUp
 *   - WakeUp
 *   - Hyper
 *   - Super
 *   - Turbo
 *   - Abort
 *   - Resume
 *   - Suspend
 *   - Again
 *   - Copy
 *   - Cut
 *   - Find
 *   - Open
 *   - Paste
 *   - Props
 *   - Select
 *   - Undo
 *   - Hiragana
 *   - Katakana
 *   - Unidentified
 *   - F13
 *   - F14
 *   - F15
 *   - F16
 *   - F17
 *   - F18
 *   - F19
 *   - F20
 *   - F21
 *   - F22
 *   - F23
 *   - F24
 *   - BrightnessDown
 *   - BrightnessUp
 *   - DisplayToggleIntExt
 *   - KeyboardLayoutSelect
 *   - LaunchAssistant
 *   - LaunchControlPanel
 *   - LaunchScreenSaver
 *   - MailForward
 *   - MailReply
 *   - MailSend
 *   - MediaFastForward
 *   - MediaPause
 *   - MediaPlay
 *   - MediaRecord
 *   - MediaRewind
 *   - MicrophoneMuteToggle
 *   - PrivacyScreenToggle
 *   - SelectTask
 *   - ShowAllWindows
 *   - ZoomToggle
 */
export type Shortcut = string

/**
 * Register a global shortcut.
 * @example
 * ```typescript
 * import { register } from '@tauri-apps/api/globalShortcut';
 * await register('CommandOrControl+Shift+C', () => {
 *   console.log('Shortcut triggered');
 * });
 * ```
 *
 * @param shortcut Shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 *
 * @since 1.0.0
 */
async function register(
  shortcut: Shortcut,
  handler: ShortcutHandler
): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'register',
      shortcut,
      handler: transformCallback(handler)
    }
  })
}

/**
 * Register a collection of global shortcuts.
 * @example
 * ```typescript
 * import { registerAll } from '@tauri-apps/api/globalShortcut';
 * await registerAll(['CommandOrControl+Shift+C', 'Ctrl+Alt+F12'], (shortcut) => {
 *   console.log(`Shortcut ${shortcut} triggered`);
 * });
 * ```
 *
 * @param shortcuts Array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 *
 * @since 1.0.0
 */
async function registerAll(
  shortcuts: Shortcut[],
  handler: ShortcutHandler
): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'registerAll',
      shortcuts,
      handler: transformCallback(handler)
    }
  })
}

/**
 * Determines whether the given shortcut is registered by this application or not.
 * @example
 * ```typescript
 * import { isRegistered } from '@tauri-apps/api/globalShortcut';
 * const isRegistered = await isRegistered('CommandOrControl+P');
 * ```
 *
 * @param shortcut Array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 *
 * @since 1.0.0
 */
async function isRegistered(shortcut: Shortcut): Promise<boolean> {
  return invokeTauriCommand({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'isRegistered',
      shortcut
    }
  })
}

/**
 * Unregister a global shortcut.
 * @example
 * ```typescript
 * import { unregister } from '@tauri-apps/api/globalShortcut';
 * await unregister('CmdOrControl+Space');
 * ```
 *
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 *
 * @since 1.0.0
 */
async function unregister(shortcut: Shortcut): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'unregister',
      shortcut
    }
  })
}

/**
 * Unregisters all shortcuts registered by the application.
 * @example
 * ```typescript
 * import { unregisterAll } from '@tauri-apps/api/globalShortcut';
 * await unregisterAll();
 * ```
 *
 * @since 1.0.0
 */
async function unregisterAll(): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'unregisterAll'
    }
  })
}

export { register, registerAll, isRegistered, unregister, unregisterAll }
