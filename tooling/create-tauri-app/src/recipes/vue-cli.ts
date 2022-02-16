// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

export const vuecli: Recipe = {
  shortName: 'vuecli',
  descriptiveName: {
    name: 'Vue CLI (https://cli.vuejs.org/)',
    value: 'vue-cli'
  },
  configUpdate: ({ cfg }) => cfg,
  preInit: async ({ cwd, cfg, ci, pm }) => {
    await shell(
      'npx',
      [
        ci ? '--yes' : '',
        '@vue/cli@latest',
        'create',
        `${cfg.appName}`,
        '--pm',
        pm.name,
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
  postInit: async ({ cfg, pm }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${pm.name === 'npm' ? 'npm run' : pm.name} tauri:serve
    `)
    return await Promise.resolve()
  }
}
