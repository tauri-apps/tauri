// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use crate::window::WindowId;
use crossbeam_channel::Sender;
pub use jni::{
  self,
  errors::Result as JniResult,
  objects::{GlobalRef, JClass, JMap, JObject, JString},
  sys::jobject,
  JNIEnv,
};
use log::Level;
pub use ndk;
use ndk::{
  input_queue::InputQueue,
  looper::{FdEvent, ForeignLooper, ThreadLooper},
};
use once_cell::sync::{Lazy, OnceCell};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::{
  collections::{BTreeMap, HashSet},
  ffi::{c_void, CStr, CString},
  fs::File,
  io::{BufRead, BufReader},
  os::unix::prelude::*,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex, RwLock, RwLockReadGuard,
  },
  thread,
  time::Duration,
};

/// Android pacakge name that could be used to reference classes
/// in the android project.
pub static PACKAGE: OnceCell<&str> = OnceCell::new();

/// Character set for encoding text content in data URLs.
/// Encodes all control characters and special characters that might cause issues in URLs.
const DATA_URL_ENCODING_SET: &AsciiSet = &CONTROLS
  .add(b' ')
  .add(b'"')
  .add(b'#')
  .add(b'%')
  .add(b'&')
  .add(b'<')
  .add(b'>')
  .add(b'?')
  .add(b'[')
  .add(b'\\')
  .add(b']')
  .add(b'^')
  .add(b'`')
  .add(b'{')
  .add(b'|')
  .add(b'}');

/// Generate JNI compilant functions that are necessary for
/// building android apps with tao.
///
/// Arguments in order:
/// 1. android app domain name in reverse snake_case as an ident (for ex: com_example)
/// 2. android package anme (for ex: wryapp)
/// 3. the android activity that has external linking for the following functions and calls them:
///       - `private external fun onActivityCreate(activity: WryActivity)``
///       - `private external fun start()`
///       - `private external fun resume()`
///       - `private external fun pause()`
///       - `private external fun stop()`
///       - `private external fun onActivitySaveInstanceState()`
///       - `private external fun onActivityDestroy(activity: WryActivity)`
///       - `private external fun onActivityLowMemory()`
///       - `private external fun onWindowFocusChanged(activity: WryActivity, focus: Boolean)`
/// 4. a on_activity_create function that will be ran once after the `onActivityCreate` function above.
/// 5. the main entry point of your android application.
#[rustfmt::skip]
#[macro_export]
macro_rules! android_binding {
  ($domain:ident, $package:ident, $activity:ident, $on_activity_create:path, $main:ident) => {
    ::tao::android_binding!($domain, $package, $activity, $setup, $main, ::tao)
  };
  ($domain:ident, $package:ident, $activity:ident, $on_activity_create:path, $main:ident, $tao:path) => {{
    // NOTE: be careful when changing how this use statement is written
    use $tao::{platform::android::prelude::android_fn, platform::android::prelude::*};
    fn _____tao_store_package_name__() {
      PACKAGE.get_or_init(move || generate_package_name!($domain, $package));
    }

    android_fn!(
      $domain,
      $package,
      $activity,
      create,
      [JObject],
      __VOID__,
      [$main],
    );
    android_fn!(
      $domain,
      $package,
      $activity,
      onActivityCreate,
      [JObject],
      __VOID__,
      [$on_activity_create],
      _____tao_store_package_name__,
    );
    android_fn!($domain, $package, $activity, start, [JObject]);
    android_fn!($domain, $package, $activity, stop, [JObject]);
    android_fn!($domain, $package, $activity, resume, [JObject]);
    android_fn!($domain, $package, $activity, pause, [JObject]);
    android_fn!($domain, $package, $activity, onActivitySaveInstanceState, [JObject]);
    android_fn!($domain, $package, $activity, onActivityDestroy, [JObject]);
    android_fn!($domain, $package, $activity, onActivityLowMemory, [JObject]);
    android_fn!($domain, $package, $activity, onWindowFocusChanged, [JObject,i32]);
    android_fn!($domain, $package, $activity, onNewIntent, [JObject]);
  }};
}

