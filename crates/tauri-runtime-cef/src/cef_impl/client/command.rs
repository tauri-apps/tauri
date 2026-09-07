// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::ffi::CStr;
use std::os::raw::c_int;
use std::sync::OnceLock;

use cef::*;

use crate::ChromeCommandGroup;
use crate::macros::wrap_with_args;

/// Commands that open a second browser window or drive a tab strip.
///
/// A Chrome style browser keeps its whole accelerator table live even when it is
/// hosted as a child view with no browser UI, so Ctrl+N opens a real Chrome window
/// next to the app's, Ctrl+Shift+N an incognito one and Ctrl+T a tab in a window
/// the app does not own. None of them has a meaning in an app window: there is no
/// tab strip to select, reorder or restore from.
const WINDOW_AND_TAB_COMMANDS: &[&CStr] = &[
  cef::resources::IDC_NEW_WINDOW,
  cef::resources::IDC_NEW_INCOGNITO_WINDOW,
  cef::resources::IDC_NEW_TAB,
  cef::resources::IDC_NEW_TAB_TO_RIGHT,
  cef::resources::IDC_DUPLICATE_TAB,
  cef::resources::IDC_RESTORE_TAB,
  cef::resources::IDC_MOVE_TAB_TO_NEW_WINDOW,
  cef::resources::IDC_MOVE_TAB_NEXT,
  cef::resources::IDC_MOVE_TAB_PREVIOUS,
  cef::resources::IDC_SHOW_AS_TAB,
  cef::resources::IDC_SELECT_NEXT_TAB,
  cef::resources::IDC_SELECT_PREVIOUS_TAB,
  cef::resources::IDC_SELECT_LAST_TAB,
  cef::resources::IDC_SELECT_TAB_0,
  cef::resources::IDC_SELECT_TAB_1,
  cef::resources::IDC_SELECT_TAB_2,
  cef::resources::IDC_SELECT_TAB_3,
  cef::resources::IDC_SELECT_TAB_4,
  cef::resources::IDC_SELECT_TAB_5,
  cef::resources::IDC_SELECT_TAB_6,
  cef::resources::IDC_SELECT_TAB_7,
  cef::resources::IDC_TAB_SEARCH,
];

/// Commands that treat the app's UI as a web document to be exported.
///
/// Printing, saving and viewing the source of a Tauri window hands the user the
/// app's own bundled markup, and opening a file replaces the app's UI with a local
/// document in the same webview. `IDC_OPEN_FILE` and `IDC_SAVE_PAGE` also raise OS
/// file dialogs the application never asked for. An app that wants any of this
/// drives it itself — `WebviewDispatch::print` still prints on request.
const DOCUMENT_COMMANDS: &[&CStr] = &[
  cef::resources::IDC_PRINT,
  cef::resources::IDC_BASIC_PRINT,
  cef::resources::IDC_SAVE_PAGE,
  cef::resources::IDC_VIEW_SOURCE,
  cef::resources::IDC_OPEN_FILE,
  cef::resources::IDC_CREATE_SHORTCUT,
  cef::resources::IDC_INSTALL_PWA,
];

/// Commands that move focus into browser chrome the window does not have.
///
/// An app window has no omnibox, search box or bookmark bar, so Ctrl+L and its
/// neighbours can only take keyboard focus out of the page and leave it nowhere
/// the user can see. `IDC_HOME` and `IDC_OPEN_CURRENT_URL` additionally navigate
/// the app's own webview away from its UI.
const BROWSER_CHROME_COMMANDS: &[&CStr] = &[
  cef::resources::IDC_FOCUS_LOCATION,
  cef::resources::IDC_FOCUS_SEARCH,
  cef::resources::IDC_FOCUS_TOOLBAR,
  cef::resources::IDC_FOCUS_MENU_BAR,
  cef::resources::IDC_FOCUS_BOOKMARKS,
  cef::resources::IDC_OPEN_CURRENT_URL,
  cef::resources::IDC_HOME,
  cef::resources::IDC_SEARCH,
  cef::resources::IDC_SHOW_APP_MENU,
];

