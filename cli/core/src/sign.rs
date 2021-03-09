use tauri_bundler::sign::{generate_key, save_keypair};
use std::path::{Path, PathBuf};

pub struct KeyGenerator {
   private_key: Option<String>,
   password: Option<String>,
   output_path: Option<PathBuf>,
   empty_password: bool,
   force: bool
}

impl Default for KeyGenerator {
   fn default() -> Self {
     Self {
      private_key: None,
      empty_password: false,
      password: None,
      output_path: None,
      force: false
     }
   }
}

impl KeyGenerator {
   pub fn new() -> Self {
      Default::default()
    }
 
    pub fn private_key(mut self, private_key: &str) -> Self {
      self.private_key = Some(private_key.to_owned());
      self
    }

    pub fn empty_password(mut self) -> Self {
      self.empty_password = true;
      self
    }

    pub fn force(mut self) -> Self {
      self.force = true;
      self
    }

    pub fn password(mut self, password: &str) -> Self {
      self.password = Some(password.to_owned());
      self
    }

    pub fn output_path(mut self, output_path: &str) -> Self {
      self.output_path = Some(Path::new(output_path).to_path_buf());
      self
    }
    
    pub fn generate_keys(self) -> crate::Result<()> {
      let mut secret_key_password: Option<String> = None;

      // If no password flag provided, we use empty string
      // If we send None to minisign they'll prompt for the password
      if self.empty_password {
        secret_key_password = Some("".to_string());
      }

      let keypair = generate_key(secret_key_password).expect("Failed to generate key");

      if let Some(output_path) = self.output_path {

        let (secret_path, public_path) = save_keypair(
          self.force,
          output_path,
          &keypair.sk,
          &keypair.pk,
        ).expect("Unable to write keypair");

        println!(
        "\nYour keypair was generated successfully\nPrivate: {} (Keep it secret!)\nPublic: {}\n---------------------------",
        secret_path.display(),
        public_path.display()
        )
      } else {
        println!(
          "\nYour secret key was generated successfully - Keep it secret!\n{}\n\n",
          keypair.sk
        );
        println!(
          "Your public key was generated successfully:\n{}\n\nAdd the public key in your tauri.conf.json\n---------------------------\n",
          keypair.pk
        );
      }

      println!("\nEnvironment variabled used to sign:\n`TAURI_PRIVATE_KEY`  Path or String of your private key\n`TAURI_KEY_PASSWORD`  Your private key password (optional)\n\nATTENTION: If you lose your private key OR password, you'll not be able to sign your update package and updates will not works.\n---------------------------\n");
   
      Ok(())
   }


}