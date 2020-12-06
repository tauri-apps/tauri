use crate::Settings;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

/// Returns true if the path has a filename indicating that it is a high-desity
/// "retina" icon.  Specifically, returns true the the file stem ends with
/// "@2x" (a convention specified by the [Apple developer docs](
/// https://developer.apple.com/library/mac/documentation/GraphicsAnimation/Conceptual/HighResolutionOSX/Optimizing/Optimizing.html)).
pub fn is_retina<P: AsRef<Path>>(path: P) -> bool {
  path
    .as_ref()
    .file_stem()
    .and_then(OsStr::to_str)
    .map(|stem| stem.ends_with("@2x"))
    .unwrap_or(false)
}

/// Creates a new file at the given path, creating any parent directories as
/// needed.
pub fn create_file(path: &Path) -> crate::Result<BufWriter<File>> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(&parent)?;
  }
  let file = File::create(path)?;
  Ok(BufWriter::new(file))
}

/// Makes a symbolic link to a directory.
#[cfg(unix)]
fn symlink_dir(src: &Path, dst: &Path) -> io::Result<()> {
  std::os::unix::fs::symlink(src, dst)
}

/// Makes a symbolic link to a directory.
#[cfg(windows)]
fn symlink_dir(src: &Path, dst: &Path) -> io::Result<()> {
  std::os::windows::fs::symlink_dir(src, dst)
}

/// Makes a symbolic link to a file.
#[cfg(unix)]
fn symlink_file(src: &Path, dst: &Path) -> io::Result<()> {
  std::os::unix::fs::symlink(src, dst)
}

/// Makes a symbolic link to a file.
#[cfg(windows)]
fn symlink_file(src: &Path, dst: &Path) -> io::Result<()> {
  std::os::windows::fs::symlink_file(src, dst)
}

/// Copies a regular file from one path to another, creating any parent
/// directories of the destination path as necessary.  Fails if the source path
/// is a directory or doesn't exist.
pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> crate::Result<()> {
  let from = from.as_ref();
  let to = to.as_ref();
  if !from.exists() {
    return Err(crate::Error::GenericError(format!(
      "{:?} does not exist",
      from
    )));
  }
  if !from.is_file() {
    return Err(crate::Error::GenericError(format!(
      "{:?} is not a file",
      from
    )));
  }
  let dest_dir = to.parent().expect("No data in parent");
  fs::create_dir_all(dest_dir)?;
  fs::copy(from, to)?;
  Ok(())
}

/// Recursively copies a directory file from one path to another, creating any
/// parent directories of the destination path as necessary.  Fails if the
/// source path is not a directory or doesn't exist, or if the destination path
/// already exists.
pub fn copy_dir(from: &Path, to: &Path) -> crate::Result<()> {
  if !from.exists() {
    return Err(crate::Error::GenericError(format!(
      "{:?} does not exist",
      from
    )));
  }
  if !from.is_dir() {
    return Err(crate::Error::GenericError(format!(
      "{:?} is not a Directory",
      from
    )));
  }
  if to.exists() {
    return Err(crate::Error::GenericError(format!(
      "{:?} already exists",
      from
    )));
  }
  let parent = to.parent().expect("No data in parent");
  fs::create_dir_all(parent)?;
  for entry in walkdir::WalkDir::new(from) {
    let entry = entry?;
    debug_assert!(entry.path().starts_with(from));
    let rel_path = entry.path().strip_prefix(from)?;
    let dest_path = to.join(rel_path);
    if entry.file_type().is_symlink() {
      let target = fs::read_link(entry.path())?;
      if entry.path().is_dir() {
        symlink_dir(&target, &dest_path)?;
      } else {
        symlink_file(&target, &dest_path)?;
      }
    } else if entry.file_type().is_dir() {
      fs::create_dir(dest_path)?;
    } else {
      fs::copy(entry.path(), dest_path)?;
    }
  }
  Ok(())
}

