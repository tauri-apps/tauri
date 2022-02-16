// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const solid: Recipe = {
  shortName: 'solid',
  descriptiveName: {
    name: 'Solid (https://github.com/solidjs/templates)',
    value: 'solid'
  },
  extraQuestions: ({ ci }) => [
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
  ],
  configUpdate: ({ cfg, pm }) => ({
    ...cfg,
    distDir: `../public`,
    devPath: 'http://localhost:3000',
    beforeDevCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} dev`,
    beforeBuildCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} build`
  }),
  preInit: async ({ cwd, cfg, answers, ci }) => {
    await shell(
      'npx',
      [
        ci ? '--yes' : '',
        'degit',
        `solidjs/templates/${(answers?.template as string) ?? 'js'}`,
        cfg.appName
      ],
      { cwd }
    )
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

export { solid }