/// Commands that walk the webview's session history.
///
/// An app window has no back stack worth exposing. The browser is created at
/// `INITIAL_LOAD_URL`, an internal placeholder, and only then navigated to the
/// app's own URL, so the very first screen already sits on a second history
/// entry. Alt+Left there navigates the
/// app's UI away to a blank placeholder page, with no way back — the app's own
/// routing is what an app's "back" means, and it does not live in Chrome's
/// session history.
///
/// This is the same reason `context_menu.rs` removes Back and Forward from the
/// page context menu; the two lists are meant to agree, because an entry an app
/// must not offer by right click must not fire from a keystroke either.
///
/// An app that does want the session history drives it itself:
/// `WebviewDispatch::go_back` and `go_forward` call the browser directly and
/// never reach the accelerator table.
const HISTORY_COMMANDS: &[&CStr] = &[cef::resources::IDC_BACK, cef::resources::IDC_FORWARD];

/// Commands that open one of Chrome's own profile-wide surfaces.
///
/// History, downloads, bookmarks, settings, the task manager and the rest load
/// Chrome WebUI pages, and they load them *in place of the app's UI* in the very
/// webview the accelerator was pressed in. They also expose the browsing data of
/// every webview sharing the request context, which is not the app's to show.
const BROWSER_SURFACE_COMMANDS: &[&CStr] = &[
  cef::resources::IDC_SHOW_HISTORY,
  cef::resources::IDC_SHOW_DOWNLOADS,
  cef::resources::IDC_SHOW_BOOKMARK_MANAGER,
  cef::resources::IDC_SHOW_BOOKMARK_BAR,
  cef::resources::IDC_BOOKMARK_THIS_TAB,
  cef::resources::IDC_BOOKMARK_ALL_TABS,
  cef::resources::IDC_OPTIONS,
  cef::resources::IDC_CLEAR_BROWSING_DATA,
  cef::resources::IDC_IMPORT_SETTINGS,
  cef::resources::IDC_TASK_MANAGER,
  cef::resources::IDC_TASK_MANAGER_SHORTCUT,
  cef::resources::IDC_SHOW_SIGNIN,
  cef::resources::IDC_ABOUT,
  cef::resources::IDC_FEEDBACK,
  cef::resources::IDC_HELP_PAGE_VIA_KEYBOARD,
];

/// Commands that open DevTools.
///
/// Blocked only when the webview disabled devtools, in which case they are the
/// accelerator-table half of what `keyboard.rs` already blocks by key code — the
/// key codes cover F12 and the inspect chord, these cover the rest of the table
/// and any platform binding the key codes miss.
const DEVTOOLS_COMMANDS: &[&CStr] = &[
  cef::resources::IDC_DEV_TOOLS,
  cef::resources::IDC_DEV_TOOLS_CONSOLE,
  cef::resources::IDC_DEV_TOOLS_DEVICES,
  cef::resources::IDC_DEV_TOOLS_INSPECT,
  cef::resources::IDC_DEV_TOOLS_TOGGLE,
];

/// Commands that change the page zoom.
///
/// Blocked only when the webview set `zoom_hotkeys_enabled` to false, which is
/// exactly what that attribute asks for. `WebviewDispatch::set_zoom` still zooms
/// on the application's own request.
///
/// That attribute **defaults to false**, so honoring it stops Ctrl+Plus,
/// Ctrl+Minus and Ctrl+0 in every CEF app that did not opt in. Two things it does
/// not stop:
///
/// - Ctrl+mouse-wheel zoom, which Chromium applies in the render widget rather
///   than through the command controller, so no `IDC_ZOOM_*` is ever dispatched
///   for it and there is nothing here to swallow.
/// - The zoom polyfill Tauri injects on Linux and macOS (never Windows) when
///   `zoom_hotkeys_enabled` is true: a page script that watches keydown and
///   Ctrl+wheel and calls `set_webview_zoom`. With the flag true this handler
///   passes the accelerators through as well, so on those two platforms a
///   keyboard zoom is applied twice — once by the polyfill's `set_zoom` and once
///   by Chrome's own step. Nothing here can suppress the polyfill; it is injected
///   from `tauri`, above this runtime.
const ZOOM_COMMANDS: &[&CStr] = &[
  cef::resources::IDC_ZOOM_PLUS,
  cef::resources::IDC_ZOOM_MINUS,
  cef::resources::IDC_ZOOM_NORMAL,
];

