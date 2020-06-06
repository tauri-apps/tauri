use tauri_updater::{http::Update, CheckStatus};

fn update() -> Result<(), Box<dyn ::std::error::Error>> {
  // setup updater
  let updater = Update::configure()
    // URL of the update server {{target}} and {{current_version}} and replaced automatically
    .url("https://gist.githubusercontent.com/lemarier/8e4703d077ebd6470810927ed2205470/raw/329b7ad9f32d439083af40e7f2090ca072d7a1cf/gistfile1.txt?target={{target}}&version={{current_version}}")
    // current app version (can be extracted from cargo.toml easilly)
    //.current_version(env!("CARGO_PKG_VERSION"))
    .current_version("0.0.1")
    // if not provided we use `env::current_exe()`
    .executable_path("/Applications/TestApp.app/Contents/MacOS/guijs")
    // check for update
    .check()?;

  match updater.status() {
    CheckStatus::UpdateAvailable(my_release) => {
      println!("New version available {:?}", my_release.version);
      println!("New version body {:?}", my_release.body);
      println!("update available {:?}", my_release);
      updater.install()?;
    }
    CheckStatus::UpToDate => println!("App already up to date"),
  }

  Ok(())
}

pub fn main() {
  if let Err(e) = update() {
    println!("[ERROR] {}", e);
    ::std::process::exit(1);
  }
}
