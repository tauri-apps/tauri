// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use dpi::LogicalSize;
use tao::{
  event::{ElementState, Event, KeyEvent, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  keyboard::KeyCode,
  window::WindowBuilder,
};

fn main() {
  let event_loop = EventLoop::new();

  let mut background_color = (100, 100, 100, 255);

  let window = WindowBuilder::new()
    .with_title("Hit space to change background color!")
    .with_inner_size(LogicalSize::new(400.0, 300.0))
    .with_background_color(background_color)
    .build(&event_loop)
    .unwrap();

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent { event, .. } => match event {
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::KeyboardInput {
          event:
            KeyEvent {
              physical_key: KeyCode::Space,
              state: ElementState::Released,
              ..
            },
          ..
        } => {
          background_color.1 = background_color.1.wrapping_add(20);
          println!("Setting background color to: {background_color:?}");
          window.set_background_color(Some(background_color));
        }
        _ => (),
      },
      _ => (),
    }
  });
}