/// Given a path (absolute or relative) to a resource file, returns the
/// relative path from the bundle resources directory where that resource
/// should be stored.
pub fn resource_relpath(path: &Path) -> PathBuf {
  let mut dest = PathBuf::new();
  for component in path.components() {
    match component {
      Component::Prefix(_) => {}
      Component::RootDir => dest.push("_root_"),
      Component::CurDir => {}
      Component::ParentDir => dest.push("_up_"),
      Component::Normal(string) => dest.push(string),
    }
  }
  dest
}

/// Prints a message to stderr, in the same format that `cargo` uses,
/// indicating that we are creating a bundle with the given filename.
pub fn print_bundling(filename: &str) -> crate::Result<()> {
  print_progress("Bundling", filename)
}

/// Prints a message to stderr, in the same format that `cargo` uses,
/// indicating that we have finished the the given bundles.
pub fn print_finished(output_paths: &[PathBuf]) -> crate::Result<()> {
  let pluralised = if output_paths.len() == 1 {
    "bundle"
  } else {
    "bundles"
  };
  let msg = format!("{} {} at:", output_paths.len(), pluralised);
  print_progress("Finished", &msg)?;
  for path in output_paths {
    println!("        {}", path.display());
  }
  Ok(())
}

/// Safely adds the terminal attribute to the terminal output.
/// If the terminal doesn't support the attribute, does nothing.
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

/// Prints a formatted bundle progress to stderr.
fn print_progress(step: &str, msg: &str) -> crate::Result<()> {
  if let Some(mut output) = term::stderr() {
    safe_term_attr(&mut *output, term::Attr::Bold)?;
    output.fg(term::color::GREEN)?;
    write!(output, "    {}", step)?;
    output.reset()?;
    writeln!(output, " {}", msg)?;
    output.flush()?;
    Ok(())
  } else {
    let mut output = io::stderr();
    write!(output, "    {}", step)?;
    writeln!(output, " {}", msg)?;
    output.flush()?;
    Ok(())
  }
}

/// Prints a warning message to stderr, in the same format that `cargo` uses.
pub fn print_warning(message: &str) -> crate::Result<()> {
  if let Some(mut output) = term::stderr() {
    safe_term_attr(&mut *output, term::Attr::Bold)?;
    output.fg(term::color::YELLOW)?;
    write!(output, "warning:")?;
    output.reset()?;
    writeln!(output, " {}", message)?;
    output.flush()?;
    Ok(())
  } else {
    let mut output = io::stderr();
    write!(output, "warning:")?;
    writeln!(output, " {}", message)?;
    output.flush()?;
    Ok(())
  }
}

/// Prints a Info message to stderr.
pub fn print_info(message: &str) -> crate::Result<()> {
  if let Some(mut output) = term::stderr() {
    safe_term_attr(&mut *output, term::Attr::Bold)?;
    output.fg(term::color::GREEN)?;
    write!(output, "info:")?;
    output.reset()?;
    writeln!(output, " {}", message)?;
    output.flush()?;
    Ok(())
  } else {
    let mut output = io::stderr();
    write!(output, "info:")?;
    writeln!(output, " {}", message)?;
    output.flush()?;
    Ok(())
  }
}

/// Prints an error to stderr, in the same format that `cargo` uses.
pub fn print_error(error: &anyhow::Error) -> crate::Result<()> {
  if let Some(mut output) = term::stderr() {
    safe_term_attr(&mut *output, term::Attr::Bold)?;
    output.fg(term::color::RED)?;
    write!(output, "error:")?;
    output.reset()?;
    safe_term_attr(&mut *output, term::Attr::Bold)?;
    writeln!(output, " {}", error)?;
    output.reset()?;
    for cause in error.chain().skip(1) {
      writeln!(output, "  Caused by: {}", cause)?;
    }
    // Add Backtrace once its stable.
    // if let Some(backtrace) = error.backtrace() {
    //   writeln!(output, "{:?}", backtrace)?;
    // }
    output.flush()?;
    std::process::exit(1)
  } else {
    let mut output = io::stderr();
    write!(output, "error:")?;
    writeln!(output, " {}", error)?;
    for cause in error.chain().skip(1) {
      writeln!(output, "  Caused by: {}", cause)?;
    }
    // if let Some(backtrace) = error.backtrace() {
    //   writeln!(output, "{:?}", backtrace)?;
    // }
    output.flush()?;
    std::process::exit(1)
  }
}

