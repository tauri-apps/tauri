// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{error::Error, sync::Arc};
use tauri_runtime_cef::{OffscreenFrame, OffscreenSnapshot};
use vello::{
  AaConfig, RenderParams, RendererOptions, Scene,
  kurbo::{Affine, Circle, Rect},
  peniko::{Color, Fill},
  wgpu,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct Layer {
  texture: wgpu::Texture,
  serial: u64,
}

impl Layer {
  fn new(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
    Self {
      texture: device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen layer"),
        size: wgpu::Extent3d {
          width,
          height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
          | wgpu::TextureUsages::COPY_DST
          | if format == wgpu::TextureFormat::Rgba8Unorm {
            wgpu::TextureUsages::STORAGE_BINDING
          } else {
            wgpu::TextureUsages::empty()
          },
        view_formats: &[],
      }),
      serial: 0,
    }
  }

  fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame: Option<&OffscreenFrame>) {
    let Some(frame) = frame else {
      if self.serial != 0 {
        *self = Self::new(device, 1, 1, wgpu::TextureFormat::Bgra8Unorm);
      }
      return;
    };
    if frame.serial == self.serial {
      return;
    }
    if self.texture.width() != frame.width || self.texture.height() != frame.height {
      *self = Self::new(
        device,
        frame.width,
        frame.height,
        wgpu::TextureFormat::Bgra8Unorm,
      );
    }
    queue.write_texture(
      self.texture.as_image_copy(),
      &frame.pixels,
      wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(frame.width * 4),
        rows_per_image: None,
      },
      self.texture.size(),
    );
    self.serial = frame.serial;
  }
}

pub struct Renderer {
  surface: wgpu::Surface<'static>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  vello: vello::Renderer,
  native: Layer,
  view: Layer,
  popup: Layer,
  params: wgpu::Buffer,
  pipeline: wgpu::RenderPipeline,
}

impl Renderer {
  pub async fn new(window: Arc<tauri::Window>) -> Result<Self> {
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window)?;
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
      })
      .await?;
    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor::default())
      .await?;
    let mut config = surface
      .get_default_config(&adapter, 1, 1)
      .ok_or("no surface configuration")?;
    // CEF's BGRA bytes already encode sRGB. Avoid encoding those values a second time.
    config.format = surface
      .get_capabilities(&adapter)
      .formats
      .into_iter()
      .find(|format| !format.is_srgb())
      .ok_or("no unorm surface format")?;
    surface.configure(&device, &config);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("CEF over Vello"),
      source: wgpu::ShaderSource::Wgsl(include_str!("composite.wgsl").into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("offscreen compositor"),
      layout: None,
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vertex"),
        compilation_options: Default::default(),
        buffers: &[],
      },
      primitive: Default::default(),
      depth_stencil: None,
      multisample: Default::default(),
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fragment"),
        compilation_options: Default::default(),
        targets: &[Some(wgpu::ColorTargetState {
          format: config.format,
          blend: None,
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      multiview_mask: None,
      cache: None,
    });
    let params = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("popup position"),
      size: 16,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let native = Layer::new(&device, 1, 1, wgpu::TextureFormat::Rgba8Unorm);
    let view = Layer::new(&device, 1, 1, wgpu::TextureFormat::Bgra8Unorm);
    let popup = Layer::new(&device, 1, 1, wgpu::TextureFormat::Bgra8Unorm);
    let vello = vello::Renderer::new(&device, RendererOptions::default())?;
    Ok(Self {
      surface,
      device,
      queue,
      config,
      vello,
      native,
      view,
      popup,
      params,
      pipeline,
    })
  }

  pub fn draw(&mut self, width: u32, height: u32, snapshot: OffscreenSnapshot) -> Result<()> {
    if width == 0 || height == 0 {
      return Ok(());
    }
    if self.config.width != width || self.config.height != height {
      self.config.width = width;
      self.config.height = height;
      self.surface.configure(&self.device, &self.config);
      self.native = Layer::new(&self.device, width, height, wgpu::TextureFormat::Rgba8Unorm);
    }
    let (output, reconfigure) = match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Success(output) => (output, false),
      wgpu::CurrentSurfaceTexture::Suboptimal(output) => (output, true),
      wgpu::CurrentSurfaceTexture::Outdated => {
        self.surface.configure(&self.device, &self.config);
        return Ok(());
      }
      wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return Ok(()),
      wgpu::CurrentSurfaceTexture::Lost => {
        return Err("native GPU surface lost; recreate the renderer".into());
      }
      wgpu::CurrentSurfaceTexture::Validation => {
        return Err("native GPU surface validation failed".into());
      }
    };
    self
      .view
      .upload(&self.device, &self.queue, snapshot.view.as_ref());
    self
      .popup
      .upload(&self.device, &self.queue, snapshot.popup.as_ref());
    let rect = snapshot.popup_rect;
    let dimensions = if snapshot.popup_visible {
      [rect.width as i32, rect.height as i32]
    } else {
      [0, 0]
    };
    let bytes: Vec<u8> = [rect.x, rect.y, dimensions[0], dimensions[1]]
      .into_iter()
      .flat_map(i32::to_ne_bytes)
      .collect();
    self.queue.write_buffer(&self.params, 0, &bytes);

    let mut scene = Scene::new();
    scene.fill(
      Fill::NonZero,
      Affine::IDENTITY,
      Color::from_rgb8(28, 38, 65),
      None,
      &Rect::new(0.0, 0.0, width as f64, height as f64),
    );
    scene.fill(
      Fill::NonZero,
      Affine::IDENTITY,
      Color::from_rgb8(243, 138, 102),
      None,
      &Circle::new(
        (width as f64 * 0.72, height as f64 * 0.48),
        height as f64 * 0.28,
      ),
    );
    let native = self.native.texture.create_view(&Default::default());
    self.vello.render_to_texture(
      &self.device,
      &self.queue,
      &scene,
      &native,
      &RenderParams {
        base_color: Color::TRANSPARENT,
        width,
        height,
        antialiasing_method: AaConfig::Area,
      },
    )?;
    let view = self.view.texture.create_view(&Default::default());
    let popup = self.popup.texture.create_view(&Default::default());
    let bindings = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("offscreen compositor"),
      layout: &self.pipeline.get_bind_group_layout(0),
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&native),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::TextureView(&view),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: wgpu::BindingResource::TextureView(&popup),
        },
        wgpu::BindGroupEntry {
          binding: 3,
          resource: self.params.as_entire_binding(),
        },
      ],
    });
    let target = output.texture.create_view(&Default::default());
    let mut encoder = self.device.create_command_encoder(&Default::default());
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("offscreen present"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &target,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
      });
      pass.set_pipeline(&self.pipeline);
      pass.set_bind_group(0, &bindings, &[]);
      pass.draw(0..3, 0..1);
    }
    self.queue.submit([encoder.finish()]);
    output.present();
    if reconfigure {
      self.surface.configure(&self.device, &self.config);
    }
    Ok(())
  }
}
