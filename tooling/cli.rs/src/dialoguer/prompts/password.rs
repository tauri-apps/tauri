// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::io;

use super::super::theme::{SimpleTheme, TermThemeRenderer, Theme};

use crate::console::Term;
use zeroize::Zeroizing;

/// Renders a password input prompt.
///
/// ## Example usage
///
/// ```rust,no_run
/// # fn test() -> Result<(), Box<std::error::Error>> {
/// use dialoguer::Password;
///
/// let password = Password::new().with_prompt("New Password")
///     .with_confirmation("Confirm password", "Passwords mismatching")
///     .interact()?;
/// println!("Length of the password is: {}", password.len());
/// # Ok(()) } fn main() { test().unwrap(); }
/// ```
pub struct Password<'a> {
  prompt: String,
  theme: &'a dyn Theme,
  allow_empty_password: bool,
  confirmation_prompt: Option<(String, String)>,
}

impl<'a> Default for Password<'a> {
  fn default() -> Password<'a> {
    Password::new()
  }
}

impl<'a> Password<'a> {
  /// Creates a password input prompt.
  pub fn new() -> Password<'static> {
    Password::with_theme(&SimpleTheme)
  }

  /// Creates a password input prompt with a specific theme.
  pub fn with_theme(theme: &'a dyn Theme) -> Password<'a> {
    Password {
      prompt: "".into(),
      theme,
      allow_empty_password: false,
      confirmation_prompt: None,
    }
  }

  /// Sets the password input prompt.
  pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut Password<'a> {
    self.prompt = prompt.into();
    self
  }

  /// Enables confirmation prompting.
  pub fn with_confirmation<A, B>(&mut self, prompt: A, mismatch_err: B) -> &mut Password<'a>
  where
    A: Into<String>,
    B: Into<String>,
  {
    self.confirmation_prompt = Some((prompt.into(), mismatch_err.into()));
    self
  }

  /// Allows/Disables empty password.
  ///
  /// By default this setting is set to false (i.e. password is not empty).
  pub fn allow_empty_password(&mut self, allow_empty_password: bool) -> &mut Password<'a> {
    self.allow_empty_password = allow_empty_password;
    self
  }

  /// Enables user interaction and returns the result.
  ///
  /// If the user confirms the result is `true`, `false` otherwise.
  /// The dialog is rendered on stderr.
  pub fn interact(&self) -> io::Result<String> {
    self.interact_on(&Term::stderr())
  }

  /// Like `interact` but allows a specific terminal to be set.
  pub fn interact_on(&self, term: &Term) -> io::Result<String> {
    let mut render = TermThemeRenderer::new(term, self.theme);
    render.set_prompts_reset_height(false);

    loop {
      let password = Zeroizing::new(self.prompt_password(&mut render, &self.prompt)?);

      if let Some((ref prompt, ref err)) = self.confirmation_prompt {
        let pw2 = Zeroizing::new(self.prompt_password(&mut render, prompt)?);

        if *password == *pw2 {
          render.clear()?;
          render.password_prompt_selection(&self.prompt)?;
          term.flush()?;
          return Ok((*password).clone());
        }

        render.error(err)?;
      } else {
        render.clear()?;
        render.password_prompt_selection(&self.prompt)?;
        term.flush()?;

        return Ok((*password).clone());
      }
    }
  }

  fn prompt_password(&self, render: &mut TermThemeRenderer, prompt: &str) -> io::Result<String> {
    loop {
      render.password_prompt(prompt)?;
      render.term().flush()?;

      let input = render.term().read_secure_line()?;

      render.add_line();

      if !input.is_empty() || self.allow_empty_password {
        return Ok(input);
      }
    }
  }
}
