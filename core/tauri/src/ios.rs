use swift_rs::SRString;

extern "C" {
  pub fn invoke_plugin(name: &SRString);
}
