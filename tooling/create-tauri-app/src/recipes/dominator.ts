// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
// @ts-expect-error
import scaffe from 'scaffe'
import { Recipe } from '../types/recipe'

export const dominator: Recipe = {
  shortName: 'dominator',
  descriptiveName: {
    name: 'Dominator (https://crates.io/crates/dominator/)',
    value: 'Dominator'
  },
  configUpdate: ({ cfg, pm }) => ({
    ...cfg,
    distDir: `../dist`,
    devPath: 'http://localhost:10001/',
    beforeDevCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} start`,
    beforeBuildCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} build`
  }),
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
