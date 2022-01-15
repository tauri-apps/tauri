// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  EventLoopIterationContext, MenuEventListeners, Message, WebContextStore, WindowEventListeners,
  WindowEventListenersMap, WindowHandle, WindowMenuEventListeners, WindowMessage, WindowWrapper,
};
use wry::application::{
  event::Event,
  event_loop::{ControlFlow, EventLoopProxy, EventLoopWindowTarget},
  window::WindowId,
};

#[cfg(target_os = "linux")]
use glutin::platform::ContextTraitExt;
#[cfg(target_os = "linux")]
use gtk::prelude::*;
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicU8, Ordering};

use std::{
  cell::RefCell,
  collections::HashMap,
  ops::Deref,
  rc::Rc,
  sync::{Arc, Mutex},
};

static EGUI_ID: once_cell::sync::Lazy<Mutex<Option<WindowId>>> =
  once_cell::sync::Lazy::new(|| Mutex::new(None));

#[allow(dead_code)]
pub enum MaybeRc<T> {
  Actual(T),
  Rc(Rc<T>),
}

impl<T> MaybeRc<T> {
  #[allow(dead_code)]
  pub fn new(t: T) -> Self {
    Self::Actual(t)
  }
}

impl<T> AsRef<T> for MaybeRc<T> {
  fn as_ref(&self) -> &T {
    match self {
      Self::Actual(t) => t,
      Self::Rc(t) => t,
    }
  }
}

impl<T> Deref for MaybeRc<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    match self {
      Self::Actual(v) => v,
      Self::Rc(r) => r.deref(),
    }
  }
}

impl<T> std::borrow::Borrow<T> for MaybeRc<T> {
  fn borrow(&self) -> &T {
    match self {
      Self::Actual(v) => &v,
      Self::Rc(r) => r.borrow(),
    }
  }
}

#[allow(dead_code)]
pub enum MaybeRcCell<T> {
  Actual(RefCell<T>),
  RcCell(Rc<RefCell<T>>),
}

impl<T> MaybeRcCell<T> {
  #[allow(dead_code)]
  pub fn new(t: T) -> Self {
    Self::Actual(RefCell::new(t))
  }
}

impl<T> Deref for MaybeRcCell<T> {
  type Target = RefCell<T>;

  fn deref(&self) -> &Self::Target {
    match self {
      Self::Actual(v) => v,
      Self::RcCell(r) => r.deref(),
    }
  }
}

pub struct GlutinWindowContext {
  pub context: MaybeRc<glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>>,
  glow_context: MaybeRc<glow::Context>,
  painter: MaybeRcCell<egui_glow::Painter>,
  integration: MaybeRcCell<egui_tao::epi::EpiIntegration>,
  #[cfg(target_os = "linux")]
  render_flow: Rc<AtomicU8>,
}

impl GlutinWindowContext {
  pub fn window(&self) -> &glutin::window::Window {
    self.context.window()
  }
}

