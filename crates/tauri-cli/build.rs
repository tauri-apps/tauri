fn main() {
  // ensure CLI rebuilds if templates change (existing behavior)
  println!("cargo:rerun-if-changed=templates/");
}
