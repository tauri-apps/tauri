// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{borrow::Cow, sync::Arc};
use winit::{
  application::ApplicationHandler,
  event::WindowEvent,
  event_loop::{ActiveEventLoop, EventLoop},
  window::{Window, WindowId},
};
use wry::{
  dpi::{LogicalPosition, LogicalSize},
  Rect, WebViewBuilder,
};

#[derive(Default)]
struct State {
  window: Option<Arc<Window>>,
  webview: Option<wry::WebView>,
  gfx_state: Option<GfxState>,
}

struct GfxState {
  surface: wgpu::Surface<'static>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  render_pipeline: wgpu::RenderPipeline,
}

impl GfxState {
  fn new(window: Arc<Window>) -> Self {
    let instance = wgpu::Instance::default();
    let window_size = window.inner_size();
    let surface = instance.create_surface(window).unwrap();

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::default(),
      force_fallback_adapter: false,
      compatible_surface: Some(&surface),
    }))
    .expect("Failed to find an appropriate adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(
      &wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits:
          wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
        memory_hints: wgpu::MemoryHints::Performance,
      },
      None,
    ))
    .expect("Failed to create device");

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                r#"
                @vertex
                fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
                    let x = f32(i32(in_vertex_index) - 1);
                    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
                    return vec4<f32>(x, y, 0.0, 1.0);
                }

                @fragment
                fn fs_main() -> @location(0) vec4<f32> {
                    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                }
                "#,
            )),
        });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[],
      push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        buffers: &[],
        compilation_options: Default::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        targets: &[Some(swapchain_format.into())],
        compilation_options: Default::default(),
      }),
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
      cache: None,
    });

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: swapchain_format,
      width: window_size.width,
      height: window_size.height,
      present_mode: wgpu::PresentMode::Fifo,
      desired_maximum_frame_latency: 2,
      alpha_mode: swapchain_capabilities.alpha_modes[0],
      view_formats: vec![],
    };

    surface.configure(&device, &config);

    Self {
      surface,
      device,
      queue,
      config,
      render_pipeline,
    }
  }

  fn render(&mut self) {
    let frame = self
      .surface
      .get_current_texture()
      .expect("Failed to acquire next swap chain texture");
    let view = frame
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      });
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.draw(0..3, 0..1);
    }

    self.queue.submit(Some(encoder.finish()));
    frame.present();
  }
}

impl ApplicationHandler for State {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let mut attributes = Window::default_attributes();
    attributes.transparent = true;
    #[cfg(windows)]
    {
      use winit::platform::windows::WindowAttributesExtWindows;
      attributes = attributes.with_clip_children(false);
    }

    let window = Arc::new(event_loop.create_window(attributes).unwrap());

    let gfx_state = GfxState::new(Arc::clone(&window));

    let webview = WebViewBuilder::new()
      .with_bounds(Rect {
        position: LogicalPosition::new(100, 100).into(),
        size: LogicalSize::new(200, 200).into(),
      })
      .with_transparent(true)
      .with_html(
        r#"<html>
                    <body style="background-color:rgba(87,87,87,0.5);"></body>
                    <script>
                        window.onload = function() {
                            document.body.innerText = `hello, ${navigator.userAgent}`;
                        };
                    </script>
                </html>"#,
      )
      .build_as_child(&window)
      .unwrap();

    window.request_redraw();

    self.window = Some(window);
    self.webview = Some(webview);
    self.gfx_state = Some(gfx_state);
  }

  fn window_event(
    &mut self,
    _event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    match event {
      WindowEvent::Resized(size) => {
        if let Some(gfx_state) = &mut self.gfx_state {
          gfx_state.config.width = size.width;
          gfx_state.config.height = size.height;
          gfx_state
            .surface
            .configure(&gfx_state.device, &gfx_state.config);

          if let Some(window) = &self.window {
            window.request_redraw();
          }
        }
      }
      WindowEvent::RedrawRequested => {
        if let Some(gfx_state) = &mut self.gfx_state {
          gfx_state.render();
        }
      }
      WindowEvent::CloseRequested => {
        std::process::exit(0);
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd",
    ))]
    {
      let context = gtk::glib::MainContext::default();
      while context.pending() {
        context.iteration(false);
      }
    }
  }
}

fn main() {
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
  ))]
  {
    use gtk::prelude::DisplayExtManual;

    gtk::init().unwrap();
    if gtk::gdk::Display::default().unwrap().backend().is_wayland() {
      panic!("This example doesn't support wayland!");
    }

    winit::platform::x11::register_xlib_error_hook(Box::new(|_display, error| {
      let error = error as *mut x11_dl::xlib::XErrorEvent;
      (unsafe { (*error).error_code }) == 170
    }));
  }

  let event_loop = EventLoop::new().unwrap();
  let mut state = State::default();
  event_loop.run_app(&mut state).unwrap();
}
