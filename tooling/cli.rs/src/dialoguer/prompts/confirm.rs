// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::io;

use super::super::theme::{SimpleTheme, TermThemeRenderer, Theme};

use crate::console::{Key, Term};

/// Renders a confirm prompt.
///
/// ## Example usage
///
/// ```rust,no_run
/// # fn test() -> Result<(), Box<dyn std::error::Error>> {
/// use dialoguer::Confirm;
///
/// if Confirm::new().with_prompt("Do you want to continue?").interact()? {
///     println!("Looks like you want to continue");
/// } else {
///     println!("nevermind then :(");
/// }
/// # Ok(()) } fn main() { test().unwrap(); }
/// ```
pub struct Confirm<'a> {
  prompt: String,
  default: Option<bool>,
  show_default: bool,
  wait_for_newline: bool,
  theme: &'a dyn Theme,
}

impl<'a> Default for Confirm<'a> {
  fn default() -> Confirm<'a> {
    Confirm::new()
  }
}

impl<'a> Confirm<'a> {
  /// Creates a confirm prompt.
  pub fn new() -> Confirm<'static> {
    Confirm::with_theme(&SimpleTheme)
  }

  /// Creates a confirm prompt with a specific theme.
  ///
  /// ## Examples
  /// ```rust,no_run
  /// use dialoguer::{
  ///     Confirm,
  ///     theme::ColorfulTheme
  /// };
  ///
  /// # fn main() -> std::io::Result<()> {
  /// let proceed = Confirm::with_theme(&ColorfulTheme::default())
  ///     .with_prompt("Do you wish to continue?")
  ///     .interact()?;
  /// #    Ok(())
  /// # }
  /// ```
  pub fn with_theme(theme: &'a dyn Theme) -> Confirm<'a> {
    Confirm {
      prompt: "".into(),
      default: None,
      show_default: true,
      wait_for_newline: false,
      theme,
    }
  }

  /// Sets the confirm prompt.
  pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut Confirm<'a> {
    self.prompt = prompt.into();
    self
  }

  #[deprecated(note = "Use with_prompt() instead", since = "0.6.0")]
  #[inline]
  pub fn with_text(&mut self, text: &str) -> &mut Confirm<'a> {
    self.with_prompt(text)
  }

  /// Sets when to react to user input.
  ///
  /// When `false` (default), we check on each user keystroke immediately as
  /// it is typed. Valid inputs can be one of 'y', 'n', or a newline to accept
  /// the default.
  ///
  /// When `true`, the user must type their choice and hit the Enter key before
  /// proceeding. Valid inputs can be "yes", "no", "y", "n", or an empty string
  /// to accept the default.
  pub fn wait_for_newline(&mut self, wait: bool) -> &mut Confirm<'a> {
    self.wait_for_newline = wait;
    self
  }

  /// Sets a default.
  ///
  /// Out of the box the prompt does not have a default and will continue
  /// to display until the user inputs something and hits enter. If a default is set the user
  /// can instead accept the default with enter.
  pub fn default(&mut self, val: bool) -> &mut Confirm<'a> {
    self.default = Some(val);
    self
  }

  /// Disables or enables the default value display.
  ///
  /// The default is to append the default value to the prompt to tell the user.
  pub fn show_default(&mut self, val: bool) -> &mut Confirm<'a> {
    self.show_default = val;
    self
  }

  /// Enables user interaction and returns the result.
  ///
  /// If the user confirms the result is `true`, `false` if declines or default (configured in [default](#method.default)) if pushes enter.
  /// Otherwise function discards input waiting for valid one.
  ///
  /// The dialog is rendered on stderr.
  pub fn interact(&self) -> io::Result<bool> {
    self.interact_on(&Term::stderr())
  }

  /// Enables user interaction and returns the result.
  ///
  /// This method is similar to [interact_on_opt](#method.interact_on_opt) except for the fact that it does not allow selection of the terminal.
  /// The dialog is rendered on stderr.
  /// Result contains `Some(bool)` if user answered "yes" or "no" or `None` if user cancelled with 'Esc' or 'q'.
  pub fn interact_opt(&self) -> io::Result<Option<bool>> {
    self.interact_on_opt(&Term::stderr())
  }

  /// Like [interact](#method.interact) but allows a specific terminal to be set.
  ///
  /// ## Examples
  ///
  /// ```rust,no_run
  /// use dialoguer::Confirm;
  /// use console::Term;
  ///
  /// # fn main() -> std::io::Result<()> {
  /// let proceed = Confirm::new()
  ///     .with_prompt("Do you wish to continue?")
  ///     .interact_on(&Term::stderr())?;
  /// #   Ok(())
  /// # }
  /// ```
  pub fn interact_on(&self, term: &Term) -> io::Result<bool> {
    self
      ._interact_on(term, false)?
      .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Quit not allowed in this case"))
  }

  /// Like [interact_opt](#method.interact_opt) but allows a specific terminal to be set.
  ///
  /// ## Examples
  /// ```rust,no_run
  /// use dialoguer::Confirm;
  /// use console::Term;
  ///
  /// fn main() -> std::io::Result<()> {
  ///     let confirmation = Confirm::new()
  ///         .interact_on_opt(&Term::stdout())?;
  ///
  ///     match confirmation {
  ///         Some(answer) => println!("User answered {}", if answer { "yes" } else { "no " }),
  ///         None => println!("User did not answer")
  ///     }
  ///
  ///     Ok(())
  /// }
  /// ```
  pub fn interact_on_opt(&self, term: &Term) -> io::Result<Option<bool>> {
    self._interact_on(term, true)
  }

  fn _interact_on(&self, term: &Term, allow_quit: bool) -> io::Result<Option<bool>> {
    let mut render = TermThemeRenderer::new(term, self.theme);

    let default_if_show = if self.show_default {
      self.default
    } else {
      None
    };

    render.confirm_prompt(&self.prompt, default_if_show)?;

    term.hide_cursor()?;
    term.flush()?;

    let rv;

    if self.wait_for_newline {
      // Waits for user input and for the user to hit the Enter key
      // before validation.
      let mut value = default_if_show;

      loop {
        let input = term.read_key()?;

        match input {
          Key::Char('y') | Key::Char('Y') => {
            value = Some(true);
          }
          Key::Char('n') | Key::Char('N') => {
            value = Some(false);
          }
          Key::Enter => {
            if !allow_quit {
              value = value.or(self.default);
            }

            if value.is_some() || allow_quit {
              rv = value;
              break;
            }
            continue;
          }
          Key::Escape | Key::Char('q') if allow_quit => {
            value = None;
          }
          _ => {
            continue;
          }
        };

        term.clear_line()?;
        render.confirm_prompt(&self.prompt, value)?;
      }
    } else {
      // Default behavior: matches continuously on every keystroke,
      // and does not wait for user to hit the Enter key.
      loop {
        let input = term.read_key()?;
        let value = match input {
          Key::Char('y') | Key::Char('Y') => Some(true),
          Key::Char('n') | Key::Char('N') => Some(false),
          Key::Enter if self.default.is_some() => Some(self.default.unwrap()),
          Key::Escape | Key::Char('q') if allow_quit => None,
          _ => {
            continue;
          }
        };

        rv = value;
        break;
      }
    }

    term.clear_line()?;
    render.confirm_prompt_selection(&self.prompt, rv)?;
    term.show_cursor()?;
    term.flush()?;

    Ok(rv)
  }
}
