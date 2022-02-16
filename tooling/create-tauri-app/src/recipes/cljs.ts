// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
import { Recipe } from '../types/recipe'
import { unlinkSync, existsSync } from 'fs'
import { emptyDir } from '../helpers/empty-dir'

export const cljs: Recipe = {
  shortName: 'cljs',
  descriptiveName: {
    name: 'ClojureScript (https://github.com/filipesilva/create-cljs-app)',
    value: 'cljs'
  },
  configUpdate: ({ cfg, pm }) => ({
    ...cfg,
    distDir: `../public`,
    devPath: 'http://localhost:3000',
    beforeDevCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} start`,
    beforeBuildCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} build`
  }),
  preInit: async ({ cwd, cfg, pm }) => {
    const npmLock = join(cwd, cfg.appName, 'package-lock.json')
    const yarnLock = join(cwd, cfg.appName, 'yarn.lock')
    const nodeModules = join(cwd, cfg.appName, 'node_modules')

    await pm.create('cljs-app', [cfg.appName], { cwd })

    // `create-cljs-app` has both a `package-lock.json` and a `yarn.lock`
    // so it is better to remove conflicting lock files and install fresh node_modules
    emptyDir(nodeModules)
    switch (pm.name) {
      case 'yarn':
        if (existsSync(npmLock)) unlinkSync(npmLock)
        break
      case 'npm':
        if (existsSync(yarnLock)) unlinkSync(yarnLock)

      case 'pnpm':
        if (existsSync(npmLock)) unlinkSync(npmLock)
        if (existsSync(yarnLock)) unlinkSync(yarnLock)
        break
      default:
        break
    }
    await pm.install({ cwd: join(cwd, cfg.appName) })
  },
  postInit: async ({ cfg, pm }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${pm.name === 'npm' ? 'npm run' : pm.name} tauri dev
    `)
    return await Promise.resolve()
  }
}
