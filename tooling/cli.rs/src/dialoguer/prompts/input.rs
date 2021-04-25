// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fmt::{Debug, Display},
  io, iter,
  str::FromStr,
};

use super::super::{
  theme::{SimpleTheme, TermThemeRenderer, Theme},
  validate::Validator,
};

use crate::console::{Key, Term};

/// Renders an input prompt.
///
/// ## Example usage
///
/// ```rust,no_run
/// use dialoguer::Input;
///
/// # fn test() -> Result<(), Box<dyn std::error::Error>> {
/// let input : String = Input::new()
///     .with_prompt("Tea or coffee?")
///     .with_initial_text("Yes")
///     .default("No".into())
///     .interact_text()?;
/// # Ok(())
/// # }
/// ```
/// It can also be used with turbofish notation:
///
/// ```rust,no_run
/// # fn test() -> Result<(), Box<dyn std::error::Error>> {
/// # use dialoguer::Input;
/// let input = Input::<String>::new()
///     .interact_text()?;
/// # Ok(())
/// # }
/// ```
pub struct Input<'a, T> {
  prompt: String,
  default: Option<T>,
  show_default: bool,
  initial_text: Option<String>,
  theme: &'a dyn Theme,
  permit_empty: bool,
  validator: Option<Box<dyn FnMut(&T) -> Option<String> + 'a>>,
}

impl<'a, T> Default for Input<'a, T>
where
  T: Clone + FromStr + Display,
  T::Err: Display + Debug,
{
  fn default() -> Input<'a, T> {
    Input::new()
  }
}

