#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[cfg(desktop)]
mod desktop;

fn main() {
  #[cfg(desktop)]
  desktop::main();
}
