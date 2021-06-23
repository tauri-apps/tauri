# Changelog

## \[1.0.0-beta.2]

- Fixes the `beforeDevCommand` on vite recipe.
  - [3c21ddc7](https://www.github.com/tauri-apps/tauri/commit/3c21ddc73cd7ab8141b730ceade46fc2dfadd996) fix(cta): use correct `beforeDevCommand` for vite recipe ([#1931](https://www.github.com/tauri-apps/tauri/pull/1931)) on 2021-06-01

## \[1.0.0-beta.1]

- Work around bugs between esbuild and npm by installing directly at the end of the sequence. Also default to using the latest on all of the installs instead of npx's cache.
  - [8a164d0](https://www.github.com/tauri-apps/tauri/commit/8a164d0a1f8eb69bdcec7ae4362d26b2f3c7ff55) fix: CTA cache and vite build ([#1806](https://www.github.com/tauri-apps/tauri/pull/1806)) on 2021-05-12

## \[1.0.0-beta.0]

- Explicitly install deps after a vite recipe.
  - [397b7af](https://www.github.com/tauri-apps/tauri/commit/397b7af395a213bf826aa52398467b7b3352b666) chore: CTA defaults in CI mode ([#1671](https://www.github.com/tauri-apps/tauri/pull/1671)) on 2021-05-05
- Shift everything out of the `bin` and into `.ts` so we can apply Typescript types.
  - [c3acbd6](https://www.github.com/tauri-apps/tauri/commit/c3acbd68ec169188c782cbaf7d100d80b3a4f39a) chore: shift CTA from bin to .ts ([#1651](https://www.github.com/tauri-apps/tauri/pull/1651)) on 2021-04-29
- We setup an e2e type test suite for CTA. It is mostly an internal change, but should help with stability moving forward.
  - [af6411d](https://www.github.com/tauri-apps/tauri/commit/af6411d5f8c9fd1c3d9b4f3c2d79e8f1bd0efbf2) feat: setup testing for CTA ([#1615](https://www.github.com/tauri-apps/tauri/pull/1615)) on 2021-04-27
- Add support for all vite templates
  - [cea3ba9](https://www.github.com/tauri-apps/tauri/commit/cea3ba9f97de9d0181a84ad085a852517bd33a65) feat(cta): add support for all vite templates ([#1670](https://www.github.com/tauri-apps/tauri/pull/1670)) on 2021-05-07
- Add a welcome prompt to let the user know about the process and links to more info including prerequisite setup steps. Also add links to each of the templates to give the user more context what they are getting into.
  - [ea28d01](https://www.github.com/tauri-apps/tauri/commit/ea28d0169168953e11416231e50b08061413a27e) create-tauri-app welcome prompt and recipes links ([#1748](https://www.github.com/tauri-apps/tauri/pull/1748)) on 2021-05-09

## \[1.0.0-beta-rc.4]

- Manually set `tauri` script instead of using `npm set-script` for compatabilty with older npm versions
  - [f708ff8](https://www.github.com/tauri-apps/tauri/commit/f708ff824e7933341536aecb49f6ee35eea621da) fix(CTA): [#1569](https://www.github.com/tauri-apps/tauri/pull/1569), manually set tauri script for compatability with older npm ([#1572](https://www.github.com/tauri-apps/tauri/pull/1572)) on 2021-04-22

## \[1.0.0-beta-rc.3]

- Remove `lodash` dependency and replace with es6 builtins
  - [edab7a6](https://www.github.com/tauri-apps/tauri/commit/edab7a66864d21b51694bf8771d21627b526c2b9) chore(deps): remove lodash from create-tauri-app ([#1532](https://www.github.com/tauri-apps/tauri/pull/1532)) on 2021-04-18
- Remove `tauri` dependency from vanilla recipe
  - [3998046](https://www.github.com/tauri-apps/tauri/commit/399804648924139c6240351a76812a3071b51f65) fix(cta): remove `tauri` dep from vanilla recipe ([#1502](https://www.github.com/tauri-apps/tauri/pull/1502)) on 2021-04-15
- Fix adding `tauri` script to package.json
  - [6c00e88](https://www.github.com/tauri-apps/tauri/commit/6c00e88e0ffa10eb7eecc312d66c5dde7dc03d0b) fix(cta): fix adding `tauri` script to package.json ([#1501](https://www.github.com/tauri-apps/tauri/pull/1501)) on 2021-04-15
  - [345f2db](https://www.github.com/tauri-apps/tauri/commit/345f2dbfc545427750c08351d1b98e966b2436c0) Apply Version Updates From Current Changes ([#1499](https://www.github.com/tauri-apps/tauri/pull/1499)) on 2021-04-14
  - [098b729](https://www.github.com/tauri-apps/tauri/commit/098b729e677dc5dc322f22a6cbd5a652a8dfa1b0) chore: CTA version was decremented, fix and adjust changelog to compensate ([#1530](https://www.github.com/tauri-apps/tauri/pull/1530)) on 2021-04-18

## \[1.0.0-beta-rc.2]

- CTA also needs the template directory published as it doesn't get bundled into the `dist` directory.
  - [7b6108e](https://www.github.com/tauri-apps/tauri/commit/7b6108e37be652a1efa4018fc1908aa0a2cdacd6) fix: cta templates dir missing ([#1496](https://www.github.com/tauri-apps/tauri/pull/1496)) on 2021-04-14

## \[1.0.0-beta-rc.1]

- CTA was missing the `files` property in the package.json which mean that the `dist` directory was not published and used.
  - [414f9a7](https://www.github.com/tauri-apps/tauri/commit/414f9a78c9b636933fd741d1b6fe7f097f496fc9) fix: cta dist publish ([#1493](https://www.github.com/tauri-apps/tauri/pull/1493)) on 2021-04-14

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
