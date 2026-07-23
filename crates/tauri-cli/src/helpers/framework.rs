// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::fmt;

#[derive(Debug, Clone)]
pub enum Framework {
  SolidJS,
  SolidStart,
  Svelte,
  SvelteKit,
  Angular,
  React,
  Nextjs,
  Gatsby,
  Nuxt,
  Quasar,
  VueCli,
  Vue,
}

impl Framework {
  pub fn dev_url(&self) -> String {
    match self {
      Self::SolidJS => "http://localhost:3000",
      Self::SolidStart => "http://localhost:3000",
      Self::Svelte => "http://localhost:8080",
      Self::SvelteKit => "http://localhost:5173",
      Self::Angular => "http://localhost:4200",
      Self::React => "http://localhost:3000",
      Self::Nextjs => "http://localhost:3000",
      Self::Gatsby => "http://localhost:8000",
      Self::Nuxt => "http://localhost:3000",
      Self::Quasar => "http://localhost:8080",
      Self::VueCli => "http://localhost:8080",
      Self::Vue => "http://localhost:8080",
    }
    .into()
  }

  pub fn frontend_dist(&self) -> String {
    match self {
      Self::SolidJS => "../dist",
      Self::SolidStart => "../dist/public",
      Self::Svelte => "../public",
      Self::SvelteKit => "../build",
      Self::Angular => "../dist",
      Self::React => "../build",
      Self::Nextjs => "../out",
      Self::Gatsby => "../public",
      Self::Nuxt => "../dist",
      Self::Quasar => "../dist/spa",
      Self::VueCli => "../dist",
      Self::Vue => "../dist",
    }
    .into()
  }
}

impl fmt::Display for Framework {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::SolidJS => write!(f, "SolidJS"),
      Self::SolidStart => write!(f, "SolidStart"),
      Self::Svelte => write!(f, "Svelte"),
      Self::SvelteKit => write!(f, "SvelteKit"),
      Self::Angular => write!(f, "Angular"),
      Self::React => write!(f, "React"),
      Self::Nextjs => write!(f, "React (Next.js)"),
      Self::Gatsby => write!(f, "React (Gatsby)"),
      Self::Nuxt => write!(f, "Vue.js (Nuxt)"),
      Self::Quasar => write!(f, "Vue.js (Quasar)"),
      Self::VueCli => write!(f, "Vue.js (Vue CLI)"),
      Self::Vue => write!(f, "Vue.js"),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Bundler {
  Webpack,
  Rollup,
  Vite,
}

impl fmt::Display for Bundler {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Webpack => write!(f, "Webpack"),
      Self::Rollup => write!(f, "Rollup"),
      Self::Vite => write!(f, "Vite"),
    }
  }
}

pub fn infer_from_package_json(package_json: &str) -> (Option<Framework>, Option<Bundler>) {
  let framework_map = [
    ("solid-start", Framework::SolidStart, Some(Bundler::Vite)),
    ("solid-js", Framework::SolidJS, Some(Bundler::Vite)),
    ("svelte", Framework::Svelte, Some(Bundler::Rollup)),
    ("@sveltejs/kit", Framework::SvelteKit, Some(Bundler::Vite)),
    ("@angular", Framework::Angular, Some(Bundler::Webpack)),
    (r#""next""#, Framework::Nextjs, Some(Bundler::Webpack)),
    ("gatsby", Framework::Gatsby, Some(Bundler::Webpack)),
    ("react", Framework::React, None),
    ("nuxt", Framework::Nuxt, Some(Bundler::Webpack)),
    ("quasar", Framework::Quasar, Some(Bundler::Webpack)),
    ("@vue/cli", Framework::VueCli, Some(Bundler::Webpack)),
    ("vue", Framework::Vue, Some(Bundler::Vite)),
  ];
  let bundler_map = [
    ("webpack", Bundler::Webpack),
    ("rollup", Bundler::Rollup),
    ("vite", Bundler::Vite),
  ];

  let (framework, framework_bundler) = framework_map
    .iter()
    .find(|(keyword, _, _)| package_json.contains(keyword))
    .map(|(_, framework, bundler)| (Some(framework.clone()), bundler.clone()))
    .unwrap_or((None, None));

  let bundler = bundler_map
    .iter()
    .find(|(keyword, _)| package_json.contains(keyword))
    .map(|(_, bundler)| bundler.clone())
    .or(framework_bundler);

  (framework, bundler)
}
