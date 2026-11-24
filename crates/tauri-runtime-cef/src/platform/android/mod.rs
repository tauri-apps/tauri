// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  platform::{EventLoopExt, MonitorExt},
  webview::AppWebview,
  window::AppWindow,
};
use cef::ImplBrowserHost;
use jni::{
  JNIEnv, JavaVM,
  objects::{GlobalRef, JClass, JObject, JValue},
  sys::jlong,
};
use std::sync::OnceLock;
use tauri_runtime::{
  Error, Result,
  dpi::{PhysicalPosition, PhysicalRect, PhysicalSize, Rect},
};
use winit::{
  event_loop::ActiveEventLoop, monitor::MonitorHandle, platform::android::activity::AndroidApp,
};

struct Context {
  vm: JavaVM,
  app: AndroidApp,
  activity: GlobalRef,
  container: OnceLock<GlobalRef>,
}
static CONTEXT: OnceLock<Context> = OnceLock::new();
static UI_THREAD: OnceLock<std::thread::ThreadId> = OnceLock::new();
type Task = Box<dyn FnOnce() + Send>;

pub(crate) fn initialize(app: &AndroidApp) -> Result<()> {
  // android-activity owns both JNI pointers for the lifetime of this activity.
  let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.unwrap();
  let activity = {
    let env = vm.attach_current_thread().unwrap();
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    env.new_global_ref(activity).unwrap()
  };
  CONTEXT
    .set(Context {
      vm,
      app: app.clone(),
      activity,
      container: OnceLock::new(),
    })
    .map_err(|_| Error::FailedToSendMessage)?;
  Ok(())
}

pub(crate) fn has_window() -> bool {
  CONTEXT.get().unwrap().app.native_window().is_some()
}

fn container() -> &'static GlobalRef {
  CONTEXT.get().unwrap().container.get_or_init(|| {
    on_ui(|| {
      with_env(|env, activity| {
        let view = env
          .call_method(
            activity,
            "getCefContainer",
            "()Landroid/widget/FrameLayout;",
            &[],
          )
          .unwrap()
          .l()
          .unwrap();
        env.new_global_ref(view).unwrap()
      })
    })
  })
}

pub(crate) fn cache_dir() -> std::path::PathBuf {
  // File/context methods are safe on the native thread, including before the
  // GameActivity lifecycle has started its event loop.
  with_env(|env, activity| {
    let directory = env
      .call_method(activity, "getCacheDir", "()Ljava/io/File;", &[])
      .unwrap()
      .l()
      .unwrap();
    let path = env
      .call_method(directory, "getAbsolutePath", "()Ljava/lang/String;", &[])
      .unwrap()
      .l()
      .unwrap();
    let path: String = env
      .get_string(&jni::objects::JString::from(path))
      .unwrap()
      .into();
    std::path::PathBuf::from(path)
  })
}

pub(crate) fn with_env<R>(f: impl FnOnce(&mut JNIEnv, &JObject) -> R) -> R {
  let context = CONTEXT
    .get()
    .expect("CEF Android context is not initialized");
  let mut env = context.vm.attach_current_thread().unwrap();
  f(&mut env, context.activity.as_obj())
}

pub(crate) fn post(f: impl FnOnce() + Send + 'static) {
  let task: Box<Task> = Box::new(Box::new(f));
  let pointer = Box::into_raw(task);
  with_env(|env, activity| {
    if let Err(error) = env.call_method(
      activity,
      "postCefTask",
      "(J)V",
      &[JValue::Long(pointer as jlong)],
    ) {
      // Java did not accept ownership of the closure.
      drop(unsafe { Box::from_raw(pointer) });
      panic!("failed to dispatch to Android UI thread: {error}");
    }
  });
}

pub(crate) fn on_ui<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> R {
  if UI_THREAD.get() == Some(&std::thread::current().id()) {
    return f();
  }
  let (tx, rx) = std::sync::mpsc::sync_channel(1);
  post(move || {
    let _ = tx.send(f());
  });
  rx.recv().expect("Android UI task did not complete")
}

