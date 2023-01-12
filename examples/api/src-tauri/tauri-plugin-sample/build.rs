use std::{
  env::var,
  fs,
  path::{PathBuf, MAIN_SEPARATOR},
};

fn main() {
  let manifest_dir = var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
  if let Ok(out_dir) = var("TAURI_PLUGIN_OUTPUT_PATH") {
    let source = manifest_dir.join("android");
    let pkg_name = var("CARGO_PKG_NAME").unwrap();

    println!("cargo:rerun-if-env-changed=TAURI_PLUGIN_OUTPUT_PATH");
    println!(
      "cargo:rerun-if-changed={}{}{}",
      out_dir, MAIN_SEPARATOR, pkg_name
    );

    let target = PathBuf::from(out_dir).join(&pkg_name);
    let _ = fs::remove_dir_all(&target);

    for entry in walkdir::WalkDir::new(&source) {
      let entry = entry.unwrap();
      let rel_path = entry.path().strip_prefix(&source).unwrap();
      let dest_path = target.join(rel_path);
      if entry.file_type().is_dir() {
        fs::create_dir(dest_path).expect("failed to create directory");
      } else {
        fs::copy(entry.path(), dest_path).expect("failed to copy Android template file");
      }
    }
  }
}
