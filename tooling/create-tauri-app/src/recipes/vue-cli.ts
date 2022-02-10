// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const vuecli: Recipe = {
  descriptiveName: {
    name: 'Vue CLI (https://cli.vuejs.org/)',
    value: 'vue-cli'
  },
  shortName: 'vuecli',
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  configUpdate: ({ cfg }) => cfg,
  preInit: async ({ cwd, cfg, ci, packageManager }) => {
    await shell(
      'npx',
      [
        ci ? '--yes' : '',
        '@vue/cli@latest',
        'create',
        `${cfg.appName}`,
        '--packageManager',
        packageManager,
        ci ? '--default' : ''
      ],
      { cwd }
    )
    await shell(
      'npx',
      [
        ci ? '--yes' : '',
        '@vue/cli',
        'add',
        'tauri',
        '--appName',
        `${cfg.appName}`,
        '--windowTitle',
        `${cfg.windowTitle}`
      ],
      {
        cwd: join(cwd, cfg.appName)
      }
    )
  },
  postInit: async ({ cfg, packageManager }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${packageManager === 'npm' ? 'npm run' : packageManager} tauri:serve
    `)
    return await Promise.resolve()
  }
}

export { vuecli }
