// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// import { join } from 'path'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const cljs: Recipe = {
  descriptiveName: {
    name: 'ClojureScript (https://clojurescript.org/)',
    value: 'cljs'
  },
  shortName: 'cljs',
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  configUpdate: ({ cfg }) => ({
    ...cfg,
    distDir: `../public`,
    devPath: 'http://localhost:3000',
  }),
  preInit: async ({ cwd, cfg, ci, packageManager }) => {
    // create-cljs-app creates the folder for you
    if (packageManager === 'yarn') {
      await shell(
        'yarn',
        [
          'create',
          'cljs-app',
          `${cfg.appName}`
        ],
        {
          cwd
        }
      )
    } else {
      await shell(
        'npx',
        [
          'create-cljs-app@latest',
          `${cfg.appName}`,
        ],
        {
          cwd
        }
      )
    }
  },
  postInit: async ({ cfg, packageManager }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}

    Run Tauri and Shadow CLJS in separate terminals:

    $ ${packageManager === 'npm' ? 'npm run' : packageManager} start
    $ ${packageManager === 'npm' ? 'npm run' : packageManager} tauri dev
    `)
    return await Promise.resolve()
  }
}

export { cljs }
