#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
  let mut res = winres::WindowsResource::new();
  res.set_icon("icons/icon.ico");
  res.compile().expect("Unable to find visual studio tools");
}

#[cfg(not(windows))]
fn main() {}