pub fn create_gl_window(
  event_loop: &EventLoopWindowTarget<Message>,
  windows: &Arc<Mutex<HashMap<WindowId, WindowWrapper>>>,
  window_event_listeners: &WindowEventListeners,
  menu_event_listeners: &MenuEventListeners,
  label: String,
  app: Box<dyn epi::App + Send>,
  native_options: epi::NativeOptions,
  proxy: EventLoopProxy<Message>,
) {
  let mut egui_id = EGUI_ID.lock().unwrap();
  if let Some(id) = *egui_id {
    if let WindowHandle::GLWindow(glutin_window_context) =
      &mut windows.lock().unwrap().get_mut(&id).unwrap().inner
    {
      let mut integration = glutin_window_context.integration.borrow_mut();
      let mut painter = glutin_window_context.painter.borrow_mut();
      integration.on_exit(glutin_window_context.context.window());
      painter.destroy(&glutin_window_context.glow_context);
    }
    *egui_id = None;
    let _ = proxy.send_event(Message::Window(id, WindowMessage::Close));
  }

  let persistence = egui_tao::epi::Persistence::from_app_name(app.name());
  let window_settings = persistence.load_window_settings();
  let window_builder =
    egui_tao::epi::window_builder(&native_options, &window_settings).with_title(app.name());
  let gl_window = unsafe {
    glutin::ContextBuilder::new()
      .with_depth_buffer(0)
      .with_srgb(true)
      .with_stencil_buffer(0)
      .with_vsync(true)
      .build_windowed(window_builder, event_loop)
      .unwrap()
      .make_current()
      .unwrap()
  };
  let window_id = gl_window.window().id();
  *egui_id = Some(window_id);

  let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };

  unsafe {
    use glow::HasContext as _;
    gl.enable(glow::FRAMEBUFFER_SRGB);
  }

  struct GlowRepaintSignal(EventLoopProxy<Message>, WindowId);

  impl epi::backend::RepaintSignal for GlowRepaintSignal {
    fn request_repaint(&self) {
      let _ = self
        .0
        .send_event(Message::Window(self.1, WindowMessage::RequestRedraw));
    }
  }

  let repaint_signal = std::sync::Arc::new(GlowRepaintSignal(proxy, window_id));

  let painter = egui_glow::Painter::new(&gl, None, "")
    .map_err(|error| eprintln!("some OpenGL error occurred {}\n", error))
    .unwrap();

  let integration = egui_tao::epi::EpiIntegration::new(
    "egui_glow",
    gl_window.window(),
    repaint_signal,
    persistence,
    app,
  );

  window_event_listeners
    .lock()
    .unwrap()
    .insert(window_id, WindowEventListenersMap::default());

  menu_event_listeners
    .lock()
    .unwrap()
    .insert(window_id, WindowMenuEventListeners::default());

  #[cfg(not(target_os = "linux"))]
  {
    windows.lock().expect("poisoned webview collection").insert(
      window_id,
      WindowWrapper {
        label,
        inner: WindowHandle::GLWindow(GlutinWindowContext {
          context: MaybeRc::new(gl_window),
          glow_context: MaybeRc::new(gl),
          painter: MaybeRcCell::new(painter),
          integration: MaybeRcCell::new(integration),
        }),
        menu_items: Default::default(),
      },
    );
  }
  #[cfg(target_os = "linux")]
  {
    let area = unsafe { gl_window.raw_handle() };
    let integration = Rc::new(RefCell::new(integration));
    let painter = Rc::new(RefCell::new(painter));
    let render_flow = Rc::new(AtomicU8::new(1));
    let gl_window = Rc::new(gl_window);
    let gl = Rc::new(gl);

    let i = integration.clone();
    let p = painter.clone();
    let r = render_flow.clone();
    let gl_window_ = Rc::downgrade(&gl_window);
    let gl_ = gl.clone();
    area.connect_render(move |_, _| {
      if let Some(gl_window) = gl_window_.upgrade() {
        let mut integration = i.borrow_mut();
        let mut painter = p.borrow_mut();
        let (needs_repaint, mut tex_allocation_data, shapes) =
          integration.update(gl_window.window());
        let clipped_meshes = integration.egui_ctx.tessellate(shapes);

        for (id, image) in tex_allocation_data.creations {
          painter.set_texture(&gl_, id, &image);
        }

        {
          let color = integration.app.clear_color();
          unsafe {
            use glow::HasContext as _;
            gl_.disable(glow::SCISSOR_TEST);
            gl_.clear_color(color[0], color[1], color[2], color[3]);
            gl_.clear(glow::COLOR_BUFFER_BIT);
          }
          painter.upload_egui_texture(&gl_, &integration.egui_ctx.font_image());
          painter.paint_meshes(
            &gl_,
            gl_window.window().inner_size().into(),
            integration.egui_ctx.pixels_per_point(),
            clipped_meshes,
          );
        }

        for id in tex_allocation_data.destructions.drain(..) {
          painter.free_texture(id);
        }

        {
          let control_flow = if integration.should_quit() {
            1
          } else if needs_repaint {
            0
          } else {
            1
          };
          r.store(control_flow, Ordering::Relaxed);
        }

        integration.maybe_autosave(gl_window.window());
      }
      gtk::Inhibit(false)
    });

    windows.lock().expect("poisoned webview collection").insert(
      window_id,
      WindowWrapper {
        label,
        inner: WindowHandle::GLWindow(GlutinWindowContext {
          context: MaybeRc::Rc(gl_window),
          glow_context: MaybeRc::Rc(gl),
          painter: MaybeRcCell::RcCell(painter),
          integration: MaybeRcCell::RcCell(integration),
          render_flow,
        }),
        menu_items: Default::default(),
      },
    );
  }
}

