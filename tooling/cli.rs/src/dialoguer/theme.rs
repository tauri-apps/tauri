//! Customizes the rendering of the elements.
use std::{fmt, io};

use crate::console::{style, Style, StyledObject, Term};

/// Implements a theme for dialoguer.
pub trait Theme {
    /// Formats a prompt.
    #[inline]
    fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        write!(f, "{}:", prompt)
    }

    /// Formats out an error.
    #[inline]
    fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
        write!(f, "error: {}", err)
    }

    /// Formats a confirm prompt.
    fn format_confirm_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<bool>,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(f, "{} ", &prompt)?;
        }
        match default {
            None => write!(f, "[y/n] ")?,
            Some(true) => write!(f, "[Y/n] ")?,
            Some(false) => write!(f, "[y/N] ")?,
        }
        Ok(())
    }

    /// Formats a confirm prompt after selection.
    fn format_confirm_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: Option<bool>,
    ) -> fmt::Result {
        let selection = selection.map(|b| if b { "yes" } else { "no" });

        match selection {
            Some(selection) if prompt.is_empty() => {
                write!(f, "{}", selection)
            }
            Some(selection) => {
                write!(f, "{} {}", &prompt, selection)
            }
            None if prompt.is_empty() => Ok(()),
            None => {
                write!(f, "{}", &prompt)
            }
        }
    }

    /// Formats an input prompt.
    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        match default {
            Some(default) if prompt.is_empty() => write!(f, "[{}]: ", default),
            Some(default) => write!(f, "{} [{}]: ", prompt, default),
            None => write!(f, "{}: ", prompt),
        }
    }

    /// Formats an input prompt after selection.
    #[inline]
    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        write!(f, "{}: {}", prompt, sel)
    }

    /// Formats a password prompt.
    #[inline]
    fn format_password_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.format_input_prompt(f, prompt, None)
    }

    /// Formats a password prompt after selection.
    #[inline]
    fn format_password_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
    ) -> fmt::Result {
        self.format_input_prompt_selection(f, prompt, "[hidden]")
    }

    /// Formats a select prompt.
    #[inline]
    fn format_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.format_prompt(f, prompt)
    }

    /// Formats a select prompt after selection.
    #[inline]
    fn format_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.format_input_prompt_selection(f, prompt, sel)
    }

    /// Formats a multi select prompt.
    #[inline]
    fn format_multi_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.format_prompt(f, prompt)
    }

    /// Formats a sort prompt.
    #[inline]
    fn format_sort_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.format_prompt(f, prompt)
    }

    /// Formats a multi_select prompt after selection.
    fn format_multi_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        write!(f, "{}: ", prompt)?;
        for (idx, sel) in selections.iter().enumerate() {
            write!(f, "{}{}", if idx == 0 { "" } else { ", " }, sel)?;
        }
        Ok(())
    }

    /// Formats a sort prompt after selection.
    #[inline]
    fn format_sort_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        self.format_multi_select_prompt_selection(f, prompt, selections)
    }

    /// Formats a select prompt item.
    fn format_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        write!(f, "{} {}", if active { ">" } else { " " }, text)
    }

    /// Formats a multi select prompt item.
    fn format_multi_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        checked: bool,
        active: bool,
    ) -> fmt::Result {
        write!(
            f,
            "{} {}",
            match (checked, active) {
                (true, true) => "> [x]",
                (true, false) => "  [x]",
                (false, true) => "> [ ]",
                (false, false) => "  [ ]",
            },
            text
        )
    }

    /// Formats a sort prompt item.
    fn format_sort_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        picked: bool,
        active: bool,
    ) -> fmt::Result {
        write!(
            f,
            "{} {}",
            match (picked, active) {
                (true, true) => "> [x]",
                (false, true) => "> [ ]",
                (_, false) => "  [ ]",
            },
            text
        )
    }
}

/// The default theme.
pub struct SimpleTheme;

impl Theme for SimpleTheme {}

