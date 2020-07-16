use term::{
  color::{GREEN, RED, YELLOW},
  Attr,
};

fn safe_term_attr<T: term::Terminal + ?Sized>(
  output: &mut T,
  attr: term::Attr,
) -> term::Result<()> {
  if output.supports_attr(attr) {
    output.attr(attr)
  } else {
    Ok(())
  }
}

pub struct Logger<'a> {
  context: &'a str,
}

impl<'a> Logger<'a> {
  pub fn new(context: &'a str) -> Self {
    Self { context }
  }

  fn print(&self, color: u32, message: &str) -> crate::Result<()> {
    if let Some(mut output) = term::stdout() {
      safe_term_attr(&mut *output, Attr::Bold)?;
      output.fg(color)?;
      write!(output, "[{}] {}", self.context, message)?;
      output.flush()?;
    } else {
      println!("[{}] {}", self.context, message);
    }
    Ok(())
  }

  pub fn log(&self, message: &str) -> crate::Result<()> {
    self.print(GREEN, message)
  }

  pub fn warn(&self, message: &str) -> crate::Result<()> {
    self.print(YELLOW, message)
  }

  pub fn error(&self, message: &str) -> crate::Result<()> {
    self.print(RED, message)
  }
}
