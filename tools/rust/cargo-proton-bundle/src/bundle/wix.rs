use handlebars::Handlebars;
use lazy_static::lazy_static;
use sha2::Digest;
use slog::warn;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

const WIX_URL: &str =
  "https://github.com/wixtoolset/wix3/releases/download/wix3111rtm/wix311-binaries.zip";
const WIX_SHA256: &str = "37f0a533b0978a454efb5dc3bd3598becf9660aaf4287e55bf68ca6b527d051d";

const VC_REDIST_X86_URL: &str =
    "https://download.visualstudio.microsoft.com/download/pr/c8edbb87-c7ec-4500-a461-71e8912d25e9/99ba493d660597490cbb8b3211d2cae4/vc_redist.x86.exe";

const VC_REDIST_X86_SHA256: &str =
  "3a43e8a55a3f3e4b73d01872c16d47a19dd825756784f4580187309e7d1fcb74";

const VC_REDIST_X64_URL: &str =
    "https://download.visualstudio.microsoft.com/download/pr/9e04d214-5a9d-4515-9960-3d71398d98c3/1e1e62ab57bbb4bf5199e8ce88f040be/vc_redist.x64.exe";

const VC_REDIST_X64_SHA256: &str =
  "d6cd2445f68815fe02489fafe0127819e44851e26dfbe702612bc0d223cbbc2b";

lazy_static! {
  static ref HANDLEBARS: Handlebars = {
    let mut handlebars = Handlebars::new();

    handlebars
      .register_template_string("main.wxs", include_str!("templates/main.wxs"))
      .unwrap();
    handlebars
  };
}
