use crate::swift::SRObject;
use cocoa::base::id;

#[repr(C)]
pub struct Invoke {
  data: (),
}

extern "C" {
  pub fn init_invoke() -> SRObject<Invoke>;
}