/// The IDC names of one [`ChromeCommandGroup`].
fn group_commands(group: ChromeCommandGroup) -> &'static [&'static CStr] {
  match group {
    ChromeCommandGroup::WindowAndTab => WINDOW_AND_TAB_COMMANDS,
    ChromeCommandGroup::Document => DOCUMENT_COMMANDS,
    ChromeCommandGroup::BrowserChrome => BROWSER_CHROME_COMMANDS,
    ChromeCommandGroup::BrowserSurface => BROWSER_SURFACE_COMMANDS,
    ChromeCommandGroup::History => HISTORY_COMMANDS,
  }
}

/// This build's numeric ids for the commands the runtime can swallow, one entry per
/// group so the webview's allowlist can be applied per group.
///
/// Chrome command ids are build-specific integers, so they are resolved from their
/// IDC names — which do not change between builds — through
/// `cef_id_for_command_id_name`. Resolving is done once for the whole process
/// rather than on every keystroke.
struct BlockedCommands {
  groups: Vec<(ChromeCommandGroup, Vec<c_int>)>,
  devtools: Vec<c_int>,
  zoom: Vec<c_int>,
}

fn blocked_commands() -> &'static BlockedCommands {
  static COMMANDS: OnceLock<BlockedCommands> = OnceLock::new();
  COMMANDS.get_or_init(|| BlockedCommands {
    groups: ChromeCommandGroup::ALL
      .iter()
      .map(|group| (*group, command_ids(&[group_commands(*group)])))
      .collect(),
    devtools: command_ids(&[DEVTOOLS_COMMANDS]),
    zoom: command_ids(&[ZOOM_COMMANDS]),
  })
}

/// The numeric ids of `groups` in the running CEF build.
///
/// A name this build does not know resolves to -1 and is dropped: an IDC name
/// retired by a later Chromium simply stops being blocked, rather than blocking
/// whatever command -1 happens to reach.
fn command_ids(groups: &[&[&CStr]]) -> Vec<c_int> {
  groups
    .iter()
    .flat_map(|names| names.iter())
    .map(|name| unsafe { cef::sys::cef_id_for_command_id_name(name.as_ptr()) })
    .filter(|id| *id != -1)
    .collect()
}

