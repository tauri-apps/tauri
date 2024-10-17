// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::convert::identity;
use std::path::{Path, PathBuf};
use std::{ffi::OsStr, str::FromStr};

use crate::{
  embedded_assets::{
    ensure_out_dir, AssetOptions, CspHashes, EmbeddedAssets, EmbeddedAssetsResult,
  },
  image::CachedIcon,
};
use base64::Engine;
use proc_macro2::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};
use syn::Expr;
use tauri_utils::acl::{ACL_MANIFESTS_FILE_NAME, CAPABILITIES_FILE_NAME};
use tauri_utils::{
  acl::capability::{Capability, CapabilityFile},
  acl::manifest::Manifest,
  acl::resolved::Resolved,
  assets::AssetKey,
  config::{CapabilityEntry, Config, FrontendDist, PatternKind},
  html::{inject_nonce_token, parse as parse_html, serialize_node as serialize_html_node, NodeRef},
  platform::Target,
  tokens::{map_lit, str_lit},
};

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
  /// Skip runtime-only types generation for tests (e.g. embed-plist usage).
  pub test: bool,
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
    test,
  } = data;

  #[allow(unused_variables)]
  let running_tests = test;

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
              "The `frontendDist` configuration is set to `{path:?}` but this path doesn't exist"
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

  let out_dir = ensure_out_dir()?;

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
        let icon = CachedIcon::new(&root, &icon_path)?;
        quote!(::std::option::Option::Some(#icon))
      } else {
        let icon_path = find_icon(
          &config,
          &config_parent,
          |i| i.ends_with(".png"),
          "icons/icon.png",
        );
        let icon = CachedIcon::new(&root, &icon_path)?;
        quote!(::std::option::Option::Some(#icon))
      }
    } else {
      // handle default window icons for Unix targets
      let icon_path = find_icon(
        &config,
        &config_parent,
        |i| i.ends_with(".png"),
        "icons/icon.png",
      );
      let icon = CachedIcon::new(&root, &icon_path)?;
      quote!(::std::option::Option::Some(#icon))
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

    let icon = CachedIcon::new_raw(&root, &icon_path)?;
    quote!(::std::option::Option::Some(#icon.to_vec()))
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
      let icon = CachedIcon::new(&root, &tray_icon_icon_path)?;
      quote!(context.set_tray_icon(::std::option::Option::Some(#icon));)
    } else {
      quote!()
    }
  } else {
    quote!()
  };

  #[cfg(target_os = "macos")]
  let maybe_embed_plist_block = if target == Target::MacOS && dev && !running_tests {
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
        plist.insert("CFBundleVersion".into(), version.clone().into());
      }
    }

    let mut plist_contents = std::io::BufWriter::new(Vec::new());
    info_plist
      .to_writer_xml(&mut plist_contents)
      .expect("failed to serialize plist");
    let plist_contents =
      String::from_utf8_lossy(&plist_contents.into_inner().unwrap()).into_owned();

    let plist = crate::Cached::try_from(plist_contents)?;
    quote!({
      tauri::embed_plist::embed_info_plist!(#plist);
    })
  } else {
    quote!()
  };
  #[cfg(not(target_os = "macos"))]
  let maybe_embed_plist_block = quote!();

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

  let plugin_global_api_scripts = if config.app.with_global_tauri {
    if let Some(scripts) = tauri_utils::plugin::read_global_api_scripts(&out_dir) {
      let scripts = scripts.into_iter().map(|s| quote!(#s));
      quote!(::std::option::Option::Some(&[#(#scripts),*]))
    } else {
      quote!(::std::option::Option::None)
    }
  } else {
    quote!(::std::option::Option::None)
  };

  let maybe_config_parent_setter = if dev {
    let config_parent = config_parent.to_string_lossy();
    quote!({
      context.with_config_parent(#config_parent);
    })
  } else {
    quote!()
  };

  let context = quote!({
    #maybe_embed_plist_block

    #[allow(unused_mut, clippy::let_and_return)]
    let mut context = #root::Context::new(
      #config,
      ::std::boxed::Box::new(#assets),
      #default_window_icon,
      #app_icon,
      #package_info,
      #pattern,
      #runtime_authority,
      #plugin_global_api_scripts
    );

    #with_tray_icon_code
    #maybe_config_parent_setter

    context
  });

  Ok(quote!({
    let thread = ::std::thread::Builder::new()
      .name(String::from("generated tauri context creation"))
      .stack_size(8 * 1024 * 1024)
      .spawn(|| #context)
      .expect("unable to create thread with 8MiB stack");

    match thread.join() {
      Ok(context) => context,
      Err(_) => {
        eprintln!("the generated Tauri `Context` panicked during creation");
        ::std::process::exit(101);
      }
    }
  }))
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
