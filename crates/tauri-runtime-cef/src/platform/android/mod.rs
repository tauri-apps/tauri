// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Android event loop.
//!
//! winit's loop runs on `android_main`, but CEF requires the Java main thread, which is the
//! browser UI thread. So `android_main` only runs the loop and posts its events to the Java
//! main thread, where the application and `WinitCefApp` live. That keeps the runtime's single
//! main thread, and [`JavaMainLoop`] stands in for winit's `ActiveEventLoop` there.
//!
//! `android_main` must never wait for the Java main thread: Android lifecycle callbacks on
//! the Java main thread can wait for `android_main`. Waiting the other way is fine, see
//! [`JavaMainLoop::create_window`].

use std::{
  cell::RefCell,
  fmt,
  path::PathBuf,
  sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
    mpsc,
  },
  time::Instant,
};

use jni::{
  JNIEnv, JavaVM,
  objects::{GlobalRef, JClass, JObject, JValue},
};
use raw_window_handle::HasDisplayHandle;
use tauri_runtime::{
  ProgressBarState, Result,
  dpi::{PhysicalPosition, PhysicalRect},
};
use tauri_utils::config::Color;
use winit::{
  application::ApplicationHandler,
  cursor::{CustomCursor, CustomCursorSource},
  error::{EventLoopError, NotSupportedError, RequestError},
  event::{StartCause, WindowEvent},
  event_loop::{
    ActiveEventLoop, ControlFlow, DeviceEvents, EventLoop, EventLoopProxy, OwnedDisplayHandle,
  },
  monitor::MonitorHandle,
  platform::android::EventLoopBuilderExtAndroid,
  window::{Theme, Window, WindowAttributes, WindowId},
};

pub use winit::platform::android::activity::AndroidApp;

use crate::{
  external_message_pump::android as pump,
  platform::{EventLoopExt, MonitorExt, monitor_bounds},
  webview::AppWebview,
  window::AppWindow,
};

/// `android.R.id.content`.
const CONTENT_VIEW_ID: i32 = 0x0102_0002;

type WindowRequest = (
  WindowAttributes,
  mpsc::SyncSender<std::result::Result<Box<dyn Window>, RequestError>>,
);

struct Shared {
  app: AndroidApp,
  proxy: EventLoopProxy,
  display: OwnedDisplayHandle,
  window_requests: Mutex<Vec<WindowRequest>>,
  /// The shortest delay CEF asked for since the last delivery.
  scheduled_work: Mutex<Option<i64>>,
  pump_timer: Mutex<Option<Instant>>,
  drain_posted: AtomicBool,
  exiting: AtomicBool,
  content_view: OnceLock<GlobalRef>,
}

static SHARED: OnceLock<Shared> = OnceLock::new();

thread_local! {
  /// The application handler. Only set on the Java main thread.
  static HANDLER: RefCell<Option<Box<dyn ApplicationHandler>>> = const { RefCell::new(None) };
}

fn shared() -> &'static Shared {
  SHARED
    .get()
    .expect("the application must be started with `tauri_runtime_cef::start_android`")
}

/// Runs `entry`, which builds and runs the tauri application, on the Java main thread, and
/// winit's event loop on this one. Call it from `android_main`.
///
/// The activity must call `getWindow().takeSurface(null)` and
/// `getWindow().takeInputQueue(null)`, so the content view can show CEF's views.
pub fn start_android(app: AndroidApp, entry: impl FnOnce() + Send + 'static) {
  let event_loop = EventLoop::builder()
    .with_android_app(app.clone())
    .build()
    .expect("failed to create the event loop");

  let shared = Shared {
    app: app.clone(),
    proxy: event_loop.create_proxy(),
    display: event_loop.owned_display_handle(),
    window_requests: Default::default(),
    scheduled_work: Default::default(),
    pump_timer: Default::default(),
    drain_posted: Default::default(),
    exiting: Default::default(),
    content_view: Default::default(),
  };
  if SHARED.set(shared).is_err() {
    panic!("`start_android` called twice");
  }

  app.run_on_java_main_thread(Box::new(entry));
  let _ = event_loop.run_app(Bridge);
}

fn post(f: impl FnOnce(&mut dyn ApplicationHandler, &JavaMainLoop) + Send + 'static) {
  shared().app.run_on_java_main_thread(Box::new(move || {
    HANDLER.with(|handler| {
      if let Some(handler) = handler.borrow_mut().as_mut() {
        f(handler.as_mut(), &JavaMainLoop(shared()));
      }
    });
  }));
}

/// The handler on `android_main`: forwards to the Java main thread, and never waits for it.
struct Bridge;

impl ApplicationHandler for Bridge {
  fn can_create_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {}

