// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
import { Recipe } from '../types/recipe'
import { unlinkSync, existsSync } from 'fs'
import { emptyDir } from '../helpers/empty-dir'
import { updatePackageJson } from '../helpers/update-package-json'

export const cra: Recipe = {
  shortName: 'cra',
  descriptiveName: {
    name: 'create-react-app (https://create-react-app.dev/)',
    value: 'create-react-app'
  },
  configUpdate: ({ cfg, pm }) => ({
    ...cfg,
    distDir: `../build`,
    devPath: 'http://localhost:3000',
    beforeDevCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} start`,
    beforeBuildCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} build`
  }),
  extraNpmDevDependencies: ['cross-env'],
  extraQuestions: ({ ci }) => [
    {
      type: 'list',
      name: 'template',
      message: 'Which create-react-app template would you like to use?',
      choices: [
        { name: 'create-react-app (JavaScript)', value: 'cra.js' },
        { name: 'create-react-app (Typescript)', value: 'cra.ts' }
      ],
      default: 'cra.js',
      loop: false,
      when: !ci
    }
  ],
  preInit: async ({ cwd, cfg, pm, answers }) => {
    const template = (answers?.template as string) ?? 'cra.js'
    await pm.create(
      'react-app',
      [
        ...(template === 'cra.ts' ? ['--template', 'typescript'] : []),
        `${cfg.appName}`,
        pm.name !== 'yarn' ? '--use-npm' : ''
      ],
      {
        cwd
      }
    )

    // create-react-app doesn't support pnpm, so we remove `node_modules` and any lock files then install them again using pnpm
    if (pm.name === 'pnpm') {
      const npmLock = join(cwd, cfg.appName, 'package-lock.json')
      const yarnLock = join(cwd, cfg.appName, 'yarn.lock')
      const nodeModules = join(cwd, cfg.appName, 'node_modules')
      if (existsSync(npmLock)) unlinkSync(npmLock)
      if (existsSync(yarnLock)) unlinkSync(yarnLock)
      emptyDir(nodeModules)
      await pm.install({ cwd: join(cwd, cfg.appName) })
    }

    updatePackageJson((pkg) => {
      return {
        ...pkg,
        scripts: {
          ...pkg.scripts,
          start: `${'cross-env BROWSER=none '}${pkg.scripts?.start as string}`
        }
      }
    }, join(cwd, cfg.appName))
  },
  postInit: async ({ pm, cfg }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${pm.name === 'npm' ? 'npm run' : pm.name} tauri dev
    `)
    return await Promise.resolve()
  }
}
