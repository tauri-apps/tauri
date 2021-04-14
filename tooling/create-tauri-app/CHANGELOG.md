# Changelog

## \[1.0.0]

- internal refactoring of `Params` to allow for easier usage without a private trait with only 1 implementor.

`ParamsPrivate` -> `ParamsBase`
`ManagerPrivate` -> `ManagerBase`
(new) `Args`, crate only. Now implements `Params`/`ParamsBase`.
`App` and `Window` use `WindowManager` directly

- Bumped due to a bump in cli.js.
- [ec27ca8](https://www.github.com/tauri-apps/tauri/commit/ec27ca81fe721e0b08ed574073547250b7a8153a) refactor(tauri): remove private params trait methods ([#1484](https://www.github.com/tauri-apps/tauri/pull/1484)) on 2021-04-14
- Updated `wry`, fixing an issue with the draggable region.
  - Bumped due to a bump in cli.js.
  - [f2d24ef](https://www.github.com/tauri-apps/tauri/commit/f2d24ef2fbd95ec7d3433ba651964f4aa3b7f48c) chore(deps): update wry ([#1482](https://www.github.com/tauri-apps/tauri/pull/1482)) on 2021-04-14

## \[1.0.0-beta-rc.0]

- Add vanilla javascript option to `create-tauri-app` through templating.
  - [c580338](https://www.github.com/tauri-apps/tauri/commit/c580338f07b71551f7fd2712e13ad0acef100095) feat(cli): add create-tauri-app ([#1106](https://www.github.com/tauri-apps/tauri/pull/1106)) on 2021-03-07
- Use a test based on an npm env var to determine which package manager to use.
  - [6e0598c](https://www.github.com/tauri-apps/tauri/commit/6e0598c807ce02a3964788c06ec1025abc1fb250) feat: derive package manager from env var on 2021-04-12
- Add initial `vite` support starting with `vue` and `vue-ts`
  - [80b7bd7](https://www.github.com/tauri-apps/tauri/commit/80b7bd7de86f59e0cafaa0efdc6e82a0db7d7ba2) feat(CTA): add initial vite support with `vue` and `vue-ts` ([#1467](https://www.github.com/tauri-apps/tauri/pull/1467)) on 2021-04-13
- Revert `tauri create` deletion and shift remaining pieces that weren't deleted to `create-tauri-app`.
  - [4ec20a4](https://www.github.com/tauri-apps/tauri/commit/4ec20a4a28823614186365c5a90512d77170cff2) feat: shift tauri create \[not wired up] ([#1330](https://www.github.com/tauri-apps/tauri/pull/1330)) on 2021-03-07
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
