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
        type: 'confirm',
        name: 'typescript',
        message: 'Enable Typescript?',
        default: true,
        loop: false,
        when: !ci
      }
    ]
  },
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../public`,
    devPath: 'http://localhost:5000',
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
        ['degit', `solidjs/template/${!!answers.typescript ? 'ts' : 'js'}`, `${cfg.appName}`],
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