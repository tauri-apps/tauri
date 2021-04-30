// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
// @ts-expect-error
import scaffe from 'scaffe'
import { Recipe } from '../types/recipe'

export const vanillajs: Recipe = {
  descriptiveName: 'Vanilla.js',
  shortName: 'vanillajs',
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../dist`,
    devPath: `../dist`,
    beforeDevCommand: `${packageManager === 'yarn' ? 'yarn' : 'npm run'} start`,
    beforeBuildCommand: `${
      packageManager === 'yarn' ? 'yarn' : 'npm run'
    } build`
  }),
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  preInit: async ({ cwd, cfg }) => {
    const { appName } = cfg
    const templateDir = join(__dirname, '../src/templates/vanilla')
    const variables = {
      name: appName
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
    const setApp =
      packageManager === 'npm'
        ? `
set tauri script once
  $ npm set-script tauri tauri
    `
        : ''

    console.log(`
change directory:
  $ cd ${cfg.appName}
${setApp}
install dependencies:
  $ ${packageManager} install

run the app:
  $ ${packageManager === 'yarn' ? 'yarn' : 'npm run'} tauri ${
      packageManager === 'npm' ? '-- ' : ''
    }dev
            `)
    return await Promise.resolve()
  }
}
