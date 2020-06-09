use tauri_updater::{http::Update, CheckStatus, DownloadStatus, InstallStatus, ProgressStatus};

fn update() -> Result<(), Box<dyn ::std::error::Error>> {
  fn get_progress(status: ProgressStatus) {
    match status {
      ProgressStatus::Download(percentage) => {
        println!("DOWNLOAD IN PROGRESS: {:?}%", percentage);
      }
      ProgressStatus::CopyFiles => {
        println!("COPY IN PROGRESS");
      }
      ProgressStatus::Extract => {
        println!("Extracting IN PROGRESS");
      }
    }
  }

  // setup updater
  let updater = Update::configure()
    // URL of the update server {{target}} and {{current_version}} and replaced automatically
    .urls(&["http://badurl.www.tld/1", "https://gist.githubusercontent.com/lemarier/106011e4a5610ef5671af15ce2f78862/raw/d4dd4fa30b9112836e0a201fd3a867446e7bac98/test.json"])
    //.url("http://badurl.www.tld/1")
    //.url("https://gist.githubusercontent.com/lemarier/106011e4a5610ef5671af15ce2f78862/raw/d4dd4fa30b9112836e0a201fd3a867446e7bac98/test.json")
    .url("https://gist.githubusercontent.com/lemarier/8e4703d077ebd6470810927ed2205470/raw/329b7ad9f32d439083af40e7f2090ca072d7a1cf/gistfile1.txt?target={{target}}&version={{current_version}}")
    // current app version (can be extracted from cargo.toml easilly)
    //.current_version(env!("CARGO_PKG_VERSION"))
    .current_version("0.0.1")
    // if not provided we use `env::current_exe()`
    .executable_path("/Applications/TestApp.app/Contents/MacOS/guijs")
    // check for update
    // Handler to get download and install progress
    // Usefull if we want to create a loading bar or something like this
    .on_progress(get_progress)
    .check()?;

  match updater.status() {
    CheckStatus::UpdateAvailable(my_release) => {
      // POPUP Asking if they want to install new version
      println!("New version available {:?}", my_release.version);

      // launch download
      match updater.download()? {
        DownloadStatus::Downloaded(extracted_archive) => {
          // POPUP `Ready to install` with Install and relaunch button
          // launch installation
          match updater.install(extracted_archive)? {
            InstallStatus::Installed => println!("Installation sucess! Restart now"),
            // if something went wrong inside the installation
            InstallStatus::Failed => {
              println!("Installation failed, download new version on www.com")
            }
          }
        }
        DownloadStatus::Failed => println!("Installation failed, download new version on www.com"),
      }
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