#[cfg(not(target_os = "linux"))]
fn win_mac_gl_loop(
  control_flow: &mut ControlFlow,
  glutin_window_context: &mut GlutinWindowContext,
  event: &Event<Message>,
  is_focused: bool,
  should_quit: &mut bool,
) {
  let mut redraw = || {
    let gl_window = &glutin_window_context.context;
    let gl = &glutin_window_context.glow_context;
    let mut integration = glutin_window_context.integration.borrow_mut();
    let mut painter = glutin_window_context.painter.borrow_mut();

    if !is_focused {
      // On Mac, a minimized Window uses up all CPU: https://github.com/emilk/egui/issues/325
      // We can't know if we are minimized: https://github.com/rust-windowing/winit/issues/208
      // But we know if we are focused (in foreground). When minimized, we are not focused.
      // However, a user may want an egui with an animation in the background,
      // so we still need to repaint quite fast.
      std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let (needs_repaint, mut tex_allocation_data, shapes) = integration.update(gl_window.window());
    let clipped_meshes = integration.egui_ctx.tessellate(shapes);

    for (id, image) in tex_allocation_data.creations {
      painter.set_texture(&gl, id, &image);
    }

    {
      let color = integration.app.clear_color();
      unsafe {
        use glow::HasContext as _;
        gl.disable(glow::SCISSOR_TEST);
        gl.clear_color(color[0], color[1], color[2], color[3]);
        gl.clear(glow::COLOR_BUFFER_BIT);
      }
      painter.upload_egui_texture(&gl, &integration.egui_ctx.font_image());
      painter.paint_meshes(
        &gl,
        gl_window.window().inner_size().into(),
        integration.egui_ctx.pixels_per_point(),
        clipped_meshes,
      );

      gl_window.swap_buffers().unwrap();
    }

    for id in tex_allocation_data.destructions.drain(..) {
      painter.free_texture(id);
    }

    {
      *control_flow = if integration.should_quit() {
        *should_quit = true;
        glutin::event_loop::ControlFlow::Wait
      } else if needs_repaint {
        gl_window.window().request_redraw();
        glutin::event_loop::ControlFlow::Poll
      } else {
        glutin::event_loop::ControlFlow::Wait
      };
    }

    integration.maybe_autosave(gl_window.window());
  };

  match event {
    glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
    glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),
    _ => (),
  }
}

#[cfg(target_os = "linux")]
fn linux_gl_loop(
  control_flow: &mut ControlFlow,
  glutin_window_context: &mut GlutinWindowContext,
  event: &Event<Message>,
) {
  let area = unsafe { glutin_window_context.context.raw_handle() };
  if let glutin::event::Event::MainEventsCleared = event {
    area.queue_render();
    match glutin_window_context.render_flow.load(Ordering::Relaxed) {
      0 => *control_flow = glutin::event_loop::ControlFlow::Poll,
      1 => *control_flow = glutin::event_loop::ControlFlow::Wait,
      2 => *control_flow = glutin::event_loop::ControlFlow::Exit,
      _ => unreachable!(),
    }
  }
}

