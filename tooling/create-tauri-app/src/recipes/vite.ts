// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// import { join } from 'path'
/// import scaffe from 'scaffe'
import { Recipe } from '../types/recipe'

const vite: Recipe = {
  shortName: 'vite',
  descriptiveName: {
    name: 'create-vite (vanilla, vue, react, svelte, preact, lit) (https://vitejs.dev/guide/#scaffolding-your-first-vite-project)',
    value: 'create-vite'
  },
  configUpdate: ({ cfg, pm }) => ({
    ...cfg,
    distDir: `../dist`,
    devPath: 'http://localhost:3000',
    beforeDevCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} dev`,
    beforeBuildCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} build`
  }),
  extraQuestions: ({ ci }) => [
    {
      type: 'list',
      name: 'template',
      message: 'Which vite template would you like to use?',
      choices: [
        'vanilla',
        'vanilla-ts',
        'vue',
        'vue-ts',
        'react',
        'react-ts',
        'preact',
        'preact-ts',
        'lit-element',
        'lit-element-ts',
        'svelte',
        'svelte-ts'
      ],
      default: 'vue',
      loop: false,
      when: !ci
    }
  ],
  preInit: async ({ cwd, cfg, pm, answers }) => {
    const template = (answers?.template as string) ?? 'vue'

    await pm.create('vite', [`${cfg.appName}`, '--template', `${template}`], {
      cwd
    })
  },
  postInit: async ({ pm, cfg }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${pm.name === 'npm' ? 'npm run' : pm.name} tauri dev
    `)
    return await Promise.resolve()
  }
}

export { vite }
