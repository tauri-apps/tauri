// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

@group(0) @binding(0) var native: texture_2d<f32>;
@group(0) @binding(1) var view: texture_2d<f32>;
@group(0) @binding(2) var popup: texture_2d<f32>;
@group(0) @binding(3) var<uniform> popup_rect: vec4<i32>;

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
  return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

fn over(front: vec4<f32>, back: vec4<f32>) -> vec4<f32> {
  return front + back * (1.0 - front.a);
}

@fragment
fn fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let pixel = vec2<i32>(position.xy);
  var color = textureLoad(native, pixel, 0);
  if all(pixel < vec2<i32>(textureDimensions(view))) {
    color = over(textureLoad(view, pixel, 0), color);
  }
  let local = pixel - popup_rect.xy;
  if all(local >= vec2<i32>(0)) && all(local < popup_rect.zw) {
    color = over(textureLoad(popup, local, 0), color);
  }
  return color;
}