/// `ndk-glue` macros register the reading end of an event pipe with the
/// main [`ThreadLooper`] under this `ident`.
/// When returned from [`ThreadLooper::poll_*`](ThreadLooper::poll_once)
/// an event can be retrieved from [`poll_events()`].
pub const NDK_GLUE_LOOPER_EVENT_PIPE_IDENT: i32 = 0;

/// The [`InputQueue`] received from Android is registered with the main
/// [`ThreadLooper`] under this `ident`.
/// When returned from [`ThreadLooper::poll_*`](ThreadLooper::poll_once)
/// an event can be retrieved from [`input_queue()`].
pub const NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT: i32 = 1;

pub fn android_log(level: Level, tag: &CStr, msg: &CStr) {
  let prio = match level {
    Level::Error => ndk_sys::android_LogPriority::ANDROID_LOG_ERROR,
    Level::Warn => ndk_sys::android_LogPriority::ANDROID_LOG_WARN,
    Level::Info => ndk_sys::android_LogPriority::ANDROID_LOG_INFO,
    Level::Debug => ndk_sys::android_LogPriority::ANDROID_LOG_DEBUG,
    Level::Trace => ndk_sys::android_LogPriority::ANDROID_LOG_VERBOSE,
  };
  unsafe {
    ndk_sys::__android_log_write(prio.0 as _, tag.as_ptr(), msg.as_ptr());
  }
}

fn find_class<'a>(
  env: &mut JNIEnv<'a>,
  activity: &JObject<'_>,
  name: String,
) -> Result<JClass<'a>, super::JniCallError> {
  let class_name = env.new_string(name.replace('/', "."))?;
  let my_class = jni_call_method!(
    env,
    activity,
    "getAppClass",
    "(Ljava/lang/String;)Ljava/lang/Class;",
    &[(&class_name).into()],
    l
  )?;
  Ok(my_class.into())
}

#[derive(Clone, Debug)]
pub struct AndroidContext {
  pub java_vm: *mut c_void,
  pub context_jobject: *mut c_void,
  pub activity_name: String,
  pub window_created: bool,
}

impl AndroidContext {
  pub fn create_activity(&self, activity_name: &str) -> Result<ActivityId, super::JniCallError> {
    let vm = unsafe { jni::JavaVM::from_raw(self.java_vm.cast()) }?;
    let mut env = vm.attach_current_thread_as_daemon()?;
    let main_activity = unsafe { JObject::from_raw(self.context_jobject.cast()) };

    let activity_class = find_class(
      &mut env,
      &main_activity,
      format!("{}/{activity_name}", PACKAGE.get().unwrap()),
    )?;

    let activity_id = jni_call_method!(
      env,
      &main_activity,
      "startActivity",
      "(Ljava/lang/Class;)I",
      &[(&activity_class).into()],
      i
    )?;

    let (tx, rx) = crossbeam_channel::bounded(1);
    ACTIVITY_CREATED_SENDERS
      .lock()
      .unwrap()
      .insert(activity_id, tx);
    rx.recv_timeout(Duration::from_secs(5)).map_err(|e| {
      log::error!("failed to create activity {activity_name}: {e}");
      super::JniCallError::new(
        jni::errors::Error::JniCall(jni::errors::JniError::Unknown),
        Some(e.to_string()),
      )
    })?;

    Ok(activity_id)
  }
}

unsafe impl Send for AndroidContext {}
unsafe impl Sync for AndroidContext {}

pub type ActivityId = i32;

pub(crate) static CONTEXTS: Lazy<Mutex<BTreeMap<ActivityId, AndroidContext>>> =
  Lazy::new(Default::default);
static WINDOW_MANAGER: Lazy<Mutex<BTreeMap<ActivityId, GlobalRef>>> = Lazy::new(Default::default);
pub(crate) static ACTIVITY_CREATED_SENDERS: Lazy<Mutex<BTreeMap<ActivityId, Sender<()>>>> =
  Lazy::new(Default::default);
static INTENT_URLS: Lazy<Mutex<Vec<url::Url>>> = Lazy::new(Default::default);
static INPUT_QUEUE: Lazy<RwLock<Option<InputQueue>>> = Lazy::new(Default::default);
static CONTENT_RECT: Lazy<RwLock<Rect>> = Lazy::new(Default::default);
static LOOPER: Lazy<Mutex<Option<ForeignLooper>>> = Lazy::new(Default::default);
static DID_RESUME: AtomicBool = AtomicBool::new(false);