wrap_with_args! {
  wrap_command_handler => TauriCefCommandHandlerArgs;

  pub struct TauriCefCommandHandler {
    devtools_enabled: bool,
    zoom_hotkeys_enabled: bool,
    allowed_chrome_commands: Vec<ChromeCommandGroup>,
    frame_navigation_state: crate::FrameNavigationState,
  }

  impl CommandHandler {
    fn on_chrome_command(
      &self,
      browser: Option<&mut Browser>,
      command_id: ::std::os::raw::c_int,
      _disposition: WindowOpenDisposition,
    ) -> ::std::os::raw::c_int {
      // Scoped the way the display handler and the frame observer are: CEF routes
      // browsers this webview does not own through this very client. A DevTools
      // window is the standing case — `show_dev_tools` passes a NULL client, but
      // F12 and the context menu's Inspect reach CEF with no pending show params,
      // and `ChromeBrowserDelegate` then reuses the opener's client — and a
      // DevTools window is a real Chrome window whose zoom, print and find
      // accelerators are its own to run. Blocking there would break them for the
      // developer without protecting anything in the app's window — and because
      // `zoom_hotkeys_enabled` defaults to false, Ctrl+Plus, Ctrl+Minus and Ctrl+0
      // were dead in DevTools for essentially every app.
      //
      // The identity is bound by the frame observer, which the root client always
      // installs, so it is recorded on this browser's first frame notification —
      // long before an accelerator can reach a browser the user has yet to see.
      if !self.owns(browser) {
        return 0;
      }

      if self.blocks(command_id) { 1 } else { 0 }
    }

    // The four predicates below are pinned to what CEF would do on its own, with
    // one exception. `wrap_command_handler!` installs every callback of the
    // handler, and the trait's own default body returns 0 — "hide" and "disable" —
    // so *not* overriding them would silently strip Chrome UI rather than leave it
    // alone. An app window has no Chrome UI for these to act on, but a CEF-owned
    // popup is a real Chrome window whose location bar is worth keeping: it is what
    // tells the user which site an SSO or OAuth page actually belongs to.
    fn is_chrome_app_menu_item_visible(
      &self,
      _browser: Option<&mut Browser>,
      _command_id: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
      1
    }

    /// The exception: a command this handler swallows is reported disabled rather
    /// than left enabled and then ignored.
    ///
    /// Only a CEF-owned popup ever consults this — it reaches
    /// `AppMenuModel::IsCommandIdEnabled`, and an app window has no app menu — but
    /// there the blocked commands used to draw as ordinary entries that did
    /// nothing when clicked. Chromium also `DCHECK`s that a command it dispatched
    /// was enabled, which an entry that is enabled and then swallowed contradicts.
    ///
    /// This does not gate the accelerator table or `HandleCommand`, so it cannot
    /// affect what `on_chrome_command` above swallows; and the scoping is the same,
    /// so a DevTools window keeps every entry of its own app menu.
    fn is_chrome_app_menu_item_enabled(
      &self,
      browser: Option<&mut Browser>,
      command_id: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
      // 1 for everything else is load-bearing, not a default: the trait's own body
      // returns 0, so anything this arm does not answer 1 to would be disabled.
      if self.owns(browser) && self.blocks(command_id) { 0 } else { 1 }
    }

    fn is_chrome_page_action_icon_visible(
      &self,
      _icon_type: ChromePageActionIconType,
    ) -> ::std::os::raw::c_int {
      1
    }

    fn is_chrome_toolbar_button_visible(
      &self,
      _button_type: ChromeToolbarButtonType,
    ) -> ::std::os::raw::c_int {
      1
    }
  }
}

impl TauriCefCommandHandler {
  /// Whether `browser` is the native browser this client's webview owns.
  ///
  /// CEF routes browsers this webview does not own through this very client — a
  /// DevTools window opened on it is the standing case — and their commands are
  /// theirs to run.
  ///
  /// Answers `false` when the identity cannot be established: CEF handed out no
  /// browser, the frame observer has not recorded one yet, or its lock is poisoned.
  /// That direction is deliberate — a browser this webview may not own must not have
  /// its commands swallowed — but it does mean the blocking is best-effort rather
  /// than a security boundary. Nothing here is: an accelerator that slips through
  /// runs a command the user could have reached from Chrome's own UI anyway, and the
  /// things that must not be reachable (the renderer sandbox, the command line
  /// lockdown, DevTools when the webview disabled them) are enforced elsewhere and
  /// do not depend on this.
  fn owns(&self, browser: Option<&mut Browser>) -> bool {
    browser
      .map(|browser| {
        self
          .frame_navigation_state
          .has_browser_id(browser.identifier())
      })
      .unwrap_or(false)
  }

  /// Whether this webview swallows `command_id`.
  ///
  /// Anything not named in the tables above runs as it does today. Clipboard,
  /// find in page, text selection, undo and redo and fullscreen are all things an
  /// app window legitimately uses, so none of them is listed. Reload is not listed
  /// either: the page context menu drops it because a right click is not how an
  /// app offers a reload, but Ctrl+R and F5 on the app's own document are harmless
  /// and are what a developer reaches for.
  ///
  /// A group the webview named in `allowed_chrome_commands` is skipped entirely, so
  /// its commands run the way they would in a browser.
  fn blocks(&self, command_id: c_int) -> bool {
    let commands = blocked_commands();
    commands
      .groups
      .iter()
      .filter(|(group, _)| !self.allowed_chrome_commands.contains(group))
      .any(|(_, ids)| ids.contains(&command_id))
      || (!self.devtools_enabled && commands.devtools.contains(&command_id))
      || (!self.zoom_hotkeys_enabled && commands.zoom.contains(&command_id))
  }
}