/// A colorful theme
pub struct ColorfulTheme {
    /// The style for default values
    pub defaults_style: Style,
    /// The style for prompt
    pub prompt_style: Style,
    /// Prompt prefix value and style
    pub prompt_prefix: StyledObject<String>,
    /// Prompt suffix value and style
    pub prompt_suffix: StyledObject<String>,
    /// Prompt on success prefix value and style
    pub success_prefix: StyledObject<String>,
    /// Prompt on success suffix value and style
    pub success_suffix: StyledObject<String>,
    /// Error prefix value and style
    pub error_prefix: StyledObject<String>,
    /// The style for error message
    pub error_style: Style,
    /// The style for hints
    pub hint_style: Style,
    /// The style for values on prompt success
    pub values_style: Style,
    /// The style for active items
    pub active_item_style: Style,
    /// The style for inactive items
    pub inactive_item_style: Style,
    /// Active item in select prefix value and style
    pub active_item_prefix: StyledObject<String>,
    /// Inctive item in select prefix value and style
    pub inactive_item_prefix: StyledObject<String>,
    /// Checked item in multi select prefix value and style
    pub checked_item_prefix: StyledObject<String>,
    /// Unchecked item in multi select prefix value and style
    pub unchecked_item_prefix: StyledObject<String>,
    /// Picked item in sort prefix value and style
    pub picked_item_prefix: StyledObject<String>,
    /// Unpicked item in sort prefix value and style
    pub unpicked_item_prefix: StyledObject<String>,
    /// Show the selections from certain prompts inline
    pub inline_selections: bool,
}

impl Default for ColorfulTheme {
    fn default() -> ColorfulTheme {
        ColorfulTheme {
            defaults_style: Style::new().for_stderr().cyan(),
            prompt_style: Style::new().for_stderr().bold(),
            prompt_prefix: style("?".to_string()).for_stderr().yellow(),
            prompt_suffix: style("›".to_string()).for_stderr().black().bright(),
            success_prefix: style("✔".to_string()).for_stderr().green(),
            success_suffix: style("·".to_string()).for_stderr().black().bright(),
            error_prefix: style("✘".to_string()).for_stderr().red(),
            error_style: Style::new().for_stderr().red(),
            hint_style: Style::new().for_stderr().black().bright(),
            values_style: Style::new().for_stderr().green(),
            active_item_style: Style::new().for_stderr().cyan(),
            inactive_item_style: Style::new().for_stderr(),
            active_item_prefix: style("❯".to_string()).for_stderr().green(),
            inactive_item_prefix: style(" ".to_string()).for_stderr(),
            checked_item_prefix: style("✔".to_string()).for_stderr().green(),
            unchecked_item_prefix: style("✔".to_string()).for_stderr().black(),
            picked_item_prefix: style("❯".to_string()).for_stderr().green(),
            unpicked_item_prefix: style(" ".to_string()).for_stderr(),
            inline_selections: true,
        }
    }
}

