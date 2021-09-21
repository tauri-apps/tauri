// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const solid: Recipe = {
  descriptiveName: {
    name: 'Solid (https://github.com/solidjs/templates)',
    value: 'solid'
  },
  shortName: 'solid',
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  extraQuestions: ({ ci }) => {
    return [
      {
        type: 'list',
        name: 'template',
        message: 'Which Solid template would you like to use?',
        choices: [
          'js',
          'ts-bootstrap',
          'ts-minimal',
          'ts-router',
          'ts-windicss',
          'ts'
        ],
        default: 'ts',
        loop: false,
        when: !ci
      }
    ]
  },
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../public`,
    devPath: 'http://localhost:3000',
    beforeDevCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } dev`,
    beforeBuildCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } build`
  }),
  preInit: async ({ cwd, cfg, answers }) => {
    await shell(
      'npx',
      ['degit', `solidjs/templates/${answers?.template ?? 'js'}`, cfg.appName],
      { cwd }
    )
  },
  postInit: async ({ cfg, packageManager }) => {
    console.log(`
    Your installation completed.
    $ cd ${cfg.appName}
    $ ${packageManager} install
    $ ${packageManager === 'npm' ? 'npm run' : packageManager} tauri dev
    `)

    return await Promise.resolve()
  }
}

export { solid }