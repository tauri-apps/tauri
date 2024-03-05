// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::convert::identity;
use std::path::{Path, PathBuf};
use std::{ffi::OsStr, str::FromStr};

use base64::Engine;
use proc_macro2::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};

use tauri_utils::acl::capability::{Capability, CapabilityFile};
use tauri_utils::acl::manifest::Manifest;
use tauri_utils::acl::resolved::Resolved;
use tauri_utils::assets::AssetKey;
use tauri_utils::config::{CapabilityEntry, Config, FrontendDist, PatternKind};
use tauri_utils::html::{
  inject_nonce_token, parse as parse_html, serialize_node as serialize_html_node,
};
use tauri_utils::platform::Target;
use tauri_utils::tokens::{map_lit, str_lit};

use crate::embedded_assets::{AssetOptions, CspHashes, EmbeddedAssets, EmbeddedAssetsError};

const ACL_MANIFESTS_FILE_NAME: &str = "acl-manifests.json";
const CAPABILITIES_FILE_NAME: &str = "capabilities.json";

/// Necessary data needed by [`context_codegen`] to generate code for a Tauri application context.
pub struct ContextData {
  pub dev: bool,
  pub config: Config,
  pub config_parent: PathBuf,
  pub root: TokenStream,
  /// Additional capabilities to include.
  pub capabilities: Option<Vec<PathBuf>>,
}

fn map_core_assets(
  options: &AssetOptions,
) -> impl Fn(&AssetKey, &Path, &mut Vec<u8>, &mut CspHashes) -> Result<(), EmbeddedAssetsError> {
  #[cfg(feature = "isolation")]
  let pattern = tauri_utils::html::PatternObject::from(&options.pattern);
  let csp = options.csp;
  let dangerous_disable_asset_csp_modification =
    options.dangerous_disable_asset_csp_modification.clone();
  move |key, path, input, csp_hashes| {
    if path.extension() == Some(OsStr::new("html")) {
      #[allow(clippy::collapsible_if)]
      if csp {
        let document = parse_html(String::from_utf8_lossy(input).into_owned());

        inject_nonce_token(&document, &dangerous_disable_asset_csp_modification);

        if dangerous_disable_asset_csp_modification.can_modify("script-src") {
          if let Ok(inline_script_elements) = document.select("script:not(empty)") {
            let mut scripts = Vec::new();
            for inline_script_el in inline_script_elements {
              let script = inline_script_el.as_node().text_contents();
              let mut hasher = Sha256::new();
              hasher.update(&script);
              let hash = hasher.finalize();
              scripts.push(format!(
                "'sha256-{}'",
                base64::engine::general_purpose::STANDARD.encode(hash)
              ));
            }
            csp_hashes
              .inline_scripts
              .entry(key.clone().into())
              .or_default()
              .append(&mut scripts);
          }
        }

        #[cfg(feature = "isolation")]
        if dangerous_disable_asset_csp_modification.can_modify("style-src") {
          if let tauri_utils::html::PatternObject::Isolation { .. } = &pattern {
            // create the csp for the isolation iframe styling now, to make the runtime less complex
            let mut hasher = Sha256::new();
            hasher.update(tauri_utils::pattern::isolation::IFRAME_STYLE);
            let hash = hasher.finalize();
            csp_hashes.styles.push(format!(
              "'sha256-{}'",
              base64::engine::general_purpose::STANDARD.encode(hash)
            ));
          }
        }

        *input = serialize_html_node(&document);
      }
    }
    Ok(())
  }
}

#[cfg(feature = "isolation")]
fn map_isolation(
  _options: &AssetOptions,
  dir: PathBuf,
) -> impl Fn(&AssetKey, &Path, &mut Vec<u8>, &mut CspHashes) -> Result<(), EmbeddedAssetsError> {
  move |_key, path, input, _csp_hashes| {
    if path.extension() == Some(OsStr::new("html")) {
      let isolation_html = tauri_utils::html::parse(String::from_utf8_lossy(input).into_owned());

      // this is appended, so no need to reverse order it
      tauri_utils::html::inject_codegen_isolation_script(&isolation_html);

      // temporary workaround for windows not loading assets
      tauri_utils::html::inline_isolation(&isolation_html, &dir);

      *input = isolation_html.to_string().as_bytes().to_vec()
    }

    Ok(())
  }
}

