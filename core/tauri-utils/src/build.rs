//! Build script utilities.

/// Link a Swift library.
#[cfg(target_os = "macos")]
pub fn link_swift_library(name: &str, source: impl AsRef<std::path::Path>) {
  let source = source.as_ref();

  let sdk_root = std::env::var_os("SDKROOT");
  std::env::remove_var("SDKROOT");

  swift_rs::SwiftLinker::new(
    &std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "10.13".into()),
  )
  .with_ios(&std::env::var("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "13.0".into()))
  .with_package(name, source)
  .link();

  if let Some(root) = sdk_root {
    std::env::set_var("SDKROOT", root);
  }
}
