use cocoa::base::id;
use swift_rs::SRString;

extern "C" {
  pub fn invoke_plugin(webview: id, name: &SRString, callback: usize, error: usize);
}