pub fn main_window_manager() -> Option<GlobalRef> {
  WINDOW_MANAGER.lock().unwrap().values().next().cloned()
}

pub fn activity_window_manager(activity_id: ActivityId) -> Option<GlobalRef> {
  WINDOW_MANAGER.lock().unwrap().get(&activity_id).cloned()
}

pub fn window_manager(activity_id: ActivityId) -> Option<GlobalRef> {
  WINDOW_MANAGER.lock().unwrap().get(&activity_id).cloned()
}

pub fn input_queue() -> RwLockReadGuard<'static, Option<InputQueue>> {
  INPUT_QUEUE.read().unwrap()
}

pub fn content_rect() -> Rect {
  CONTENT_RECT.read().unwrap().clone()
}

pub fn main_android_context() -> Option<AndroidContext> {
  CONTEXTS.lock().unwrap().values().next().cloned()
}

pub fn next_available_activity() -> Option<(ActivityId, AndroidContext)> {
  CONTEXTS
    .lock()
    .unwrap()
    .iter()
    .filter(|(_, ctx)| !ctx.window_created)
    .next()
    .map(|(id, ctx)| (*id, ctx.clone()))
}

pub static PIPE: Lazy<[OwnedFd; 2]> = Lazy::new(|| {
  let mut pipe: [RawFd; 2] = Default::default();
  unsafe { libc::pipe(pipe.as_mut_ptr()) };
  pipe.map(|fd| unsafe { OwnedFd::from_raw_fd(fd) })
});

pub fn poll_events() -> Option<Event> {
  unsafe {
    let size = std::mem::size_of::<Event>();
    let mut event = Event::Start;
    if libc::read(PIPE[0].as_raw_fd(), &mut event as *mut _ as *mut _, size)
      == size as libc::ssize_t
    {
      Some(event)
    } else {
      None
    }
  }
}

pub fn take_intent_urls() -> Vec<url::Url> {
  INTENT_URLS.lock().unwrap().drain(..).collect()
}