pub fn execute_with_verbosity(cmd: &mut Command, settings: &Settings) -> crate::Result<()> {
  let stdio_config = if settings.is_verbose() {
    Stdio::piped
  } else {
    Stdio::null
  };
  let mut child = cmd
    .stdout(stdio_config())
    .stderr(stdio_config())
    .spawn()
    .expect("failed to spawn command");
  if settings.is_verbose() {
    let stdout = child.stdout.as_mut().expect("Failed to get stdout handle");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
      println!("{}", line.expect("Failed to get line"));
    }
  }

  let status = child.wait()?;
  if status.success() {
    Ok(())
  } else {
    Err(anyhow::anyhow!("command failed").into())
  }
}

#[cfg(test)]
mod tests {
  use super::{copy_dir, create_file, is_retina, resource_relpath, symlink_file};
  use std::io::Write;
  use std::path::PathBuf;

  #[test]
  fn create_file_with_parent_dirs() {
    let tmp = tempfile::tempdir().expect("Unable to create temp dir");
    assert!(!tmp.path().join("parent").exists());
    {
      let mut file =
        create_file(&tmp.path().join("parent/file.txt")).expect("Failed to create file");
      writeln!(file, "Hello, world!").expect("unable to write file");
    }
    assert!(tmp.path().join("parent").is_dir());
    assert!(tmp.path().join("parent/file.txt").is_file());
  }

  #[cfg(not(windows))]
  #[test]
  fn copy_dir_with_symlinks() {
    // Create a directory structure that looks like this:
    //   ${TMP}/orig/
    //       sub/
    //           file.txt
    //       link -> sub/file.txt
    let tmp = tempfile::tempdir().expect("unable to create tempdir");
    {
      let mut file =
        create_file(&tmp.path().join("orig/sub/file.txt")).expect("Unable to create file");
      writeln!(file, "Hello, world!").expect("Unable to write to file");
    }
    symlink_file(
      &PathBuf::from("sub/file.txt"),
      &tmp.path().join("orig/link"),
    )
    .expect("Failed to create symlink");
    assert_eq!(
      std::fs::read(tmp.path().join("orig/link"))
        .expect("Failed to read file")
        .as_slice(),
      b"Hello, world!\n"
    );
    // Copy ${TMP}/orig to ${TMP}/parent/copy, and make sure that the
    // directory structure, file, and symlink got copied correctly.
    copy_dir(&tmp.path().join("orig"), &tmp.path().join("parent/copy"))
      .expect("Failed to copy dir");
    assert!(tmp.path().join("parent/copy").is_dir());
    assert!(tmp.path().join("parent/copy/sub").is_dir());
    assert!(tmp.path().join("parent/copy/sub/file.txt").is_file());
    assert_eq!(
      std::fs::read(tmp.path().join("parent/copy/sub/file.txt"))
        .expect("Failed to read file")
        .as_slice(),
      b"Hello, world!\n"
    );
    assert!(tmp.path().join("parent/copy/link").exists());
    assert_eq!(
      std::fs::read_link(tmp.path().join("parent/copy/link")).expect("Failed to read from symlink"),
      PathBuf::from("sub/file.txt")
    );
    assert_eq!(
      std::fs::read(tmp.path().join("parent/copy/link"))
        .expect("Failed to read from file")
        .as_slice(),
      b"Hello, world!\n"
    );
  }

  #[test]
  fn retina_icon_paths() {
    assert!(!is_retina("data/icons/512x512.png"));
    assert!(is_retina("data/icons/512x512@2x.png"));
  }

  #[test]
  fn resource_relative_paths() {
    assert_eq!(
      resource_relpath(&PathBuf::from("./data/images/button.png")),
      PathBuf::from("data/images/button.png")
    );
    assert_eq!(
      resource_relpath(&PathBuf::from("../../images/wheel.png")),
      PathBuf::from("_up_/_up_/images/wheel.png")
    );
    assert_eq!(
      resource_relpath(&PathBuf::from("/home/ferris/crab.png")),
      PathBuf::from("_root_/home/ferris/crab.png")
    );
  }
}
