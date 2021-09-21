// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const solid: Recipe = {
  descriptiveName: {
    name: 'SolidJS (https://github.com/solidjs/solid)',
    value: 'solidjs'
  },
  shortName: 'solid',
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  extraQuestions: ({ ci }) => {
    return [
      {
        type: 'list',
        name: 'template',
        message: 'Which SolidJS template would you like to use? (Read more at https://github.com/solidjs/templates)',
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
      packageManager === 'yarn' ? 'npm run' : packageManager
    } dev`,
    beforeBuildCommand: `${
      packageManager === 'yarn' ? 'npm run' : packageManager
    } build`
  }),
  preInit: async ({ cwd, cfg, answers }) => {
    await shell(
      'npx',
      ['degit', `solidjs/templates/${answers?.template}`, `${cfg.appName}`],
      { cwd }
    )
  },
  postInit: async ({ cfg, packageManager }) => {
    console.log(`
    Your installation completed.
    $ cd ${cfg.appName}
    $ ${packageManager} install
    $ ${packageManager === 'npm' ? 'npm run' : packageManager} tauri ${
      packageManager === 'npm' ? '--' : ''
    }dev
    `)

    return await Promise.resolve()
  }
}

export { solid }