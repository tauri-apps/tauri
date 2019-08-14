extern crate tauri;
extern crate serde_derive;
extern crate serde_json;

use crate::tauri::process::{ProcessExt, Signal, SystemExt};

fn update() -> Result<(), String> {
  let target = tauri::platform::target_triple().map_err(|_| "Could not determine target")?;
  let github_release = tauri::updater::github::get_latest_release("jaemk", "self_update")
    .map_err(|_| "Could not fetch latest release")?;
  match github_release.asset_for(&target) {
    Some(github_release_asset) => {
      let release = tauri::updater::Release {
        version: github_release.tag.trim_start_matches('v').to_string(),
        download_url: github_release_asset.download_url,
        asset_name: github_release_asset.name,
      };

      let status = tauri::updater::Update::configure()
        .unwrap()
        .release(release)
        .bin_path_in_archive("github")
        .bin_name("app")
        .bin_install_path(&tauri::command::command_path("app".to_string()).unwrap())
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()
        .unwrap()
        .update()
        .unwrap();

      println!("found release: {}", status.version());

      /*let tmp_dir = tauri::dir::with_temp_dir(|dir| {
          let file_path = dir.path().join("my-temporary-note.pdf");
          let mut tmp_archive = std::fs::File::create(file_path).unwrap();
          tauri::http::download(&"https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf".to_string(), &mut tmp_archive, true).unwrap();
      });*/

      Ok(())
    }
    None => Err(format!("Could not find release for target {}", target)),
  }
}

fn restart_app(app_command: String) -> Result<(), String> {
  let mut system = tauri::process::System::new();
  let parent_process = tauri::process::get_parent_process(&mut system)
    .map_err(|_| "Could not determine parent process")?;
  if parent_process.name() == "app" {
    parent_process.kill(Signal::Kill);
    std::thread::sleep(std::time::Duration::from_secs(1));
    std::process::Command::new(app_command)
      .spawn()
      .map_err(|_| "Could not start app")?;
  }
  Ok(())
}

fn run_updater() -> Result<(), String> {
  let app_command = tauri::command::relative_command("app".to_string())
    .map_err(|_| "Could not determine app path")?;
  update()?;
  restart_app(app_command)?;
  Ok(())
}

fn main() {
  match run_updater() {
    Ok(_) => {}
    Err(err) => panic!(err),
  };
}
