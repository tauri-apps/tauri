use crate::Error;

use sysinfo;

pub use sysinfo::{Process, ProcessExt, Signal, System, SystemExt};

pub fn get_parent_process(system: &mut sysinfo::System) -> Result<&Process, Error> {
  let pid = sysinfo::get_current_pid().unwrap();
  system.refresh_process(pid);
  let current_process = system
    .get_process(pid)
    .ok_or(Error::ParentProcess)?;
  let parent_pid = current_process.parent().ok_or(Error::ParentPID)?;
  let parent_process = system
    .get_process(parent_pid)
    .ok_or(Error::ParentProcess)?;

  println!("{}", pid);
  Ok(parent_process)
}
