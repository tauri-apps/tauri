// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'
import { unlinkSync, existsSync } from 'fs'
import { emptyDir } from '../helpers/empty-dir'

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
  preInit: async ({ cwd, cfg, packageManager, ci }) => {
    const npmLock = join(cwd, cfg.appName, 'package-lock.json')
    const yarnLock = join(cwd, cfg.appName, 'yarn.lock')
    const nodeModules = join(cwd, cfg.appName, 'node_modules')

    switch (packageManager) {
      case 'yarn':
        await shell('yarn', ['create', 'cljs-app', `${cfg.appName}`], {
          cwd
        })

        // `create-cljs-app` has both a `package-lock.json` and a `yarn.lock`
        // so it is better to remove conflicting lock files and install fresh node_modules
        if (existsSync(npmLock)) unlinkSync(npmLock)
        emptyDir(nodeModules)

        await shell('yarn', ['install'], { cwd: join(cwd, cfg.appName) })
        break

      case 'npm':
        await shell(
          'npx',
          [ci ? '--yes' : '', 'create-cljs-app@latest', `${cfg.appName}`],
          {
            cwd
          }
        )

        if (existsSync(yarnLock)) unlinkSync(yarnLock)
        emptyDir(nodeModules)

        await shell('npm', ['install'], { cwd: join(cwd, cfg.appName) })
        break

      case 'pnpm':
        await shell(
          'npx',
          [ci ? '--yes' : '', 'create-cljs-app@latest', `${cfg.appName}`],
          {
            cwd
          }
        )

        if (existsSync(npmLock)) unlinkSync(npmLock)
        emptyDir(nodeModules)

        await shell('pnpm', ['install'], { cwd: join(cwd, cfg.appName) })
        break
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