impl Theme for ColorfulTheme {
    /// Formats a prompt.
    fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.prompt_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        write!(f, "{}", &self.prompt_suffix)
    }

    /// Formats an error
    fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
        write!(
            f,
            "{} {}",
            &self.error_prefix,
            self.error_style.apply_to(err)
        )
    }

    /// Formats an input prompt.
    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.prompt_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        match default {
            Some(default) => write!(
                f,
                "{} {} ",
                self.hint_style.apply_to(&format!("({})", default)),
                &self.prompt_suffix
            ),
            None => write!(f, "{} ", &self.prompt_suffix),
        }
    }

    /// Formats a confirm prompt.
    fn format_confirm_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<bool>,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.prompt_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        match default {
            None => write!(
                f,
                "{} {}",
                self.hint_style.apply_to("(y/n)"),
                &self.prompt_suffix
            ),
            Some(true) => write!(
                f,
                "{} {} {}",
                self.hint_style.apply_to("(y/n)"),
                &self.prompt_suffix,
                self.defaults_style.apply_to("yes")
            ),
            Some(false) => write!(
                f,
                "{} {} {}",
                self.hint_style.apply_to("(y/n)"),
                &self.prompt_suffix,
                self.defaults_style.apply_to("no")
            ),
        }
    }

    /// Formats a confirm prompt after selection.
    fn format_confirm_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: Option<bool>,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.success_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }
        let selection = selection.map(|b| if b { "yes" } else { "no" });

        match selection {
            Some(selection) => {
                write!(
                    f,
                    "{} {}",
                    &self.success_suffix,
                    self.values_style.apply_to(selection)
                )
            }
            None => {
                write!(f, "{}", &self.success_suffix)
            }
        }
    }

    /// Formats an input prompt after selection.
    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.success_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        write!(
            f,
            "{} {}",
            &self.success_suffix,
            self.values_style.apply_to(sel)
        )
    }

    /// Formats a password prompt after selection.
    fn format_password_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
    ) -> fmt::Result {
        self.format_input_prompt_selection(f, prompt, "********")
    }

    /// Formats a multi select prompt after selection.
    fn format_multi_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.success_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        write!(f, "{} ", &self.success_suffix)?;

        if self.inline_selections {
            for (idx, sel) in selections.iter().enumerate() {
                write!(
                    f,
                    "{}{}",
                    if idx == 0 { "" } else { ", " },
                    self.values_style.apply_to(sel)
                )?;
            }
        }

        Ok(())
    }

    /// Formats a select prompt item.
    fn format_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        let details = match active {
            true => (
                &self.active_item_prefix,
                self.active_item_style.apply_to(text),
            ),
            false => (
                &self.inactive_item_prefix,
                self.inactive_item_style.apply_to(text),
            ),
        };

        write!(f, "{} {}", details.0, details.1)
    }

    /// Formats a multi select prompt item.
    fn format_multi_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        checked: bool,
        active: bool,
    ) -> fmt::Result {
        let details = match (checked, active) {
            (true, true) => (
                &self.checked_item_prefix,
                self.active_item_style.apply_to(text),
            ),
            (true, false) => (
                &self.checked_item_prefix,
                self.inactive_item_style.apply_to(text),
            ),
            (false, true) => (
                &self.unchecked_item_prefix,
                self.active_item_style.apply_to(text),
            ),
            (false, false) => (
                &self.unchecked_item_prefix,
                self.inactive_item_style.apply_to(text),
            ),
        };

        write!(f, "{} {}", details.0, details.1)
    }

    /// Formats a sort prompt item.
    fn format_sort_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        picked: bool,
        active: bool,
    ) -> fmt::Result {
        let details = match (picked, active) {
            (true, true) => (
                &self.picked_item_prefix,
                self.active_item_style.apply_to(text),
            ),
            (false, true) => (
                &self.unpicked_item_prefix,
                self.active_item_style.apply_to(text),
            ),
            (_, false) => (
                &self.unpicked_item_prefix,
                self.inactive_item_style.apply_to(text),
            ),
        };

        write!(f, "{} {}", details.0, details.1)
    }
}

/// Helper struct to conveniently render a theme ot a term.
pub(crate) struct TermThemeRenderer<'a> {
    term: &'a Term,
    theme: &'a dyn Theme,
    height: usize,
    prompt_height: usize,
    prompts_reset_height: bool,
}

