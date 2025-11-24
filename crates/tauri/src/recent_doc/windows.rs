use crate::command;

#[command(root = "crate")]
/// add recent
pub fn add_recent_document(path: &str) -> crate::Result<()> {
  #[cfg(windows)]
  {
    use windows::{core::*, Win32::Foundation::*, Win32::System::Com::*, Win32::UI::Shell::*};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    unsafe {
      let path_wide: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(iter::once(0))
        .collect();
      let item = SHCreateItemFromParsingName(path_wide.as_str(), None).ok()?;

      let info: SHARDAPPIDINFO = SHARDAPPIDINFO {
        psi: Some(item),
        pszAppID: GetAppUserModelId()?,
      };

      SHAddToRecentDocs(SHARD_APPIDINFO, &info as *const _);
    }
  }

  #[cfg(target_os = "macos")]
  {
    use objc2_appkit::NSDocumentController;
    use objc2::rc::Id;
    use objc2::foundation::{NSString, NSURL};

    unsafe {
      let ns_path = NSURL::file_url_with_path(&NSString::from_str(path));
      let controller: Id<NSDocumentController> = NSDocumentController::shared_document_controller();
      controller.note_new_recent_document_url(&ns_path);
    }
  }

  Ok(())
}

#[command(root = "crate")]
pub fn clear_recent_documents() -> crate::Result<()> {
  #[cfg(windows)]
  {
    use windows::Win32::UI::Shell::*;
    unsafe {
      SHAddToRecentDocs(SHARD_APPIDINFO, std::ptr::null());
    }
  }

  #[cfg(target_os = "macos")]
  {
    use objc2_appkit::NSDocumentController;
    use objc2::rc::Id;

    unsafe {
      let controller: Id<NSDocumentController> = NSDocumentController::shared_document_controller();
      controller.clear_recent_documents();
    }
  }

  Ok(())
}

#[command(root = "crate")]
pub fn get_recent_documents() -> crate::Result<Vec<String>> {
  let mut recent_docs = Vec::new();

  #[cfg(windows)]
  {
    use std::fs;
    use std::path::{Path, PathBuf};
    use windows::{core::*, Win32::Foundation::*, Win32::System::Com::*, Win32::UI::Shell::*};
    unsafe {
      let recent_path_ptr: PWSTR = SHGetKnownFolderPath(&FOLDERID_Recent, 0, None)?;

      if !recent_path_ptr.is_null() {
        let recent_path = PWSTR::from_raw(recent_path_ptr.0);
        let recent_os_string = recent_path.to_string();
        let recent_folder = PathBuf::from(recent_os_string);

        if let Ok(entries) = fs::read_dir(recent_folder) {
          for entry in entries.flatten() {
            if let Ok(entry) = entry {
              let path = entry.path();

              if path.extension().and_then(|s| s.to_str()) == Some("lnk") {
                if let Ok(resolved_path) = Self::resolve_shortcut(&path) {
                  recent_docs.push(resolved_path);
                }
              }
            }
          }
        }

        CoTaskMemFree(recent_path_ptr.0 as *mut _);
      }
    }
  }

  #[cfg(target_os = "macos")]
  {
    use objc2_appkit::NSDocumentController;
    use objc2::rc::Id;
    use objc2::foundation::{NSArray, NSString};

    unsafe {
      let controller: Id<NSDocumentController> = NSDocumentController::shared_document_controller();
      let urls: Id<NSArray<NSString>> = controller.recent_document_urls();

      for i in 0..urls.count() {
        if let Some(ns_string) = urls.object_at(i) {
          recent_docs.push(ns_string.to_string());
        }
      }
    }
  }

  Ok(recent_docs)
}

#[cfg(windows)]
fn resolve_shortcut(lnk_path: &Path) -> crate::Result<String> {
  let path_string = String::new();

  use windows::{core::*, Win32::Foundation::*, Win32::System::Com::*, Win32::UI::Shell::*};
  unsafe {
    // Create IShellLink instance
    let shell_link: IShellLinkW =
      CoCreateInstance(&ShellLink as *const _, None, CLSCTX_INPROC_SERVER)?;

    // Get IPersistFile interface
    let persist_file: IPersistFile = shell_link.cast()?;

    // Convert path to wide string
    let path_wide = HSTRING::from(lnk_path);

    // Load the shortcut file
    persist_file.Load(&path_wide, STGM_READ)?;

    // Resolve the target path
    let mut target_path = [0u16; MAX_PATH as usize];
    shell_link.GetPath(&mut target_path, None, None, 0)?;

    // Convert wide string to regular string
    let path_string = PWSTR::from_raw(target_path.as_mut_ptr()).to_string()?;
  }

  Ok(path_string)
}