  fn window_event(
    &mut self,
    _event_loop: &dyn ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    post(move |handler, event_loop| handler.window_event(event_loop, window_id, event));
  }

  fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
    let shared = shared();
    for (attributes, reply) in shared.window_requests.lock().unwrap().drain(..) {
      let _ = reply.send(event_loop.create_window(attributes));
    }

    if shared.drain_posted.swap(true, Ordering::AcqRel) {
      return;
    }
    shared.app.run_on_java_main_thread(Box::new(|| {
      let shared = self::shared();
      shared.drain_posted.store(false, Ordering::Release);
      let scheduled = shared.scheduled_work.lock().unwrap().take();
      if let Some(delay_ms) = scheduled {
        pump::deliver_schedule(delay_ms);
      }
      HANDLER.with(|handler| {
        if let Some(handler) = handler.borrow_mut().as_mut() {
          let event_loop = JavaMainLoop(shared);
          handler.proxy_wake_up(&event_loop);
          handler.about_to_wait(&event_loop);
        }
      });
    }));
  }

  fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
    let shared = shared();
    if shared.exiting.load(Ordering::Acquire) {
      event_loop.exit();
      return;
    }

    let mut timer = shared.pump_timer.lock().unwrap();
    let control_flow = match *timer {
      Some(deadline) if deadline <= Instant::now() => {
        *timer = None;
        shared
          .app
          .run_on_java_main_thread(Box::new(pump::deliver_timeout));
        ControlFlow::Wait
      }
      Some(deadline) => ControlFlow::WaitUntil(deadline),
      None => ControlFlow::Wait,
    };
    event_loop.set_control_flow(control_flow);
  }
}

pub(crate) fn schedule_pump_work(delay_ms: i64) {
  let shared = shared();
  {
    let mut scheduled = shared.scheduled_work.lock().unwrap();
    *scheduled = Some(scheduled.map_or(delay_ms, |current| current.min(delay_ms)));
  }
  shared.proxy.wake_up();
}

pub(crate) fn set_pump_timer(deadline: Option<Instant>) {
  let shared = shared();
  *shared.pump_timer.lock().unwrap() = deadline;
  if deadline.is_some() {
    // `android_main` has to pick up the new deadline.
    shared.proxy.wake_up();
  }
}

/// The event loop as seen from the Java main thread.
#[derive(Clone, Copy)]
pub(crate) struct JavaMainLoop(&'static Shared);

impl fmt::Debug for JavaMainLoop {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("JavaMainLoop").finish()
  }
}

impl JavaMainLoop {
  pub(crate) fn get() -> Self {
    Self(shared())
  }

  /// Hands `app` to the loop and returns: the Java main thread must go back to its `Looper`.
  pub(crate) fn run_app<A: ApplicationHandler + 'static>(
    self,
    mut app: A,
  ) -> std::result::Result<(), EventLoopError> {
    app.new_events(&self, StartCause::Init);
    app.can_create_surfaces(&self);
    HANDLER.with(|handler| *handler.borrow_mut() = Some(Box::new(app)));
    Ok(())
  }
}

impl ActiveEventLoop for JavaMainLoop {
  fn create_proxy(&self) -> EventLoopProxy {
    self.0.proxy.clone()
  }

  /// winit's Android window can only come from `android_main`. Waiting for it is safe because
  /// `android_main` never waits for this thread.
  fn create_window(
    &self,
    window_attributes: WindowAttributes,
  ) -> std::result::Result<Box<dyn Window>, RequestError> {
    let (reply, result) = mpsc::sync_channel(1);
    self
      .0
      .window_requests
      .lock()
      .unwrap()
      .push((window_attributes, reply));
    self.0.proxy.wake_up();
    result
      .recv()
      .map_err(|_| RequestError::from(NotSupportedError::new("the event loop has exited")))?
  }

  fn create_custom_cursor(
    &self,
    _custom_cursor: CustomCursorSource,
  ) -> std::result::Result<CustomCursor, RequestError> {
    Err(NotSupportedError::new("create_custom_cursor is not supported").into())
  }

  fn available_monitors(&self) -> Box<dyn Iterator<Item = MonitorHandle>> {
    Box::new(std::iter::empty())
  }

  fn primary_monitor(&self) -> Option<MonitorHandle> {
    None
  }

  fn listen_device_events(&self, _allowed: DeviceEvents) {}

  fn system_theme(&self) -> Option<Theme> {
    None
  }

  fn set_control_flow(&self, _control_flow: ControlFlow) {}

  fn control_flow(&self) -> ControlFlow {
    ControlFlow::Wait
  }

  fn exit(&self) {
    self.0.exiting.store(true, Ordering::Release);
    self.0.proxy.wake_up();
  }

  fn exiting(&self) -> bool {
    self.0.exiting.load(Ordering::Acquire)
  }

