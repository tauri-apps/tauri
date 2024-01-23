use std::{fmt::Display, str::FromStr};

use crate::Result;

pub fn request_input<T>(
  prompt: &str,
  initial: Option<T>,
  skip: bool,
  allow_empty: bool,
) -> Result<Option<T>>
where
  T: Clone + FromStr + Display + ToString,
  T::Err: Display + std::fmt::Debug,
{
  if skip {
    Ok(initial)
  } else {
    let theme = dialoguer::theme::ColorfulTheme::default();
    let mut builder = dialoguer::Input::with_theme(&theme)
      .with_prompt(prompt)
      .allow_empty(allow_empty);

    if let Some(v) = initial {
      builder = builder.with_initial_text(v.to_string());
    }

    builder.interact_text().map(Some).map_err(Into::into)
  }
}
