// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
// @ts-expect-error
import scaffe from 'scaffe'
import { Recipe } from '../types/recipe'

export const dominator: Recipe = {
  descriptiveName: {
    name: 'Dominator (https://crates.io/crates/dominator/)',
    value: 'Dominator'
  },
  shortName: 'dominator',
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../dist`,
    devPath: 'http://localhost:10001/',
    beforeDevCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } start`,
    beforeBuildCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } build`
  }),
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  preInit: async ({ cwd, cfg }) => {
    const { appName, windowTitle } = cfg
    const templateDir = join(__dirname, '../src/templates/dominator')
    const variables = {
      name: appName,
      title: windowTitle
    }

    try {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
      await scaffe.generate(templateDir, join(cwd, appName), {
        overwrite: true,
        variables
      })
    } catch (err) {
      console.log(err)
    }
  },
  postInit: async ({ cfg, packageManager }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${packageManager} install
    $ ${packageManager === 'npm' ? 'npm run' : packageManager} tauri ${
      packageManager === 'npm' ? '--' : ''
    } dev
    `)
    return await Promise.resolve()
  }
}