  fn owned_display_handle(&self) -> OwnedDisplayHandle {
    self.0.display.clone()
  }

  fn rwh_06_handle(&self) -> &dyn HasDisplayHandle {
    &self.0.display
  }
}

/// Runs `f` with the JNI env of this thread and the activity.
pub(crate) fn with_activity<R>(f: impl FnOnce(&mut JNIEnv<'_>, &JObject<'_>) -> R) -> R {
  let app = &shared().app;
  // SAFETY: the pointers stay valid for the life of the process and of `app`.
  let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.expect("invalid JavaVM");
  let mut env = vm
    .attach_current_thread_permanently()
    .expect("failed to attach to the JavaVM");
  let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
  f(&mut env, &activity)
}

/// CEF's main args. The JNI reference has to live as long as CEF, so it is never released.
pub(crate) fn main_args() -> cef::args::Args {
  with_activity(|env, activity| {
    let context = env
      .call_method(
        activity,
        "getApplicationContext",
        "()Landroid/content/Context;",
        &[],
      )
      .and_then(|context| context.l())
      .and_then(|context| env.new_global_ref(context))
      .expect("failed to get the application context");
    let args = cef::args::Args::from_android(
      shared().app.vm_as_ptr().cast(),
      context.as_obj().as_raw().cast(),
    );
    std::mem::forget(context);
    args
  })
}

/// The app-private directory for CEF's profile.
pub(crate) fn cache_dir() -> PathBuf {
  shared()
    .app
    .internal_data_path()
    .expect("the app has no internal data path")
}

pub(crate) fn find_class<'a>(
  env: &mut JNIEnv<'a>,
  activity: &JObject<'_>,
  name: String,
) -> std::result::Result<JClass<'a>, jni::errors::Error> {
  let class_name = env.new_string(name.replace('/', "."))?;
  let class_loader = env
    .call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
    .l()?;
  let class = env
    .call_method(
      class_loader,
      "loadClass",
      "(Ljava/lang/String;)Ljava/lang/Class;",
      &[JValue::Object(&class_name)],
    )?
    .l()?;
  Ok(class.into())
}

pub(crate) fn activity_name() -> Result<String> {
  with_activity(|env, activity| {
    let class = env
      .call_method(activity, "getClass", "()Ljava/lang/Class;", &[])?
      .l()?;
    let name = env
      .call_method(class, "getName", "()Ljava/lang/String;", &[])?
      .l()?;
    Ok::<String, jni::errors::Error>(env.get_string(&name.into())?.into())
  })
  .map_err(|_| tauri_runtime::Error::FailedToReceiveMessage)
}

impl MonitorExt for MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    monitor_bounds(self)
  }
}

impl EventLoopExt for dyn ActiveEventLoop + '_ {
  fn set_badge_count(&self, _count: Option<i64>, _desktop_filename: Option<String>) {}

  fn set_badge_label(&self, _label: Option<String>) {}

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    Ok(PhysicalPosition::default())
  }
}

impl AppWindow {
  pub(crate) fn cef_host_handle(&self) -> cef::sys::cef_window_handle_t {
    let shared = shared();
    let view = shared.content_view.get_or_init(|| {
      with_activity(|env, activity| {
        env
          .call_method(
            activity,
            "findViewById",
            "(I)Landroid/view/View;",
            &[JValue::Int(CONTENT_VIEW_ID)],
          )
          .and_then(|view| view.l())
          .and_then(|view| env.new_global_ref(view))
          .expect("failed to get the content view")
      })
    });
    view.as_obj().as_raw().cast()
  }

  pub(crate) fn is_enabled(&self) -> bool {
    true
  }

  pub(crate) fn set_enabled(&self, _enabled: bool) {}

  pub(crate) fn set_progress_bar(&self, _state: ProgressBarState) {}

  pub(crate) fn set_background_color(&mut self, _color: Option<Color>) {}
}

/// CEF sizes and shows its view inside the content view, so these have nothing to do.
impl AppWebview {
  pub(crate) fn native_parent_matches(&self, _parent: &AppWindow) -> Option<bool> {
    None
  }

  pub(crate) fn native_visible(&self) -> Option<bool> {
    None
  }

  pub(crate) fn set_background_color(&self, _color: Option<Color>) {}

  pub(crate) fn bounds(&self) -> Option<tauri_runtime::dpi::Rect> {
    None
  }

  pub(crate) fn reparent(&self, _parent: &AppWindow) {}

  pub(crate) fn apply_visible(&self, _visible: bool) {}

  pub(crate) fn apply_physical_bounds(
    &self,
    _scale: f64,
    _x: i32,
    _y: i32,
    _width: i32,
    _height: i32,
  ) {
  }
}
