#[derive(Default)]
pub struct Build {
  debug: bool,
  targets: Option<Vec<String>>,
}

impl Build {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn debug(mut self) -> Self {
    self.debug = true;
    self
  }

  pub fn targets(mut self, targets: Vec<String>) -> Self {
    self.targets = Some(targets);
    self
  }

  pub fn run(self) -> crate::Result<()> {
    unimplemented!()
  }
}
