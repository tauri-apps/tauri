// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const completeLogMsg = `
  Your installation completed.
  To start, run \`yarn dev\` and \`yarn tauri dev\`
`

const svelte: Recipe = {
  descriptiveName: {
    name: 'Svelte (https://svelte.dev/)',
    value: 'svelte'
  },
  shortName: 'svelte',
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  extraQuestions: () => {
    return [
      {
        type: 'confirm',
        name: 'typescript',
        message: 'Enable Typescript?',
        default: true,
        loop: false
      }
    ]
  },
  configUpdate: ({ cfg }) => ({
    ...cfg,
    distDir: `../public`,
    devPath: 'http://localhost:5000',
  }),
  preInit: async ({ cwd, cfg, answers }) => {
    let typescript = false
    if (answers) {
      typescript = !!answers.typescript
    }

    await shell(
      'npx',
      [
        'degit',
        'sveltejs/template',
        `${cfg.appName}`,
      ],
      { cwd }
    )

    // Add Typescript
    if (typescript) {
      await shell(
        'node',
        [
          'scripts/setupTypeScript.js',
        ],
        { cwd: join(cwd, cfg.appName) }
      )
    }

  },
  postInit: async () => {
    console.log(completeLogMsg)
    return await Promise.resolve()
  }
}

export { svelte }