impl<'a, T> Input<'a, T>
where
  T: Clone + FromStr + Display,
  T::Err: Display + Debug,
{
  /// Creates an input prompt.
  pub fn new() -> Input<'a, T> {
    Input::with_theme(&SimpleTheme)
  }

  /// Creates an input prompt with a specific theme.
  pub fn with_theme(theme: &'a dyn Theme) -> Input<'a, T> {
    Input {
      prompt: "".into(),
      default: None,
      show_default: true,
      initial_text: None,
      theme,
      permit_empty: false,
      validator: None,
    }
  }

  /// Sets the input prompt.
  pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut Input<'a, T> {
    self.prompt = prompt.into();
    self
  }

  /// Sets initial text that user can accept or erase.
  pub fn with_initial_text<S: Into<String>>(&mut self, val: S) -> &mut Input<'a, T> {
    self.initial_text = Some(val.into());
    self
  }

  /// Sets a default.
  ///
  /// Out of the box the prompt does not have a default and will continue
  /// to display until the user inputs something and hits enter. If a default is set the user
  /// can instead accept the default with enter.
  pub fn default(&mut self, value: T) -> &mut Input<'a, T> {
    self.default = Some(value);
    self
  }

  /// Enables or disables an empty input
  ///
  /// By default, if there is no default value set for the input, the user must input a non-empty string.
  pub fn allow_empty(&mut self, val: bool) -> &mut Input<'a, T> {
    self.permit_empty = val;
    self
  }

  /// Disables or enables the default value display.
  ///
  /// The default behaviour is to append [`default`] to the prompt to tell the
  /// user what is the default value.
  ///
  /// This method does not affect existance of default value, only its display in the prompt!
  pub fn show_default(&mut self, val: bool) -> &mut Input<'a, T> {
    self.show_default = val;
    self
  }

  /// Registers a validator.
  ///
  /// # Example
  ///
  /// ```no_run
  /// # use dialoguer::Input;
  /// let mail: String = Input::new()
  ///     .with_prompt("Enter email")
  ///     .validate_with(|input: &String| -> Result<(), &str> {
  ///         if input.contains('@') {
  ///             Ok(())
  ///         } else {
  ///             Err("This is not a mail address")
  ///         }
  ///     })
  ///     .interact()
  ///     .unwrap();
  /// ```
  pub fn validate_with<V>(&mut self, mut validator: V) -> &mut Input<'a, T>
  where
    V: Validator<T> + 'a,
    T: 'a,
  {
    let mut old_validator_func = self.validator.take();

    self.validator = Some(Box::new(move |value: &T| -> Option<String> {
      if let Some(old) = old_validator_func.as_mut() {
        if let Some(err) = old(value) {
          return Some(err);
        }
      }

      match validator.validate(value) {
        Ok(()) => None,
        Err(err) => Some(err.to_string()),
      }
    }));

    self
  }

  /// Enables the user to enter a printable ascii sequence and returns the result.
  ///
  /// Its difference from [`interact`](#method.interact) is that it only allows ascii characters for string,
  /// while [`interact`](#method.interact) allows virtually any character to be used e.g arrow keys.
  ///
  /// The dialog is rendered on stderr.
  pub fn interact_text(&mut self) -> io::Result<T> {
    self.interact_text_on(&Term::stderr())
  }

  /// Like [`interact_text`](#method.interact_text) but allows a specific terminal to be set.
  pub fn interact_text_on(&mut self, term: &Term) -> io::Result<T> {
    let mut render = TermThemeRenderer::new(term, self.theme);

    loop {
      let default_string = self.default.as_ref().map(|x| x.to_string());

      render.input_prompt(
        &self.prompt,
        if self.show_default {
          default_string.as_deref()
        } else {
          None
        },
      )?;
      term.flush()?;

      // Read input by keystroke so that we can suppress ascii control characters
      if !term.features().is_attended() {
        return Ok("".to_owned().parse::<T>().unwrap());
      }

      let mut chars: Vec<char> = Vec::new();
      let mut position = 0;

      if let Some(initial) = self.initial_text.as_ref() {
        term.write_str(initial)?;
        chars = initial.chars().collect();
        position = chars.len();
      }

      loop {
        match term.read_key()? {
          Key::Backspace if position > 0 => {
            position -= 1;
            chars.remove(position);
            term.clear_chars(1)?;

            let tail: String = chars[position..].iter().collect();

            if !tail.is_empty() {
              term.write_str(&tail)?;
              term.move_cursor_left(tail.len())?;
            }

            term.flush()?;
          }
          Key::Char(chr) if !chr.is_ascii_control() => {
            chars.insert(position, chr);
            position += 1;
            let tail: String = iter::once(&chr).chain(chars[position..].iter()).collect();
            term.write_str(&tail)?;
            term.move_cursor_left(tail.len() - 1)?;
            term.flush()?;
          }
          Key::ArrowLeft if position > 0 => {
            term.move_cursor_left(1)?;
            position -= 1;
            term.flush()?;
          }
          Key::ArrowRight if position < chars.len() => {
            term.move_cursor_right(1)?;
            position += 1;
            term.flush()?;
          }
          Key::Enter => break,
          Key::Unknown => {
            return Err(io::Error::new(
              io::ErrorKind::NotConnected,
              "Not a terminal",
            ))
          }
          _ => (),
        }
      }
      let input = chars.iter().collect::<String>();

      term.clear_line()?;
      render.clear()?;

      if chars.is_empty() {
        if let Some(ref default) = self.default {
          render.input_prompt_selection(&self.prompt, &default.to_string())?;
          term.flush()?;
          return Ok(default.clone());
        } else if !self.permit_empty {
          continue;
        }
      }

      match input.parse::<T>() {
        Ok(value) => {
          if let Some(ref mut validator) = self.validator {
            if let Some(err) = validator(&value) {
              render.error(&err)?;
              continue;
            }
          }

          render.input_prompt_selection(&self.prompt, &input)?;
          term.flush()?;

          return Ok(value);
        }
        Err(err) => {
          render.error(&err.to_string())?;
          continue;
        }
      }
    }
  }

  /// Enables user interaction and returns the result.
  ///
  /// Allows any characters as input, including e.g arrow keys.
  /// Some of the keys might have undesired behavior.
  /// For more limited version, see [`interact_text`](#method.interact_text).
  ///
  /// If the user confirms the result is `true`, `false` otherwise.
  /// The dialog is rendered on stderr.
  pub fn interact(&mut self) -> io::Result<T> {
    self.interact_on(&Term::stderr())
  }

  /// Like [`interact`](#method.interact) but allows a specific terminal to be set.
  pub fn interact_on(&mut self, term: &Term) -> io::Result<T> {
    let mut render = TermThemeRenderer::new(term, self.theme);

    loop {
      let default_string = self.default.as_ref().map(|x| x.to_string());

      render.input_prompt(
        &self.prompt,
        if self.show_default {
          default_string.as_deref()
        } else {
          None
        },
      )?;
      term.flush()?;

      let input = if let Some(initial_text) = self.initial_text.as_ref() {
        term.read_line_initial_text(initial_text)?
      } else {
        term.read_line()?
      };

      render.add_line();
      term.clear_line()?;
      render.clear()?;

      if input.is_empty() {
        if let Some(ref default) = self.default {
          render.input_prompt_selection(&self.prompt, &default.to_string())?;
          term.flush()?;
          return Ok(default.clone());
        } else if !self.permit_empty {
          continue;
        }
      }

      match input.parse::<T>() {
        Ok(value) => {
          if let Some(ref mut validator) = self.validator {
            if let Some(err) = validator(&value) {
              render.error(&err)?;
              continue;
            }
          }

          render.input_prompt_selection(&self.prompt, &input)?;
          term.flush()?;

          return Ok(value);
        }
        Err(err) => {
          render.error(&err.to_string())?;
          continue;
        }
      }
    }
  }
}
