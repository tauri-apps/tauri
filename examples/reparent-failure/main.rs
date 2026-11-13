// Regression: a failed WebView reparent must preserve source ownership.
//
// Run as: cargo run -p tauri --features unstable --example reparent-failure
// The application exits automatically after checking failure and recovery.

use std::{thread, time::Duration};
use tauri::{LogicalPosition, LogicalSize, Manager, WebviewUrl};

struct TestExit {
  app: tauri::AppHandle,
  passed: bool,
}

impl Drop for TestExit {
  fn drop(&mut self) {
    let code = if self.passed { 0 } else { 1 };
    eprintln!("REGRESSION_EXIT: {code}");
    self.app.exit(code);
  }
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let source = tauri::WebviewWindowBuilder::new(
        app,
        "reparent-source",
        WebviewUrl::App(Default::default()),
      )
      .title("Reparent Regression Source")
      .inner_size(640.0, 480.0)
      .build()?;

      let child = source.as_ref().window_ref().add_child(
        tauri::webview::WebviewBuilder::new("reparent-child", WebviewUrl::App(Default::default())),
        LogicalPosition::new(20.0, 20.0),
        LogicalSize::new(300.0, 200.0),
      )?;

      let handle = app.handle().clone();

      thread::spawn(move || {
        let mut exit = TestExit {
          app: handle.clone(),
          passed: false,
        };

        thread::sleep(Duration::from_secs(2));

        let valid = tauri::window::WindowBuilder::new(&handle, "reparent-valid")
          .title("Valid Destination")
          .inner_size(500.0, 400.0)
          .build()
          .expect("create valid destination");

        let doomed = tauri::window::WindowBuilder::new(&handle, "reparent-doomed")
          .title("Destroyed Destination")
          .inner_size(500.0, 400.0)
          .build()
          .expect("create destroyed destination");

        doomed.close().expect("close destroyed destination");

        let mut removed = false;
        for _ in 0..40 {
          if handle.get_window("reparent-doomed").is_none() {
            removed = true;
            break;
          }
          thread::sleep(Duration::from_millis(100));
        }

        assert!(removed, "destination must be unregistered");
        println!("REGRESSION: DESTINATION_UNREGISTERED");

        assert!(
          child.reparent(&doomed).is_err(),
          "reparent to destroyed window must return an error"
        );
        println!("REGRESSION: EXPECTED_ERROR");

        child
          .set_bounds(tauri::Rect {
            position: LogicalPosition::new(40.0, 30.0).into(),
            size: LogicalSize::new(280.0, 190.0).into(),
          })
          .expect("source WebView remains usable");
        println!("REGRESSION: SOURCE_USABLE");

        child
          .reparent(&valid)
          .expect("valid reparent after failure must succeed");
        println!("REGRESSION: RETRY_SUCCEEDED");

        exit.passed = true;
        println!("REGRESSION: PASS");
      });

      Ok(())
    })
    .run(tauri::generate_context!(
      "../../examples/multiwebview/tauri.conf.json"
    ))
    .expect("reparent regression application failed");
}