/// Build a `tauri::Context` for including in application code.
pub fn context_codegen(data: ContextData) -> Result<TokenStream, EmbeddedAssetsError> {
  let ContextData {
    dev,
    config,
    config_parent,
    root,
    capabilities: additional_capabilities,
  } = data;

  let target = std::env::var("TARGET")
    .or_else(|_| std::env::var("TAURI_ENV_TARGET_TRIPLE"))
    .as_deref()
    .map(Target::from_triple)
    .unwrap_or_else(|_| Target::current());

  let mut options = AssetOptions::new(config.app.security.pattern.clone())
    .freeze_prototype(config.app.security.freeze_prototype)
    .dangerous_disable_asset_csp_modification(
      config
        .app
        .security
        .dangerous_disable_asset_csp_modification
        .clone(),
    );
  let csp = if dev {
    config
      .app
      .security
      .dev_csp
      .as_ref()
      .or(config.app.security.csp.as_ref())
  } else {
    config.app.security.csp.as_ref()
  };
  if csp.is_some() {
    options = options.with_csp();
  }

  let assets = if dev && config.build.dev_url.is_some() {
    Default::default()
  } else {
    match &config.build.frontend_dist {
      Some(url) => match url {
        FrontendDist::Url(_url) => Default::default(),
        FrontendDist::Directory(path) => {
          let assets_path = config_parent.join(path);
          if !assets_path.exists() {
            panic!(
              "The `frontendDist` configuration is set to `{:?}` but this path doesn't exist",
              path
            )
          }
          EmbeddedAssets::new(assets_path, &options, map_core_assets(&options))?
        }
        FrontendDist::Files(files) => EmbeddedAssets::new(
          files
            .iter()
            .map(|p| config_parent.join(p))
            .collect::<Vec<_>>(),
          &options,
          map_core_assets(&options),
        )?,
        _ => unimplemented!(),
      },
      None => Default::default(),
    }
  };

  let out_dir = {
    let out_dir = std::env::var("OUT_DIR")
      .map_err(|_| EmbeddedAssetsError::OutDir)
      .map(PathBuf::from)
      .and_then(|p| p.canonicalize().map_err(|_| EmbeddedAssetsError::OutDir))?;

    // make sure that our output directory is created
    std::fs::create_dir_all(&out_dir).map_err(|_| EmbeddedAssetsError::OutDir)?;

    out_dir
  };

  let default_window_icon = {
    if target == Target::Windows {
      // handle default window icons for Windows targets
      let icon_path = find_icon(
        &config,
        &config_parent,
        |i| i.ends_with(".ico"),
        "icons/icon.ico",
      );
      if icon_path.exists() {
        ico_icon(&root, &out_dir, icon_path).map(|i| quote!(::std::option::Option::Some(#i)))?
      } else {
        let icon_path = find_icon(
          &config,
          &config_parent,
          |i| i.ends_with(".png"),
          "icons/icon.png",
        );
        png_icon(&root, &out_dir, icon_path).map(|i| quote!(::std::option::Option::Some(#i)))?
      }
    } else {
      // handle default window icons for Unix targets
      let icon_path = find_icon(
        &config,
        &config_parent,
        |i| i.ends_with(".png"),
        "icons/icon.png",
      );
      png_icon(&root, &out_dir, icon_path).map(|i| quote!(::std::option::Option::Some(#i)))?
    }
  };

  let app_icon = if target == Target::MacOS && dev {
    let mut icon_path = find_icon(
      &config,
      &config_parent,
      |i| i.ends_with(".icns"),
      "icons/icon.png",
    );
    if !icon_path.exists() {
      icon_path = find_icon(
        &config,
        &config_parent,
        |i| i.ends_with(".png"),
        "icons/icon.png",
      );
    }
    raw_icon(&out_dir, icon_path)?
  } else {
    quote!(::std::option::Option::None)
  };

  let package_name = if let Some(product_name) = &config.product_name {
    quote!(#product_name.to_string())
  } else {
    quote!(env!("CARGO_PKG_NAME").to_string())
  };
  let package_version = if let Some(version) = &config.version {
    semver::Version::from_str(version)?;
    quote!(#version.to_string())
  } else {
    quote!(env!("CARGO_PKG_VERSION").to_string())
  };
  let package_info = quote!(
    #root::PackageInfo {
      name: #package_name,
      version: #package_version.parse().unwrap(),
      authors: env!("CARGO_PKG_AUTHORS"),
      description: env!("CARGO_PKG_DESCRIPTION"),
      crate_name: env!("CARGO_PKG_NAME"),
    }
  );

  let with_tray_icon_code = if target.is_desktop() {
    if let Some(tray) = &config.app.tray_icon {
      let tray_icon_icon_path = config_parent.join(&tray.icon_path);
      let ext = tray_icon_icon_path.extension();
      if ext.map_or(false, |e| e == "ico") {
        ico_icon(&root, &out_dir, tray_icon_icon_path)
          .map(|i| quote!(context.set_tray_icon(Some(#i));))?
      } else if ext.map_or(false, |e| e == "png") {
        png_icon(&root, &out_dir, tray_icon_icon_path)
          .map(|i| quote!(context.set_tray_icon(Some(#i));))?
      } else {
        quote!(compile_error!(
          "The tray icon extension must be either `.ico` or `.png`."
        ))
      }
    } else {
      quote!()
    }
  } else {
    quote!()
  };

  #[cfg(target_os = "macos")]
  let info_plist = if target == Target::MacOS && dev {
    let info_plist_path = config_parent.join("Info.plist");
    let mut info_plist = if info_plist_path.exists() {
      plist::Value::from_file(&info_plist_path)
        .unwrap_or_else(|e| panic!("failed to read plist {}: {}", info_plist_path.display(), e))
    } else {
      plist::Value::Dictionary(Default::default())
    };

    if let Some(plist) = info_plist.as_dictionary_mut() {
      if let Some(product_name) = &config.product_name {
        plist.insert("CFBundleName".into(), product_name.clone().into());
      }
      if let Some(version) = &config.version {
        plist.insert("CFBundleShortVersionString".into(), version.clone().into());
      }
      let format =
        time::format_description::parse("[year][month][day].[hour][minute][second]").unwrap();
      if let Ok(build_number) = time::OffsetDateTime::now_utc().format(&format) {
        plist.insert("CFBundleVersion".into(), build_number.into());
      }
    }

    info_plist
      .to_file_xml(out_dir.join("Info.plist"))
      .expect("failed to write Info.plist");
    quote!({
      tauri::embed_plist::embed_info_plist!(concat!(std::env!("OUT_DIR"), "/Info.plist"));
    })
  } else {
    quote!(())
  };
  #[cfg(not(target_os = "macos"))]
  let info_plist = quote!(());

  let pattern = match &options.pattern {
    PatternKind::Brownfield => quote!(#root::Pattern::Brownfield(std::marker::PhantomData)),
    #[cfg(not(feature = "isolation"))]
    PatternKind::Isolation { dir: _ } => {
      quote!(#root::Pattern::Brownfield(std::marker::PhantomData))
    }
    #[cfg(feature = "isolation")]
    PatternKind::Isolation { dir } => {
      let dir = config_parent.join(dir);
      if !dir.exists() {
        panic!("The isolation application path is set to `{dir:?}` but it does not exist")
      }

      let mut sets_isolation_hook = false;

      let key = uuid::Uuid::new_v4().to_string();
      let map_isolation = map_isolation(&options, dir.clone());
      let assets = EmbeddedAssets::new(dir, &options, |key, path, input, csp_hashes| {
        // we check if `__TAURI_ISOLATION_HOOK__` exists in the isolation code
        // before modifying the files since we inject our own `__TAURI_ISOLATION_HOOK__` reference in HTML files
        if String::from_utf8_lossy(input).contains("__TAURI_ISOLATION_HOOK__") {
          sets_isolation_hook = true;
        }
        map_isolation(key, path, input, csp_hashes)
      })?;

      if !sets_isolation_hook {
        panic!("The isolation application does not contain a file setting the `window.__TAURI_ISOLATION_HOOK__` value.");
      }

      let schema = options.isolation_schema;

      quote!(#root::Pattern::Isolation {
        assets: ::std::sync::Arc::new(#assets),
        schema: #schema.into(),
        key: #key.into(),
        crypto_keys: std::boxed::Box::new(::tauri::utils::pattern::isolation::Keys::new().expect("unable to generate cryptographically secure keys for Tauri \"Isolation\" Pattern")),
      })
    }
  };

  let acl_file_path = out_dir.join(ACL_MANIFESTS_FILE_NAME);
  let acl: BTreeMap<String, Manifest> = if acl_file_path.exists() {
    let acl_file =
      std::fs::read_to_string(acl_file_path).expect("failed to read plugin manifest map");
    serde_json::from_str(&acl_file).expect("failed to parse plugin manifest map")
  } else {
    Default::default()
  };

  let capabilities_file_path = out_dir.join(CAPABILITIES_FILE_NAME);
  let mut capabilities_from_files: BTreeMap<String, Capability> = if capabilities_file_path.exists()
  {
    let capabilities_file =
      std::fs::read_to_string(capabilities_file_path).expect("failed to read capabilities");
    serde_json::from_str(&capabilities_file).expect("failed to parse capabilities")
  } else {
    Default::default()
  };

  let mut capabilities = if config.app.security.capabilities.is_empty() {
    capabilities_from_files
  } else {
    let mut capabilities = BTreeMap::new();
    for capability_entry in &config.app.security.capabilities {
      match capability_entry {
        CapabilityEntry::Inlined(capability) => {
          capabilities.insert(capability.identifier.clone(), capability.clone());
        }
        CapabilityEntry::Reference(id) => {
          let capability = capabilities_from_files
            .remove(id)
            .unwrap_or_else(|| panic!("capability with identifier {id} not found"));
          capabilities.insert(id.clone(), capability);
        }
      }
    }
    capabilities
  };

  let acl_tokens = map_lit(
    quote! { ::std::collections::BTreeMap },
    &acl,
    str_lit,
    identity,
  );

  if let Some(paths) = additional_capabilities {
    for path in paths {
      let capability = CapabilityFile::load(&path)
        .unwrap_or_else(|e| panic!("failed to read capability {}: {e}", path.display()));
      match capability {
        CapabilityFile::Capability(c) => {
          capabilities.insert(c.identifier.clone(), c);
        }
        CapabilityFile::List(capabilities_list)
        | CapabilityFile::NamedList {
          capabilities: capabilities_list,
        } => {
          capabilities.extend(
            capabilities_list
              .into_iter()
              .map(|c| (c.identifier.clone(), c)),
          );
        }
      }
    }
  }

  let resolved = Resolved::resolve(&acl, capabilities, target).expect("failed to resolve ACL");
  let runtime_authority = quote!(#root::ipc::RuntimeAuthority::new(#acl_tokens, #resolved));

  Ok(quote!({
    #[allow(unused_mut, clippy::let_and_return)]
    let mut context = #root::Context::new(
      #config,
      ::std::boxed::Box::new(#assets),
      #default_window_icon,
      #app_icon,
      #package_info,
      #info_plist,
      #pattern,
      #runtime_authority
    );
    #with_tray_icon_code
    context
  }))
}

fn ico_icon<P: AsRef<Path>>(
  root: &TokenStream,
  out_dir: &Path,
  path: P,
) -> Result<TokenStream, EmbeddedAssetsError> {
  let path = path.as_ref();
  let bytes = std::fs::read(path)
    .unwrap_or_else(|e| panic!("failed to read icon {}: {}", path.display(), e))
    .to_vec();
  let icon_dir = ico::IconDir::read(std::io::Cursor::new(bytes))
    .unwrap_or_else(|e| panic!("failed to parse icon {}: {}", path.display(), e));
  let entry = &icon_dir.entries()[0];
  let rgba = entry
    .decode()
    .unwrap_or_else(|e| panic!("failed to decode icon {}: {}", path.display(), e))
    .rgba_data()
    .to_vec();
  let width = entry.width();
  let height = entry.height();

  let icon_file_name = path.file_name().unwrap();
  let out_path = out_dir.join(icon_file_name);
  write_if_changed(&out_path, &rgba).map_err(|error| EmbeddedAssetsError::AssetWrite {
    path: path.to_owned(),
    error,
  })?;

  let icon_file_name = icon_file_name.to_str().unwrap();
  let icon = quote!(#root::Image::new(include_bytes!(concat!(std::env!("OUT_DIR"), "/", #icon_file_name)), #width, #height));
  Ok(icon)
}

fn raw_icon<P: AsRef<Path>>(out_dir: &Path, path: P) -> Result<TokenStream, EmbeddedAssetsError> {
  let path = path.as_ref();
  let bytes = std::fs::read(path)
    .unwrap_or_else(|e| panic!("failed to read icon {}: {}", path.display(), e))
    .to_vec();

  let out_path = out_dir.join(path.file_name().unwrap());
  write_if_changed(&out_path, &bytes).map_err(|error| EmbeddedAssetsError::AssetWrite {
    path: path.to_owned(),
    error,
  })?;

  let icon_path = path.file_name().unwrap().to_str().unwrap().to_string();
  let icon = quote!(::std::option::Option::Some(
    include_bytes!(concat!(std::env!("OUT_DIR"), "/", #icon_path)).to_vec()
  ));
  Ok(icon)
}

fn png_icon<P: AsRef<Path>>(
  root: &TokenStream,
  out_dir: &Path,
  path: P,
) -> Result<TokenStream, EmbeddedAssetsError> {
  let path = path.as_ref();
  let bytes = std::fs::read(path)
    .unwrap_or_else(|e| panic!("failed to read icon {}: {}", path.display(), e))
    .to_vec();
  let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
  let mut reader = decoder
    .read_info()
    .unwrap_or_else(|e| panic!("failed to read icon {}: {}", path.display(), e));

  let (color_type, _) = reader.output_color_type();

  if color_type != png::ColorType::Rgba {
    panic!("icon {} is not RGBA", path.display());
  }

  let mut buffer: Vec<u8> = Vec::new();
  while let Ok(Some(row)) = reader.next_row() {
    buffer.extend(row.data());
  }
  let width = reader.info().width;
  let height = reader.info().height;

  let icon_file_name = path.file_name().unwrap();
  let out_path = out_dir.join(icon_file_name);
  write_if_changed(&out_path, &buffer).map_err(|error| EmbeddedAssetsError::AssetWrite {
    path: path.to_owned(),
    error,
  })?;

  let icon_file_name = icon_file_name.to_str().unwrap();
  let icon = quote!(#root::Image::new(include_bytes!(concat!(std::env!("OUT_DIR"), "/", #icon_file_name)), #width, #height));
  Ok(icon)
}

fn write_if_changed(out_path: &Path, data: &[u8]) -> std::io::Result<()> {
  use std::fs::File;
  use std::io::Write;

  if let Ok(curr) = std::fs::read(out_path) {
    if curr == data {
      return Ok(());
    }
  }

  let mut out_file = File::create(out_path)?;
  out_file.write_all(data)
}

fn find_icon<F: Fn(&&String) -> bool>(
  config: &Config,
  config_parent: &Path,
  predicate: F,
  default: &str,
) -> PathBuf {
  let icon_path = config
    .bundle
    .icon
    .iter()
    .find(|i| predicate(i))
    .cloned()
    .unwrap_or_else(|| default.to_string());
  config_parent.join(icon_path)
}
