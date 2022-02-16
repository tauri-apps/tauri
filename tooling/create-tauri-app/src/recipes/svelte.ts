// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

export const svelte: Recipe = {
  shortName: 'svelte',
  descriptiveName: {
    name: 'Svelte (https://github.com/sveltejs/template)',
    value: 'svelte'
  },
  extraQuestions: ({ ci }) => [
    {
      type: 'confirm',
      name: 'typescript',
      message: 'Enable Typescript?',
      default: true,
      loop: false,
      when: !ci
    }
  ],
  configUpdate: ({ cfg, pm }) => ({
    ...cfg,
    distDir: `../public`,
    devPath: 'http://localhost:8080',
    beforeDevCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} dev`,
    beforeBuildCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} build`
  }),
  preInit: async ({ cwd, cfg, answers, ci }) => {
    await shell(
      'npx',
      [ci ? '--yes' : '', 'degit', 'sveltejs/template', `${cfg.appName}`],
      {
        cwd
      }
    )

    if (answers?.typescript) {
      await shell('node', ['scripts/setupTypeScript.js'], {
        cwd: join(cwd, cfg.appName)
      })
    }
  },
  postInit: async ({ cfg, pm }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${pm.name} install
    $ ${pm.name === 'npm' ? 'npm run' : pm.name} tauri dev
    `)

    return await Promise.resolve()
  }
}
