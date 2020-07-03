#[cfg(open)]
pub fn open(uri: String) {
  crate::spawn(move || {
    #[cfg(test)]
    assert!(uri.contains("http://"));

    #[cfg(not(test))]
    webbrowser::open(&uri).expect("Failed to open webbrowser with uri");
  });
}
