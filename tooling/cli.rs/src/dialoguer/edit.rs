use std::{
    env,
    ffi::{OsStr, OsString},
    fs, io,
    io::{Read, Write},
    process,
};

/// Launches the default editor to edit a string.
///
/// ## Example
///
/// ```rust,no_run
/// use dialoguer::Editor;
///
/// if let Some(rv) = Editor::new().edit("Enter a commit message").unwrap() {
///     println!("Your message:");
///     println!("{}", rv);
/// } else {
///     println!("Abort!");
/// }
/// ```
pub struct Editor {
    editor: OsString,
    extension: String,
    require_save: bool,
    trim_newlines: bool,
}

fn get_default_editor() -> OsString {
    if let Some(prog) = env::var_os("VISUAL") {
        return prog;
    }
    if let Some(prog) = env::var_os("EDITOR") {
        return prog;
    }
    if cfg!(windows) {
        "notepad.exe".into()
    } else {
        "vi".into()
    }
}

impl Default for Editor {
    fn default() -> Editor {
        Editor::new()
    }
}

impl Editor {
    /// Creates a new editor.
    pub fn new() -> Editor {
        Editor {
            editor: get_default_editor(),
            extension: ".txt".into(),
            require_save: true,
            trim_newlines: true,
        }
    }

    /// Sets a specific editor executable.
    pub fn executable<S: AsRef<OsStr>>(&mut self, val: S) -> &mut Editor {
        self.editor = val.as_ref().into();
        self
    }

    /// Sets a specific extension
    pub fn extension(&mut self, val: &str) -> &mut Editor {
        self.extension = val.into();
        self
    }

    /// Enables or disables the save requirement.
    pub fn require_save(&mut self, val: bool) -> &mut Editor {
        self.require_save = val;
        self
    }

    /// Enables or disables trailing newline stripping.
    ///
    /// This is on by default.
    pub fn trim_newlines(&mut self, val: bool) -> &mut Editor {
        self.trim_newlines = val;
        self
    }

    /// Launches the editor to edit a string.
    ///
    /// Returns `None` if the file was not saved or otherwise the
    /// entered text.
    pub fn edit(&self, s: &str) -> io::Result<Option<String>> {
        let mut f = tempfile::Builder::new()
            .prefix("edit-")
            .suffix(&self.extension)
            .rand_bytes(12)
            .tempfile()?;
        f.write_all(s.as_bytes())?;
        f.flush()?;
        let ts = fs::metadata(f.path())?.modified()?;

        let s: String = self.editor.clone().into_string().unwrap();
        let mut iterator = s.split(' ');
        let cmd = iterator.next().unwrap();
        let args: Vec<&str> = iterator.collect();

        let rv = process::Command::new(cmd)
            .args(args)
            .arg(f.path())
            .spawn()?
            .wait()?;

        if rv.success() && self.require_save && ts >= fs::metadata(f.path())?.modified()? {
            return Ok(None);
        }

        let mut new_f = fs::File::open(f.path())?;
        let mut rv = String::new();
        new_f.read_to_string(&mut rv)?;

        if self.trim_newlines {
            let len = rv.trim_end_matches(&['\n', '\r'][..]).len();
            rv.truncate(len);
        }

        Ok(Some(rv))
    }
}
