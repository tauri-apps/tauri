// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{io, iter::repeat, ops::Rem};

use super::super::theme::{SimpleTheme, TermThemeRenderer, Theme};

use crate::console::{Key, Term};

/// Renders a multi select prompt.
///
/// ## Example usage
/// ```rust,no_run
/// # fn test() -> Result<(), Box<dyn std::error::Error>> {
/// use dialoguer::MultiSelect;
///
/// let items = vec!["Option 1", "Option 2"];
/// let chosen : Vec<usize> = MultiSelect::new()
///     .items(&items)
///     .interact()?;
/// # Ok(())
/// # }
/// ```
pub struct MultiSelect<'a> {
  defaults: Vec<bool>,
  items: Vec<String>,
  prompt: Option<String>,
  clear: bool,
  theme: &'a dyn Theme,
  paged: bool,
}

impl<'a> Default for MultiSelect<'a> {
  fn default() -> MultiSelect<'a> {
    MultiSelect::new()
  }
}

impl<'a> MultiSelect<'a> {
  /// Creates a multi select prompt.
  pub fn new() -> MultiSelect<'static> {
    MultiSelect::with_theme(&SimpleTheme)
  }

  /// Creates a multi select prompt with a specific theme.
  pub fn with_theme(theme: &'a dyn Theme) -> MultiSelect<'a> {
    MultiSelect {
      items: vec![],
      defaults: vec![],
      clear: true,
      prompt: None,
      theme,
      paged: false,
    }
  }

  /// Enables or disables paging
  pub fn paged(&mut self, val: bool) -> &mut MultiSelect<'a> {
    self.paged = val;
    self
  }

  /// Sets the clear behavior of the menu.
  ///
  /// The default is to clear the menu.
  pub fn clear(&mut self, val: bool) -> &mut MultiSelect<'a> {
    self.clear = val;
    self
  }

  /// Sets a defaults for the menu.
  pub fn defaults(&mut self, val: &[bool]) -> &mut MultiSelect<'a> {
    self.defaults = val
      .to_vec()
      .iter()
      .cloned()
      .chain(repeat(false))
      .take(self.items.len())
      .collect();
    self
  }

  /// Add a single item to the selector.
  #[inline]
  pub fn item<T: ToString>(&mut self, item: T) -> &mut MultiSelect<'a> {
    self.item_checked(item, false)
  }

  /// Add a single item to the selector with a default checked state.
  pub fn item_checked<T: ToString>(&mut self, item: T, checked: bool) -> &mut MultiSelect<'a> {
    self.items.push(item.to_string());
    self.defaults.push(checked);
    self
  }

  /// Adds multiple items to the selector.
  pub fn items<T: ToString>(&mut self, items: &[T]) -> &mut MultiSelect<'a> {
    for item in items {
      self.items.push(item.to_string());
      self.defaults.push(false);
    }
    self
  }

  /// Adds multiple items to the selector with checked state
  pub fn items_checked<T: ToString>(&mut self, items: &[(T, bool)]) -> &mut MultiSelect<'a> {
    for &(ref item, checked) in items {
      self.items.push(item.to_string());
      self.defaults.push(checked);
    }
    self
  }

  /// Prefaces the menu with a prompt.
  ///
  /// When a prompt is set the system also prints out a confirmation after
  /// the selection.
  pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut MultiSelect<'a> {
    self.prompt = Some(prompt.into());
    self
  }

  /// Enables user interaction and returns the result.
  ///
  /// The user can select the items with the space bar and on enter
  /// the selected items will be returned.
  pub fn interact(&self) -> io::Result<Vec<usize>> {
    self.interact_on(&Term::stderr())
  }

  /// Like [interact](#method.interact) but allows a specific terminal to be set.
  pub fn interact_on(&self, term: &Term) -> io::Result<Vec<usize>> {
    let mut page = 0;

    if self.items.is_empty() {
      return Err(io::Error::new(
        io::ErrorKind::Other,
        "Empty list of items given to `MultiSelect`",
      ));
    }

    let capacity = if self.paged {
      term.size().0 as usize - 1
    } else {
      self.items.len()
    };

    let pages = (self.items.len() as f64 / capacity as f64).ceil() as usize;

    let mut render = TermThemeRenderer::new(term, self.theme);
    let mut sel = 0;

    if let Some(ref prompt) = self.prompt {
      render.multi_select_prompt(prompt)?;
    }

    let mut size_vec = Vec::new();

    for items in self
      .items
      .iter()
      .flat_map(|i| i.split('\n'))
      .collect::<Vec<_>>()
    {
      let size = &items.len();
      size_vec.push(*size);
    }

    let mut checked: Vec<bool> = self.defaults.clone();

    loop {
      for (idx, item) in self
        .items
        .iter()
        .enumerate()
        .skip(page * capacity)
        .take(capacity)
      {
        render.multi_select_prompt_item(item, checked[idx], sel == idx)?;
      }

      term.hide_cursor()?;
      term.flush()?;

      match term.read_key()? {
        Key::ArrowDown | Key::Char('j') => {
          if sel == !0 {
            sel = 0;
          } else {
            sel = (sel as u64 + 1).rem(self.items.len() as u64) as usize;
          }
        }
        Key::ArrowUp | Key::Char('k') => {
          if sel == !0 {
            sel = self.items.len() - 1;
          } else {
            sel = ((sel as i64 - 1 + self.items.len() as i64) % (self.items.len() as i64)) as usize;
          }
        }
        Key::ArrowLeft | Key::Char('h') => {
          if self.paged {
            if page == 0 {
              page = pages - 1;
            } else {
              page -= 1;
            }

            sel = page * capacity;
          }
        }
        Key::ArrowRight | Key::Char('l') => {
          if self.paged {
            if page == pages - 1 {
              page = 0;
            } else {
              page += 1;
            }

            sel = page * capacity;
          }
        }
        Key::Char(' ') => {
          checked[sel] = !checked[sel];
        }
        Key::Escape => {
          if self.clear {
            render.clear()?;
          }

          if let Some(ref prompt) = self.prompt {
            render.multi_select_prompt_selection(prompt, &[][..])?;
          }

          term.show_cursor()?;
          term.flush()?;

          return Ok(
            self
              .defaults
              .clone()
              .into_iter()
              .enumerate()
              .filter_map(|(idx, checked)| if checked { Some(idx) } else { None })
              .collect(),
          );
        }
        Key::Enter => {
          if self.clear {
            render.clear()?;
          }

          if let Some(ref prompt) = self.prompt {
            let selections: Vec<_> = checked
              .iter()
              .enumerate()
              .filter_map(|(idx, &checked)| {
                if checked {
                  Some(self.items[idx].as_str())
                } else {
                  None
                }
              })
              .collect();

            render.multi_select_prompt_selection(prompt, &selections[..])?;
          }

          term.show_cursor()?;
          term.flush()?;

          return Ok(
            checked
              .into_iter()
              .enumerate()
              .filter_map(|(idx, checked)| if checked { Some(idx) } else { None })
              .collect(),
          );
        }
        _ => {}
      }

      if sel < page * capacity || sel >= (page + 1) * capacity {
        page = sel / capacity;
      }

      render.clear_preserve_prompt(&size_vec)?;
    }
  }
}