unsafe fn wake(event: Event) {
  log::trace!("{:?}", event);
  let size = std::mem::size_of::<Event>();
  let res = libc::write(PIPE[1].as_raw_fd(), &event as *const _ as *const _, size);
  assert_eq!(res, size as libc::ssize_t);
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Rect {
  pub left: u32,
  pub top: u32,
  pub right: u32,
  pub bottom: u32,
}

// event must be copyable to be used in the event loop
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[repr(u8)]
pub enum Event {
  Start,
  Resume,
  Pause,
  Stop,
  LowMemory,
  WindowEvent { id: WindowId, event: WindowEvent },
  ContentRectChanged,
  Opened,
}

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[repr(u8)]
pub enum WindowEvent {
  Focused(bool),
  Created,
  Resized,
  RedrawNeeded,
  Destroyed,
}

pub unsafe fn create(_env: JNIEnv, _: JClass, _: JObject, main: fn()) {
  let logpipe = {
    let mut logpipe: [RawFd; 2] = Default::default();
    libc::pipe(logpipe.as_mut_ptr());
    libc::dup2(logpipe[1], libc::STDOUT_FILENO);
    libc::dup2(logpipe[1], libc::STDERR_FILENO);

    logpipe.map(|fd| unsafe { OwnedFd::from_raw_fd(fd) })
  };
  thread::spawn(move || {
    let tag = CStr::from_bytes_with_nul(b"RustStdoutStderr\0").unwrap();
    let file = File::from_raw_fd(logpipe[0].as_raw_fd());
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    loop {
      buffer.clear();
      if let Ok(len) = reader.read_line(&mut buffer) {
        if len == 0 {
          break;
        } else if let Ok(msg) = CString::new(buffer.clone()) {
          android_log(Level::Info, tag, &msg);
        }
      }
    }
  });

  let looper_ready = Arc::new(Condvar::new());
  let signal_looper_ready = looper_ready.clone();

  thread::spawn(move || {
    let looper = ThreadLooper::prepare();
    let foreign = looper.into_foreign();
    foreign
      .add_fd(
        PIPE[0].as_fd(),
        NDK_GLUE_LOOPER_EVENT_PIPE_IDENT,
        FdEvent::INPUT,
        std::ptr::null_mut(),
      )
      .unwrap();

    {
      let mut locked_looper = LOOPER.lock().unwrap();
      *locked_looper = Some(foreign);
      signal_looper_ready.notify_one();
    }

    main();
  });

  // Don't return from this function (`ANativeActivity_onCreate`) until the thread
  // has created its `ThreadLooper` and assigned it to the static `LOOPER` variable.
  let locked_looper = LOOPER.lock().unwrap();
  let _mutex_guard = looper_ready
    .wait_while(locked_looper, |looper| looper.is_none())
    .unwrap();
}

#[allow(non_snake_case)]
pub unsafe fn onActivityCreate(
  mut env: JNIEnv,
  _jclass: JClass,
  activity: JObject,
  setup: unsafe fn(&str, JNIEnv, &ThreadLooper, GlobalRef),
) {
  let intent =
    jni_call_method!(env, &activity, "getIntent", "()Landroid/content/Intent;", l).unwrap();

  let activity_id = jni_call_method!(env, &activity, "getId", "()I", i).unwrap();

  let activity_name: JString = jni_call_method!(
    env,
    &activity,
    "getLocalClassName",
    "()Ljava/lang/String;",
    l
  )
  .unwrap()
  .into();
  let activity_name = env
    .get_string(&activity_name)
    .unwrap()
    .to_string_lossy()
    .to_string();

  // Initialize global context
  let window_manager = jni_call_method!(
    env,
    &activity,
    "getWindowManager",
    "()Landroid/view/WindowManager;",
    l
  )
  .unwrap();
  let window_manager = env.new_global_ref(window_manager).unwrap();
  WINDOW_MANAGER
    .lock()
    .unwrap()
    .insert(activity_id, window_manager);
  let activity = env.new_global_ref(activity).unwrap();
  let vm = env.get_java_vm().unwrap();
  let thread_env = vm.attach_current_thread_as_daemon().unwrap();

  CONTEXTS.lock().unwrap().insert(
    activity_id,
    AndroidContext {
      java_vm: vm.get_java_vm_pointer() as *mut _,
      context_jobject: activity.as_obj().as_raw() as *mut _,
      activity_name,
      window_created: false,
    },
  );
  let looper = ThreadLooper::for_thread().unwrap();
  setup(PACKAGE.get().unwrap(), thread_env, &looper, activity);

  if let Some(tx) = ACTIVITY_CREATED_SENDERS
    .lock()
    .unwrap()
    .remove(&activity_id)
  {
    let _ = tx.send(());
  }

  handle_intent(env, intent);
}

pub unsafe fn resume(_: JNIEnv, _: JClass, _: JObject) {
  let did_resume = DID_RESUME.swap(true, Ordering::Relaxed);
  // first Activity onResume() is called even after onCreate()
  // to match the iOS implementation, we ignore the first resume event
  if did_resume {
    wake(Event::Resume);
  }
}

pub unsafe fn pause(_: JNIEnv, _: JClass, _: JObject) {
  wake(Event::Pause);
}

#[allow(non_snake_case)]
pub unsafe fn onWindowFocusChanged(
  mut env: JNIEnv,
  _: JClass,
  activity: JObject,
  has_focus: libc::c_int,
) {
  let activity_id = jni_call_method!(env, &activity, "getId", "()I", i).unwrap();
  let event = Event::WindowEvent {
    id: WindowId(super::WindowId(activity_id)),
    event: WindowEvent::Focused(has_focus != 0),
  };
  wake(event);
}

#[allow(non_snake_case)]
pub unsafe fn onNewIntent(env: JNIEnv, _: JClass, intent: JObject) {
  handle_intent(env, intent);
}

pub unsafe fn handle_intent(mut env: JNIEnv, intent: JObject) {
  let action = jni_call_method!(env, &intent, "getAction", "()Ljava/lang/String;", l)
    .unwrap()
    .into();
  let action = env
    .get_string(&action)
    .map(|action| action.to_string_lossy().to_string());

  let Ok(action) = action else {
    return;
  };

  // Only handle SEND, SEND_MULTIPLE, and VIEW actions
  if action != "android.intent.action.SEND"
    && action != "android.intent.action.VIEW"
    && action != "android.intent.action.SEND_MULTIPLE"
  {
    return;
  }

  let mut urls = HashSet::new();

  // Get intent type (may be null)
  let intent_type = jni_call_method!(env, &intent, "getType", "()Ljava/lang/String;", l)
    .ok()
    .filter(|jstr| !jstr.is_null())
    .map(|jstr| jstr.into())
    .and_then(|intent_type: JString| {
      env
        .get_string(&intent_type)
        .ok()
        .map(|s| s.to_string_lossy().to_string())
    });

  // Handle text/plain intents (EXTRA_TEXT)
  if intent_type.as_deref() == Some("text/plain") {
    let extra_text_key = env.new_string("android.intent.extra.TEXT").unwrap();
    let extra_text = jni_call_method!(
      env,
      &intent,
      "getStringExtra",
      "(Ljava/lang/String;)Ljava/lang/String;",
      &[(&extra_text_key).into()],
      l
    )
    .ok()
    .and_then(|jstr| {
      let jstr: JString = jstr.into();
      env
        .get_string(&jstr)
        .ok()
        .map(|s| s.to_string_lossy().to_string())
    });

    if let Some(text) = extra_text {
      if !text.is_empty() {
        // Check if it's a valid URL
        if let Ok(url) = url::Url::parse(&text) {
          urls.insert(url);
        } else {
          // If not a URL, create a data URL for plain text
          // Use percent encoding for the text content
          let encoded = utf8_percent_encode(&text, DATA_URL_ENCODING_SET).to_string();
          if let Ok(url) = url::Url::parse(&format!("data:text/plain,{}", encoded)) {
            urls.insert(url);
          }
        }
      }
    }
  }

  // Handle ClipData (API >= KITKAT, which is API 19)
  // We'll try to get clip data, and if it fails, we'll continue
  let clip_data = jni_call_method!(
    env,
    &intent,
    "getClipData",
    "()Landroid/content/ClipData;",
    l
  )
  .ok()
  .filter(|clip_data| !clip_data.is_null());

  if let Some(clip_data) = clip_data {
    // getItemCount may return null if the intent has only a single uri in which case the uri can be received by getData|String instead.
    if let Ok(item_count) = jni_call_method!(env, &clip_data, "getItemCount", "()I", i) {
      for i in 0..item_count {
        let clip_item = jni_call_method!(
          env,
          &clip_data,
          "getItemAt",
          "(I)Landroid/content/ClipData$Item;",
          &[i.into()],
          l
        )
        .unwrap();

        let uri = jni_call_method!(env, &clip_item, "getUri", "()Landroid/net/Uri;", l)
          .ok()
          .filter(|uri| !uri.is_null());

        if let Some(uri) = uri {
          let uri_string: JString =
            jni_call_method!(env, &uri, "toString", "()Ljava/lang/String;", l)
              .unwrap()
              .into();
          let uri_str = env
            .get_string(&uri_string)
            .unwrap()
            .to_string_lossy()
            .to_string();
          if let Ok(url) = url::Url::parse(&uri_str) {
            urls.insert(url);
          } else {
            log::error!("failed to parse URI: {}", uri_str);
          }
        }
      }
    }
  }

  // Handle EXTRA_STREAM (for file sharing)
  let extras = jni_call_method!(env, &intent, "getExtras", "()Landroid/os/Bundle;", l)
    .ok()
    .filter(|extras| !extras.is_null());

  if let Some(extras) = extras {
    let extra_stream_key = env.new_string("android.intent.extra.STREAM").unwrap();
    let extra_stream = jni_call_method!(
      env,
      &extras,
      "get",
      "(Ljava/lang/String;)Ljava/lang/Object;",
      &[(&extra_stream_key).into()],
      l
    )
    .ok()
    .filter(|extra_stream| !extra_stream.is_null());

    if let Some(stream_uri) = extra_stream {
      let uri_string: JString =
        jni_call_method!(env, &stream_uri, "toString", "()Ljava/lang/String;", l)
          .unwrap()
          .into();
      let uri_str = env
        .get_string(&uri_string)
        .unwrap()
        .to_string_lossy()
        .to_string();
      if let Ok(url) = url::Url::parse(&uri_str) {
        urls.insert(url);
      } else {
        log::error!("failed to parse URI: {}", uri_str);
      }
    }
  }

  // Handle getDataString() for VIEW intents (deeplinks)
  if action == "android.intent.action.VIEW" {
    let data_string = jni_call_method!(env, &intent, "getDataString", "()Ljava/lang/String;", l)
      .ok()
      .filter(|data_string| !data_string.is_null())
      .and_then(|jstr| {
        let jstr: JString = jstr.into();
        env
          .get_string(&jstr)
          .ok()
          .map(|s| s.to_string_lossy().to_string())
      });

    if let Some(data_str) = data_string {
      if let Ok(url) = url::Url::parse(&data_str) {
        urls.insert(url);
      } else {
        log::error!("failed to parse data string: {}", data_str);
      }
    } else {
      log::error!("Intent data string is null");
    }
  }

  if !urls.is_empty() {
    INTENT_URLS.lock().unwrap().extend(urls);
    wake(Event::Opened);
  }
}

pub unsafe fn start(_: JNIEnv, _: JClass, _: JObject) {
  wake(Event::Start);
}

pub unsafe fn stop(_: JNIEnv, _: JClass, _: JObject) {
  wake(Event::Stop);
}

#[allow(non_snake_case)]
pub unsafe fn onActivityDestroy(mut env: JNIEnv, _: JClass, activity: JObject) {
  let activity_id = jni_call_method!(env, &activity, "getId", "()I", i).unwrap();

  let is_changing_configurations =
    jni_call_method!(env, &activity, "isChangingConfigurations", "()Z", z).unwrap();

  // keep our Rust window references alive when the activity is going to be recreated due to configuration changes
  // e.g. rotation, multi window mode change etc
  if !is_changing_configurations {
    wake(Event::WindowEvent {
      id: WindowId(super::WindowId(activity_id)),
      event: WindowEvent::Destroyed,
    });
    CONTEXTS.lock().unwrap().remove(&activity_id);
    WINDOW_MANAGER.lock().unwrap().remove(&activity_id);
  }
}

///////////////////////////////////////////////
// Events below are not used by event loop yet.
///////////////////////////////////////////////

#[allow(non_snake_case)]
pub unsafe fn onActivitySaveInstanceState(_: JNIEnv, _: JClass, _: JObject) {}

#[allow(non_snake_case)]
pub unsafe fn onActivityLowMemory(_: JNIEnv, _: JClass, _: JObject) {
  wake(Event::LowMemory);
}

/*
unsafe extern "C" fn on_window_resized(
  activity: *mut ANativeActivity,
  _window: *mut ANativeWindow,
) {
  wake(activity, Event::WindowResized);
}

unsafe extern "C" fn on_input_queue_created(
  activity: *mut ANativeActivity,
  queue: *mut AInputQueue,
) {
  let input_queue = InputQueue::from_ptr(NonNull::new(queue).unwrap());
  let locked_looper = LOOPER.lock().unwrap();
  // The looper should always be `Some` after `fn init()` returns, unless
  // future code cleans it up and sets it back to `None` again.
  let looper = locked_looper.as_ref().expect("Looper does not exist");
  input_queue.attach_looper(looper, NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT);
  *INPUT_QUEUE.write().unwrap() = Some(input_queue);
  wake(activity, Event::InputQueueCreated);
}

unsafe extern "C" fn on_input_queue_destroyed(
  activity: *mut ANativeActivity,
  queue: *mut AInputQueue,
) {
  wake(activity, Event::InputQueueDestroyed);
  let mut input_queue_guard = INPUT_QUEUE.write().unwrap();
  assert_eq!(input_queue_guard.as_ref().unwrap().ptr().as_ptr(), queue);
  let input_queue = InputQueue::from_ptr(NonNull::new(queue).unwrap());
  input_queue.detach_looper();
  *input_queue_guard = None;
}

unsafe extern "C" fn on_content_rect_changed(activity: *mut ANativeActivity, rect: *const ARect) {
  let rect = Rect {
    left: (*rect).left as _,
    top: (*rect).top as _,
    right: (*rect).right as _,
    bottom: (*rect).bottom as _,
  };
  *CONTENT_RECT.write().unwrap() = rect;
  wake(activity, Event::ContentRectChanged);
}
*/
