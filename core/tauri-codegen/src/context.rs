// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::convert::identity;
use std::path::{Path, PathBuf};
use std::{ffi::OsStr, str::FromStr};

use base64::Engine;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use sha2::{Digest, Sha256};

use syn::punctuated::Punctuated;
use syn::{Expr, Ident, PathArguments, PathSegment, Token};
use tauri_utils::acl::capability::{Capability, CapabilityFile};
use tauri_utils::acl::manifest::Manifest;
use tauri_utils::acl::resolved::Resolved;
use tauri_utils::assets::AssetKey;
use tauri_utils::config::{CapabilityEntry, Config, FrontendDist, PatternKind};
use tauri_utils::html::{
  inject_nonce_token, parse as parse_html, serialize_node as serialize_html_node, NodeRef,
};
use tauri_utils::platform::Target;
use tauri_utils::plugin::GLOBAL_API_SCRIPT_FILE_LIST_PATH;
use tauri_utils::tokens::{map_lit, str_lit};

use crate::embedded_assets::{
  AssetOptions, CspHashes, EmbeddedAssets, EmbeddedAssetsError, EmbeddedAssetsResult,
};

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
  /// The custom assets implementation
  pub assets: Option<Expr>,
}

fn inject_script_hashes(document: &NodeRef, key: &AssetKey, csp_hashes: &mut CspHashes) {
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

fn map_core_assets(
  options: &AssetOptions,
) -> impl Fn(&AssetKey, &Path, &mut Vec<u8>, &mut CspHashes) -> EmbeddedAssetsResult<()> {
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
          inject_script_hashes(&document, key, csp_hashes);
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
) -> impl Fn(&AssetKey, &Path, &mut Vec<u8>, &mut CspHashes) -> EmbeddedAssetsResult<()> {
  // create the csp for the isolation iframe styling now, to make the runtime less complex
  let mut hasher = Sha256::new();
  hasher.update(tauri_utils::pattern::isolation::IFRAME_STYLE);
  let hash = hasher.finalize();
  let iframe_style_csp_hash = format!(
    "'sha256-{}'",
    base64::engine::general_purpose::STANDARD.encode(hash)
  );

  move |key, path, input, csp_hashes| {
    if path.extension() == Some(OsStr::new("html")) {
      let isolation_html = parse_html(String::from_utf8_lossy(input).into_owned());

      // this is appended, so no need to reverse order it
      tauri_utils::html::inject_codegen_isolation_script(&isolation_html);

      // temporary workaround for windows not loading assets
      tauri_utils::html::inline_isolation(&isolation_html, &dir);

      inject_nonce_token(
        &isolation_html,
        &tauri_utils::config::DisabledCspModificationKind::Flag(false),
      );

      inject_script_hashes(&isolation_html, key, csp_hashes);

      csp_hashes.styles.push(iframe_style_csp_hash.clone());

      *input = isolation_html.to_string().as_bytes().to_vec()
    }

    Ok(())
  }
}

/// Build a `tauri::Context` for including in application code.
pub fn context_codegen(data: ContextData) -> EmbeddedAssetsResult<TokenStream> {
  let ContextData {
    dev,
    config,
    config_parent,
    root,
    capabilities: additional_capabilities,
    assets,
  } = data;

  let target = std::env::var("TAURI_ENV_TARGET_TRIPLE")
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

  let assets = if let Some(assets) = assets {
    quote!(#assets)
  } else if dev && config.build.dev_url.is_some() {
    let assets = EmbeddedAssets::default();
    quote!(#assets)
  } else {
    let assets = match &config.build.frontend_dist {
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
    };
    quote!(#assets)
  };

  let out_dir = make_output_directory()?;

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
        ico_icon(&root, &out_dir, &icon_path).map(|i| quote!(::std::option::Option::Some(#i)))?
      } else {
        let icon_path = find_icon(
          &config,
          &config_parent,
          |i| i.ends_with(".png"),
          "icons/icon.png",
        );
        png_icon(&root, &out_dir, &icon_path).map(|i| quote!(::std::option::Option::Some(#i)))?
      }
    } else {
      // handle default window icons for Unix targets
      let icon_path = find_icon(
        &config,
        &config_parent,
        |i| i.ends_with(".png"),
        "icons/icon.png",
      );
      png_icon(&root, &out_dir, &icon_path).map(|i| quote!(::std::option::Option::Some(#i)))?
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
    raw_icon(&out_dir, &icon_path)?
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
      let result = image_icon(&root, &out_dir, &tray_icon_icon_path)
        .map(|i| quote!(context.set_tray_icon(Some(#i));));
      if let Err(EmbeddedAssetsError::InvalidImageExtension { .. }) = &result {
        quote!(compile_error!("The tray icon extension must be either `.ico` or `.png`, but {extension} is given in {path}"))
      } else {
        result?
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
    PatternKind::Brownfield => quote!(#root::Pattern::Brownfield),
    #[cfg(not(feature = "isolation"))]
    PatternKind::Isolation { dir: _ } => {
      quote!(#root::Pattern::Brownfield)
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

  let plugin_global_api_script_file_list_path = out_dir.join(GLOBAL_API_SCRIPT_FILE_LIST_PATH);
  let plugin_global_api_script =
    if config.app.with_global_tauri && plugin_global_api_script_file_list_path.exists() {
      let file_list_str = std::fs::read_to_string(plugin_global_api_script_file_list_path)
        .expect("failed to read plugin global API script paths");
      let file_list = serde_json::from_str::<Vec<PathBuf>>(&file_list_str)
        .expect("failed to parse plugin global API script paths");

      let mut plugins = Vec::new();
      for path in file_list {
        plugins.push(std::fs::read_to_string(&path).unwrap_or_else(|e| {
          panic!(
            "failed to read plugin global API script {}: {e}",
            path.display()
          )
        }));
      }

      Some(plugins)
    } else {
      None
    };

  let plugin_global_api_script = if let Some(scripts) = plugin_global_api_script {
    let scripts = scripts.into_iter().map(|s| quote!(#s));
    quote!(::std::option::Option::Some(&[#(#scripts),*]))
  } else {
    quote!(::std::option::Option::None)
  };

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
      #runtime_authority,
      #plugin_global_api_script
    );
    #with_tray_icon_code
    context
  }))
}

pub fn icon_image_codegen(path: &Path) -> EmbeddedAssetsResult<TokenStream> {
  let out_dir = make_output_directory()?;
  let mut segments = Punctuated::new();
  segments.push(PathSegment {
    ident: Ident::new("tauri", Span::call_site()),
    arguments: PathArguments::None,
  });
  let root = syn::Path {
    leading_colon: Some(Token![::](Span::call_site())),
    segments,
  };
  image_icon(&root.to_token_stream(), &out_dir, path)
}

fn make_output_directory() -> EmbeddedAssetsResult<PathBuf> {
  let out_dir = std::env::var("OUT_DIR")
    .map_err(|_| EmbeddedAssetsError::OutDir)
    .map(PathBuf::from)
    .and_then(|p| p.canonicalize().map_err(|_| EmbeddedAssetsError::OutDir))?;

  // make sure that our output directory is created
  std::fs::create_dir_all(&out_dir).map_err(|_| EmbeddedAssetsError::OutDir)?;
  Ok(out_dir)
}

fn image_icon(
  root: &TokenStream,
  out_dir: &Path,
  path: &Path,
) -> EmbeddedAssetsResult<TokenStream> {
  let extension = path.extension().unwrap_or_default();
  if extension == "ico" {
    ico_icon(root, out_dir, path)
  } else if extension == "png" {
    png_icon(root, out_dir, path)
  } else {
    Err(EmbeddedAssetsError::InvalidImageExtension {
      extension: extension.into(),
      path: path.to_path_buf(),
    })
  }
}

fn raw_icon(out_dir: &Path, path: &Path) -> EmbeddedAssetsResult<TokenStream> {
  let bytes =
    std::fs::read(path).unwrap_or_else(|e| panic!("failed to read icon {}: {}", path.display(), e));

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

fn ico_icon(root: &TokenStream, out_dir: &Path, path: &Path) -> EmbeddedAssetsResult<TokenStream> {
  let file = std::fs::File::open(path)
    .unwrap_or_else(|e| panic!("failed to open icon {}: {}", path.display(), e));
  let icon_dir = ico::IconDir::read(file)
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
  let icon = quote!(#root::image::Image::new(include_bytes!(concat!(std::env!("OUT_DIR"), "/", #icon_file_name)), #width, #height));
  Ok(icon)
}

fn png_icon(root: &TokenStream, out_dir: &Path, path: &Path) -> EmbeddedAssetsResult<TokenStream> {
  let file = std::fs::File::open(path)
    .unwrap_or_else(|e| panic!("failed to open icon {}: {}", path.display(), e));
  let decoder = png::Decoder::new(file);
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
  let icon = quote!(#root::image::Image::new(include_bytes!(concat!(std::env!("OUT_DIR"), "/", #icon_file_name)), #width, #height));
  Ok(icon)
}

fn write_if_changed(out_path: &Path, data: &[u8]) -> std::io::Result<()> {
  if let Ok(curr) = std::fs::read(out_path) {
    if curr == data {
      return Ok(());
    }
  }
  std::fs::write(out_path, data)
}

fn find_icon(
  config: &Config,
  config_parent: &Path,
  predicate: impl Fn(&&String) -> bool,
  default: &str,
) -> PathBuf {
  let icon_path = config
    .bundle
    .icon
    .iter()
    .find(predicate)
    .map(AsRef::as_ref)
    .unwrap_or(default);
  config_parent.join(icon_path)
}