#[cfg(feature = "egui")]
pub fn handle_gl_loop(
  event: &Event<'_, Message>,
  _event_loop: &EventLoopWindowTarget<Message>,
  control_flow: &mut ControlFlow,
  context: EventLoopIterationContext<'_>,
  _web_context: &WebContextStore,
  is_focused: &mut bool,
) -> bool {
  let mut prevent_default = false;
  let EventLoopIterationContext {
    callback,
    windows,
    window_event_listeners,
    menu_event_listeners,
    ..
  } = context;
  let egui_id = EGUI_ID.lock().unwrap();
  if let Some(id) = *egui_id {
    let mut windows_lock = windows.lock().unwrap();
    let mut should_quit = false;
    if let Some(win) = windows_lock.get_mut(&id) {
      if let WindowHandle::GLWindow(glutin_window_context) = &mut win.inner {
        #[cfg(not(target_os = "linux"))]
        win_mac_gl_loop(
          control_flow,
          glutin_window_context,
          &event,
          *is_focused,
          &mut should_quit,
        );
        #[cfg(target_os = "linux")]
        linux_gl_loop(control_flow, glutin_window_context, &event);

        if let glutin::event::Event::WindowEvent {
          event, window_id, ..
        } = event
        {
          if window_id == &id {
            match event {
              glutin::event::WindowEvent::Focused(new_focused) => {
                *is_focused = *new_focused;
              }
              glutin::event::WindowEvent::Resized(physical_size) => {
                glutin_window_context.context.resize(*physical_size);
              }
              _ => (),
            }

            if let glutin::event::WindowEvent::CloseRequested = event {
              drop(windows_lock);
              let w = super::on_close_requested(
                callback,
                *window_id,
                windows.clone(),
                control_flow,
                window_event_listeners,
                menu_event_listeners.clone(),
              );
              if let Some(mut win) = w {
                if let WindowHandle::GLWindow(glutin_window_context) = &mut win.inner {
                  // marker
                  let gl_window = &glutin_window_context.context;
                  let mut integration = glutin_window_context.integration.borrow_mut();
                  integration.on_event(&event);
                  if integration.should_quit() {
                    should_quit = true;
                    *control_flow = glutin::event_loop::ControlFlow::Wait;
                  }
                  gl_window.window().request_redraw();
                }
              }
              prevent_default = true;
            } else {
              // same as the `marker` above
              let gl_window = &glutin_window_context.context;
              let mut integration = glutin_window_context.integration.borrow_mut();
              integration.on_event(&event);
              if integration.should_quit() {
                should_quit = true;
                *control_flow = glutin::event_loop::ControlFlow::Wait;
              }
              gl_window.window().request_redraw();
            }
          }
        }
      }
    }

    if should_quit {
      drop(egui_id);
      super::on_window_close(
        callback,
        id,
        windows.lock().unwrap(),
        control_flow,
        #[cfg(target_os = "linux")]
        window_event_listeners,
        menu_event_listeners.clone(),
      );
    }
  }

  prevent_default
}

pub fn on_window_close(window_id: &WindowId, webview: &mut WindowWrapper) {
  // Destrooy GL context if its a GLWindow
  let mut egui_id = EGUI_ID.lock().unwrap();
  if let Some(id) = *egui_id {
    if &id == window_id {
      #[cfg(not(target_os = "linux"))]
      if let WindowHandle::GLWindow(glutin_window_context) = &webview.inner {
        glutin_window_context
          .integration
          .borrow_mut()
          .on_exit(glutin_window_context.window());
        glutin_window_context
          .painter
          .borrow_mut()
          .destroy(&glutin_window_context.glow_context);
        *egui_id = None;
      }
      #[cfg(target_os = "linux")]
      if let WindowHandle::GLWindow(glutin_window_context) = &mut webview.inner {
        let mut integration = glutin_window_context.integration.borrow_mut();
        integration.on_exit(glutin_window_context.context.window());
        glutin_window_context
          .painter
          .borrow_mut()
          .destroy(&glutin_window_context.glow_context);
        *egui_id = None;
      }
    }
  }
}
