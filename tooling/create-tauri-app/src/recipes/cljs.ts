// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'
import { rmSync, existsSync } from 'fs'

const cljs: Recipe = {
  descriptiveName: {
    name: 'ClojureScript (https://github.com/filipesilva/create-cljs-app)',
    value: 'cljs'
  },
  shortName: 'cljs',
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../public`,
    devPath: 'http://localhost:3000',
    beforeDevCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } start`,
    beforeBuildCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } build`
  }),
  preInit: async ({ cwd, cfg, packageManager }) => {
    // create-cljs-app creates the folder for you
    if (packageManager === 'yarn') {
      await shell('yarn', ['create', 'cljs-app', `${cfg.appName}`], {
        cwd
      })
      /* `create-cljs-app` has both a `package-lock.json` and a `yarn.lock`
         I think it's a good idea to get rid of conflicting lockfiles. */
      const npmLock = join(cwd, cfg.appName, 'package-lock.json')
      if (existsSync(npmLock)) rmSync(npmLock)
      await shell('yarn', ['install'], { cwd: join(cwd, cfg.appName) })
    } else {
      await shell('npx', ['create-cljs-app@latest', `${cfg.appName}`], {
        cwd
      })
      /* Remove Unnecessary lockfile as above. */
      const yarnLock = join(cwd, cfg.appName, 'yarn.lock')
      if (existsSync(yarnLock)) rmSync(yarnLock)
      await shell('npm', ['install'], { cwd: join(cwd, cfg.appName) })
    }
  },
  postInit: async ({ cfg, packageManager }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${packageManager === 'npm' ? 'npm run' : packageManager} tauri dev
    `)
    return await Promise.resolve()
  }
}

export { cljs }
