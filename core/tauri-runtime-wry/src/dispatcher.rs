use tauri_runtime::dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use tauri_runtime::menu::{MenuEvent, MenuUpdate};
use tauri_runtime::monitor::Monitor;
use tauri_runtime::window::{
  CursorIcon, DetachedWindow, PendingWindow, UserAttentionType, WindowEvent,
};
use tauri_runtime::{Dispatch, Error, Icon, UserEvent};
use tauri_utils::Theme;
use wry::webview::Url;

use crate::macros::{getter, window_getter};
use crate::window::{WindowBuilder, WindowMessage};
use crate::wrappers::{MonitorHandleWrapper, WryIcon};
use crate::{send_user_message, Context, Message, Result, WebviewId, WebviewMessage, Wry};

/// The Tauri [`Dispatch`] for [`Wry`].
#[derive(Debug, Clone)]
pub struct WryDispatcher<T: UserEvent> {
  pub(crate) webview_id: WebviewId,
  pub(crate) context: Context<T>,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for WryDispatcher<T> {}

impl<T: UserEvent> Dispatch<T> for WryDispatcher<T> {
  type Runtime = Wry<T>;
  type WindowBuilder = WindowBuilder;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    send_user_message(&self.context, Message::Task(Box::new(f)))
  }

  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> u64 {
    let id = self.context.next_id();
    let _ = self.context.proxy.send_event(Message::Window(
      self.webview_id,
      WindowMessage::AddWindowEventListener(id, Box::new(f)),
    ));
    id
  }

  fn on_menu_event<F: Fn(&MenuEvent) + Send + 'static>(&self, f: F) -> u64 {
    let id = self.context.next_id();
    let _ = self.context.proxy.send_event(Message::Window(
      self.webview_id,
      WindowMessage::AddMenuEventListener(id, Box::new(f)),
    ));
    id
  }

  fn with_webview<F: FnOnce(Box<dyn std::any::Any>) + Send + 'static>(&self, f: F) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.webview_id,
        WindowMessage::WithWebview(Box::new(move |webview| f(Box::new(webview)))),
      ),
    )
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn open_devtools(&self) {
    let _ = send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::OpenDevTools),
    );
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn close_devtools(&self) {
    let _ = send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::CloseDevTools),
    );
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn is_devtools_open(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsDevToolsOpen)
  }

  // Getters

  fn url(&self) -> Result<Url> {
    window_getter!(self, WindowMessage::Url)
  }

  fn scale_factor(&self) -> Result<f64> {
    window_getter!(self, WindowMessage::ScaleFactor)
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    window_getter!(self, WindowMessage::InnerPosition)?
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    window_getter!(self, WindowMessage::OuterPosition)?
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    window_getter!(self, WindowMessage::InnerSize)
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    window_getter!(self, WindowMessage::OuterSize)
  }

  fn is_fullscreen(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsFullscreen)
  }

  fn is_minimized(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMinimized)
  }

  fn is_maximized(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMaximized)
  }

  fn is_decorated(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsDecorated)
  }

  fn is_resizable(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsResizable)
  }

  fn is_visible(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsVisible)
  }

  fn title(&self) -> Result<String> {
    window_getter!(self, WindowMessage::Title)
  }

  fn is_menu_visible(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMenuVisible)
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    Ok(window_getter!(self, WindowMessage::CurrentMonitor)?.map(|m| MonitorHandleWrapper(m).into()))
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    Ok(window_getter!(self, WindowMessage::PrimaryMonitor)?.map(|m| MonitorHandleWrapper(m).into()))
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    Ok(
      window_getter!(self, WindowMessage::AvailableMonitors)?
        .into_iter()
        .map(|m| MonitorHandleWrapper(m).into())
        .collect(),
    )
  }

  fn theme(&self) -> Result<Theme> {
    window_getter!(self, WindowMessage::Theme)
  }

  #[cfg(linuxy)]
  fn gtk_window(&self) -> Result<gtk::ApplicationWindow> {
    window_getter!(self, WindowMessage::GtkWindow).map(|w| w.0)
  }

  fn raw_window_handle(&self) -> Result<raw_window_handle::RawWindowHandle> {
    window_getter!(self, WindowMessage::RawWindowHandle).map(|w| w.0)
  }

  // Setters

  fn center(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::Center),
    )
  }

  fn print(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(self.webview_id, WebviewMessage::Print),
    )
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.webview_id,
        WindowMessage::RequestUserAttention(request_type.map(Into::into)),
      ),
    )
  }

  // Creates a window by dispatching a message to the event loop.
  // Note that this must be called from a separate thread, otherwise the channel will introduce a deadlock.
  fn create_window(
    &mut self,
    pending: PendingWindow<T, Self::Runtime>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    self.context.create_webview(pending)
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetResizable(resizable)),
    )
  }

  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetTitle(title.into())),
    )
  }

  fn maximize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::Maximize),
    )
  }

  fn unmaximize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::Unmaximize),
    )
  }

  fn minimize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::Minimize),
    )
  }

  fn unminimize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::Unminimize),
    )
  }

  fn show_menu(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::ShowMenu),
    )
  }

  fn hide_menu(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::HideMenu),
    )
  }

  fn show(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::Show),
    )
  }

  fn hide(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::Hide),
    )
  }

  fn close(&self) -> Result<()> {
    // NOTE: close cannot use the `send_user_message` function because it accesses the event loop callback
    self
      .context
      .proxy
      .send_event(Message::Window(self.webview_id, WindowMessage::Close))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetDecorations(decorations)),
    )
  }

  fn set_shadow(&self, enable: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetShadow(enable)),
    )
  }

  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.webview_id,
        WindowMessage::SetAlwaysOnTop(always_on_top),
      ),
    )
  }

  fn set_content_protected(&self, protected: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.webview_id,
        WindowMessage::SetContentProtected(protected),
      ),
    )
  }

  fn set_size(&self, size: Size) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetSize(size)),
    )
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetMinSize(size)),
    )
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetMaxSize(size)),
    )
  }

  fn set_position(&self, position: Position) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetPosition(position)),
    )
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetFullscreen(fullscreen)),
    )
  }

  fn set_focus(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetFocus),
    )
  }

  fn set_icon(&self, icon: Icon) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.webview_id,
        WindowMessage::SetIcon(WryIcon::try_from(icon)?.0),
      ),
    )
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetSkipTaskbar(skip)),
    )
  }

  fn set_cursor_grab(&self, grab: bool) -> crate::Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetCursorGrab(grab)),
    )
  }

  fn set_cursor_visible(&self, visible: bool) -> crate::Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetCursorVisible(visible)),
    )
  }

  fn set_cursor_icon(&self, icon: CursorIcon) -> crate::Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::SetCursorIcon(icon)),
    )
  }

  fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> crate::Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.webview_id,
        WindowMessage::SetCursorPosition(position.into()),
      ),
    )
  }

  fn set_ignore_cursor_events(&self, ignore: bool) -> crate::Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.webview_id,
        WindowMessage::SetIgnoreCursorEvents(ignore),
      ),
    )
  }

  fn start_dragging(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::DragWindow),
    )
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(
        self.webview_id,
        WebviewMessage::EvaluateScript(script.into()),
      ),
    )
  }

  fn update_menu_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.webview_id, WindowMessage::UpdateMenuItem(id, update)),
    )
  }
}
