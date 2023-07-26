//! Runtime authority.

pub use tauri_utils::namespace::MemberResolution;

/// The runtime authority verifies if a given IPC call is authorized.
#[derive(Default)]
pub struct RuntimeAuthority {
  members: Vec<MemberResolution>,
}

impl RuntimeAuthority {
  /// Creates the default (empty) runtime authority.
  pub fn new() -> Self {
    Self::default()
  }

  /// Adds the given member resolution to this authority.
  pub fn add_member(&mut self, member: MemberResolution) {
    self.members.push(member);
  }

  /// Determines if the given command is allowed for the member.
  pub fn is_allowed(&self, member: &str, command: &String) -> bool {
    if let Some(member) = self.members.iter().find(|m| m.member == member) {
      member.commands.contains(command)
    } else {
      false
    }
  }
}
