use crate::swift::SRString;
use cocoa::base::id;

#[repr(C)]
pub struct Invoke {
  data: (),
}

extern "C" {
  pub fn invoke_plugin(name: &SRString);
}