/// Run a CEF operation synchronously on Java's UI thread.
///
/// Safety: captured values must support access on that thread. The caller must
/// not capture winit objects or thread-local state. Unlike a posted task, this
/// blocks until completion, so borrowed CEF argument structures stay alive and
/// cannot be accessed concurrently by the calling thread.
pub(crate) unsafe fn on_ui_scoped<R: Send + 'static, F: FnOnce() -> R>(f: F) -> R {
  let mut work = Some(f);
  let pointer = (&mut work as *mut Option<F>) as usize;
  on_ui(move || {
    let work = unsafe { &mut *(pointer as *mut Option<F>) };
    work.take().unwrap()()
  })
}

/// Run startup work while acknowledging GameActivity's synchronous lifecycle commands.
/// Safety: the same capture restrictions as `on_ui_scoped` apply.
pub(crate) unsafe fn on_ui_with_event_loop<R: Send + 'static, F: FnOnce() -> R>(
  event_loop: &mut winit::event_loop::EventLoop,
  f: F,
) -> R {
  use winit::{application::ApplicationHandler, event_loop::pump_events::EventLoopExtPumpEvents};
  struct Startup;
  impl ApplicationHandler for Startup {
    fn can_create_surfaces(&mut self, _: &dyn ActiveEventLoop) {}
    fn window_event(
      &mut self,
      _: &dyn ActiveEventLoop,
      _: winit::window::WindowId,
      _: winit::event::WindowEvent,
    ) {
    }
  }
  let mut work = Some(f);
  let pointer = (&mut work as *mut Option<F>) as usize;
  let (tx, rx) = std::sync::mpsc::sync_channel(1);
  post(move || {
    let work = unsafe { &mut *(pointer as *mut Option<F>) };
    let _ = tx.send(work.take().unwrap()());
  });
  loop {
    match rx.try_recv() {
      Ok(result) => return result,
      Err(std::sync::mpsc::TryRecvError::Disconnected) => {
        panic!("CEF startup task did not complete")
      }
      Err(std::sync::mpsc::TryRecvError::Empty) => {}
    }
    event_loop.pump_app_events(Some(std::time::Duration::from_millis(10)), &mut Startup);
  }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_tauri_cef_CefActivity_nativeRunTask(
  _env: JNIEnv,
  _class: JClass,
  pointer: jlong,
) {
  UI_THREAD.get_or_init(|| std::thread::current().id());
  // Each pointer is posted exactly once by `post` and consumed on Java's UI thread.
  let task = unsafe { Box::from_raw(pointer as *mut Task) };
  task();
}

pub(crate) fn find_class<'a>(
  env: &mut JNIEnv<'a>,
  activity: &JObject,
  name: String,
) -> jni::errors::Result<JClass<'a>> {
  let loader = env
    .call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
    .l()?;
  let name = JObject::from(env.new_string(name.replace('/', "."))?);
  env
    .call_method(
      loader,
      "loadClass",
      "(Ljava/lang/String;)Ljava/lang/Class;",
      &[JValue::Object(&name)],
    )?
    .l()
    .map(JClass::from)
}

fn with_view<R: Send + 'static>(
  handle: usize,
  f: impl FnOnce(&mut JNIEnv, &JObject) -> R + Send + 'static,
) -> R {
  on_ui(move || {
    with_env(|env, _| {
      let view = unsafe { JObject::from_raw(handle as jni::sys::jobject) };
      f(env, &view)
    })
  })
}

