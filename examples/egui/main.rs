// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{egui, epi, AppHandle};

use std::sync::mpsc::{channel, Receiver, Sender};

#[tauri::command]
async fn open_native_window(app: AppHandle) -> String {
  let (egui_app, rx) = Layout::new();
  let native_options = epi::NativeOptions {
    resizable: false,
    ..Default::default()
  };

  app
    .create_egui_window(
      "native-window".to_string(),
      Box::new(egui_app),
      native_options,
    )
    .unwrap();

  rx.recv().unwrap_or_else(|_| String::new())
}

struct Layout {
  input: String,
  tx: Sender<String>,
}

impl Layout {
  pub fn new() -> (Self, Receiver<String>) {
    let (tx, rx) = channel();
    (
      Self {
        input: "".into(),
        tx,
      },
      rx,
    )
  }
}

impl epi::App for Layout {
  fn name(&self) -> &str {
    "Glutin Window"
  }

  fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
    let Self { input, tx, .. } = self;

    let size = egui::Vec2 { x: 340., y: 100. };

    frame.set_window_size(size);
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.heading("Tauri example");

      let (valid, textfield) = ui
        .horizontal(|ui| {
          let field = ui.add(egui::TextEdit::singleline(input).hint_text("Input"));
          (input.len() > 0, field)
        })
        .inner;

      let mut button = ui.add_enabled(valid, egui::Button::new("Submit"));
      button.rect.min.x = 100.;
      button.rect.max.x = 100.;
      if (textfield.lost_focus() && ui.input().key_pressed(egui::Key::Enter)) || button.clicked() {
        let _ = tx.send(input.clone());
        input.clear();
        frame.quit();
      }
    });
  }
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![open_native_window])
    .run(tauri::generate_context!(
      "../../examples/egui/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