impl<'a> TermThemeRenderer<'a> {
    pub fn new(term: &'a Term, theme: &'a dyn Theme) -> TermThemeRenderer<'a> {
        TermThemeRenderer {
            term,
            theme,
            height: 0,
            prompt_height: 0,
            prompts_reset_height: true,
        }
    }

    pub fn set_prompts_reset_height(&mut self, val: bool) {
        self.prompts_reset_height = val;
    }

    pub fn term(&self) -> &Term {
        self.term
    }

    pub fn add_line(&mut self) {
        self.height += 1;
    }

    fn write_formatted_str<
        F: FnOnce(&mut TermThemeRenderer, &mut dyn fmt::Write) -> fmt::Result,
    >(
        &mut self,
        f: F,
    ) -> io::Result<()> {
        let mut buf = String::new();
        f(self, &mut buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        self.height += buf.chars().filter(|&x| x == '\n').count();
        self.term.write_str(&buf)
    }

    fn write_formatted_line<
        F: FnOnce(&mut TermThemeRenderer, &mut dyn fmt::Write) -> fmt::Result,
    >(
        &mut self,
        f: F,
    ) -> io::Result<()> {
        let mut buf = String::new();
        f(self, &mut buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        self.height += buf.chars().filter(|&x| x == '\n').count() + 1;
        self.term.write_line(&buf)
    }

    fn write_formatted_prompt<
        F: FnOnce(&mut TermThemeRenderer, &mut dyn fmt::Write) -> fmt::Result,
    >(
        &mut self,
        f: F,
    ) -> io::Result<()> {
        self.write_formatted_line(f)?;
        if self.prompts_reset_height {
            self.prompt_height = self.height;
            self.height = 0;
        }
        Ok(())
    }

    pub fn error(&mut self, err: &str) -> io::Result<()> {
        self.write_formatted_line(|this, buf| this.theme.format_error(buf, err))
    }

    pub fn confirm_prompt(&mut self, prompt: &str, default: Option<bool>) -> io::Result<()> {
        self.write_formatted_str(|this, buf| this.theme.format_confirm_prompt(buf, prompt, default))
    }

    pub fn confirm_prompt_selection(&mut self, prompt: &str, sel: Option<bool>) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| {
            this.theme.format_confirm_prompt_selection(buf, prompt, sel)
        })
    }

    pub fn input_prompt(&mut self, prompt: &str, default: Option<&str>) -> io::Result<()> {
        self.write_formatted_str(|this, buf| this.theme.format_input_prompt(buf, prompt, default))
    }

    pub fn input_prompt_selection(&mut self, prompt: &str, sel: &str) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| {
            this.theme.format_input_prompt_selection(buf, prompt, sel)
        })
    }

    pub fn password_prompt(&mut self, prompt: &str) -> io::Result<()> {
        self.write_formatted_str(|this, buf| {
            write!(buf, "\r")?;
            this.theme.format_password_prompt(buf, prompt)
        })
    }

    pub fn password_prompt_selection(&mut self, prompt: &str) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| {
            this.theme.format_password_prompt_selection(buf, prompt)
        })
    }

    pub fn select_prompt(&mut self, prompt: &str) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| this.theme.format_select_prompt(buf, prompt))
    }

    pub fn select_prompt_selection(&mut self, prompt: &str, sel: &str) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| {
            this.theme.format_select_prompt_selection(buf, prompt, sel)
        })
    }

    pub fn select_prompt_item(&mut self, text: &str, active: bool) -> io::Result<()> {
        self.write_formatted_line(|this, buf| {
            this.theme.format_select_prompt_item(buf, text, active)
        })
    }

    pub fn multi_select_prompt(&mut self, prompt: &str) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| this.theme.format_multi_select_prompt(buf, prompt))
    }

    pub fn multi_select_prompt_selection(&mut self, prompt: &str, sel: &[&str]) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| {
            this.theme
                .format_multi_select_prompt_selection(buf, prompt, sel)
        })
    }

    pub fn multi_select_prompt_item(
        &mut self,
        text: &str,
        checked: bool,
        active: bool,
    ) -> io::Result<()> {
        self.write_formatted_line(|this, buf| {
            this.theme
                .format_multi_select_prompt_item(buf, text, checked, active)
        })
    }

    pub fn sort_prompt(&mut self, prompt: &str) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| this.theme.format_sort_prompt(buf, prompt))
    }

    pub fn sort_prompt_selection(&mut self, prompt: &str, sel: &[&str]) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| {
            this.theme.format_sort_prompt_selection(buf, prompt, sel)
        })
    }

    pub fn sort_prompt_item(&mut self, text: &str, picked: bool, active: bool) -> io::Result<()> {
        self.write_formatted_line(|this, buf| {
            this.theme
                .format_sort_prompt_item(buf, text, picked, active)
        })
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.term
            .clear_last_lines(self.height + self.prompt_height)?;
        self.height = 0;
        Ok(())
    }

    pub fn clear_preserve_prompt(&mut self, size_vec: &[usize]) -> io::Result<()> {
        let mut new_height = self.height;
        //Check each item size, increment on finding an overflow
        for size in size_vec {
            if *size > self.term.size().1 as usize {
                new_height += 1;
            }
        }
        self.term.clear_last_lines(new_height)?;
        self.height = 0;
        Ok(())
    }
}
