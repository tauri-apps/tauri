use std::{
  env,
  error::Error,
  fs::{read_to_string, File},
  io::{BufWriter, Write},
  path::Path,
};

pub fn main() -> Result<(), Box<dyn Error>> {
  let out_dir = env::var("OUT_DIR")?;

  let dest_config_path = Path::new(&out_dir).join("tauri.conf.json");
  let mut config_file = BufWriter::new(File::create(&dest_config_path)?);

  match env::var_os("TAURI_CONFIG") {
    Some(tauri_config) => {
      let tauri_config_string = tauri_config.into_string().unwrap();
      write!(config_file, "{}", tauri_config_string)?;
    }
    None => match env::var_os("TAURI_DIR") {
      Some(tauri_dir) => {
        let tauri_dir_string = tauri_dir.into_string().unwrap();

        println!("cargo:rerun-if-changed={}", tauri_dir_string);

        let original_config_path = Path::new(&tauri_dir_string).join("tauri.conf.json");
        let original_config = read_to_string(original_config_path)?;

        write!(config_file, "{}", original_config)?;
      }
      None => {
        write!(config_file, "")?;
        println!("Build error: Couldn't find ENV: TAURI_CONFIG or TAURI_DIR");
      }
    },
  }
  Ok(())
}