impl MonitorExt for MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    super::monitor_bounds(self)
  }
}
impl EventLoopExt for dyn ActiveEventLoop + '_ {
  fn set_badge_count(&self, _: Option<i64>, _: Option<String>) {}
  fn set_badge_label(&self, _: Option<String>) {}
  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    Err(Error::FailedToGetCursorPosition)
  }
}
impl AppWindow {
  pub(crate) fn cef_host_handle(&self) -> cef::sys::cef_window_handle_t {
    container().as_obj().as_raw().cast()
  }
  pub(crate) fn is_enabled(&self) -> bool {
    with_view(self.cef_host_handle() as usize, |env, view| {
      env
        .call_method(view, "isEnabled", "()Z", &[])
        .unwrap()
        .z()
        .unwrap()
    })
  }
  pub(crate) fn set_enabled(&self, enabled: bool) {
    with_view(self.cef_host_handle() as usize, move |env, view| {
      env
        .call_method(view, "setEnabled", "(Z)V", &[JValue::Bool(enabled as u8)])
        .unwrap();
    });
  }
  pub(crate) fn set_background_color(&mut self, color: Option<tauri_utils::config::Color>) {
    let color = color
      .map(|tauri_utils::config::Color(r, g, b, a)| {
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
      })
      .unwrap_or(0xffffffff);
    with_view(self.cef_host_handle() as usize, move |env, view| {
      env
        .call_method(
          view,
          "setBackgroundColor",
          "(I)V",
          &[JValue::Int(color as i32)],
        )
        .unwrap();
    });
  }
  pub(crate) fn set_progress_bar(&self, _: tauri_runtime::ProgressBarState) {}
}
impl AppWebview {
  fn android_view(&self) -> usize {
    self.host.window_handle() as usize
  }
  pub(crate) fn native_parent_matches(&self, parent: &AppWindow) -> Option<bool> {
    let parent = parent.cef_host_handle() as usize;
    Some(with_view(self.android_view(), move |env, view| {
      let actual = env
        .call_method(view, "getParent", "()Landroid/view/ViewParent;", &[])
        .unwrap()
        .l()
        .unwrap();
      let parent = unsafe { JObject::from_raw(parent as jni::sys::jobject) };
      env.is_same_object(actual, parent).unwrap()
    }))
  }
  pub(crate) fn native_visible(&self) -> Option<bool> {
    Some(with_view(self.android_view(), |env, view| {
      env
        .call_method(view, "isShown", "()Z", &[])
        .unwrap()
        .z()
        .unwrap()
    }))
  }
  pub(crate) fn bounds(&self) -> Option<Rect> {
    Some(with_view(self.android_view(), |env, view| {
      let mut get = |method| {
        env
          .call_method(view, method, "()I", &[])
          .unwrap()
          .i()
          .unwrap()
      };
      Rect {
        position: PhysicalPosition::new(get("getLeft"), get("getTop")).into(),
        size: PhysicalSize::new(get("getWidth") as u32, get("getHeight") as u32).into(),
      }
    }))
  }
  pub(crate) fn reparent(&self, _: &AppWindow) { /* One Android activity owns the main webview. */
  }
  pub(crate) fn apply_visible(&self, visible: bool) {
    with_view(self.android_view(), move |env, view| {
      env
        .call_method(
          view,
          "setVisibility",
          "(I)V",
          &[JValue::Int(if visible { 0 } else { 8 })],
        )
        .unwrap();
    });
  }
  pub(crate) fn set_background_color(&self, color: Option<tauri_utils::config::Color>) {
    let tauri_utils::config::Color(r, g, b, a) =
      color.unwrap_or(tauri_utils::config::Color(255, 255, 255, 255));
    let color = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
    with_view(self.android_view(), move |env, view| {
      env
        .call_method(
          view,
          "setBackgroundColor",
          "(I)V",
          &[JValue::Int(color as i32)],
        )
        .unwrap();
    });
  }
  pub(crate) fn apply_physical_bounds(&self, _: f64, x: i32, y: i32, width: i32, height: i32) {
    with_view(self.android_view(), move |env, view| {
      let params = env
        .new_object(
          "android/widget/FrameLayout$LayoutParams",
          "(II)V",
          &[JValue::Int(width), JValue::Int(height)],
        )
        .unwrap();
      env
        .set_field(&params, "leftMargin", "I", JValue::Int(x))
        .unwrap();
      env
        .set_field(&params, "topMargin", "I", JValue::Int(y))
        .unwrap();
      env
        .call_method(
          view,
          "setLayoutParams",
          "(Landroid/view/ViewGroup$LayoutParams;)V",
          &[JValue::Object(&params)],
        )
        .unwrap();
    });
  }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_tauri_plugin_PluginManager_handlePluginResponse(
  mut env: JNIEnv,
  _: JObject,
  id: i32,
  success: jni::objects::JString,
  error: jni::objects::JString,
) {
  tauri::handle_android_plugin_response(&mut env, id, success, error);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_tauri_plugin_PluginManager_sendChannelData(
  mut env: JNIEnv,
  _: JObject,
  id: i64,
  data: jni::objects::JString,
) {
  tauri::send_channel_data(&mut env, id, data);
}
