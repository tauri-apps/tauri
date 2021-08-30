// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// import { join } from 'path'
/// import scaffe from 'scaffe'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const vite: Recipe = {
  descriptiveName: {
    name: 'create-vite (https://vitejs.dev/guide/#scaffolding-your-first-vite-project)',
    value: 'create-vite'
  },
  shortName: 'vite',
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../dist`,
    devPath: 'http://localhost:3000',
    beforeDevCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } dev`,
    beforeBuildCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } build`
  }),
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  extraQuestions: ({ ci }) => {
    return [
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
    ]
  },
  preInit: async ({ cwd, cfg, packageManager, answers, ci }) => {
    const template = answers?.template ? (answers.template as string) : 'vue'

    // Vite creates the folder for you
    if (packageManager === 'yarn') {
      await shell(
        'yarn',
        [
          ci ? '--non-interactive' : '',
          'create',
          'vite',
          `${cfg.appName}`,
          '--template',
          `${template}`
        ],
        {
          cwd
        }
      )
    } else {
      await shell(
        packageManager === 'pnpm' ? 'pnpx' : 'npx',
        [
          ci ? '--yes' : '',
          'create-vite@latest',
          `${cfg.appName}`,
          '--template',
          `${template}`
        ],
        {
          cwd
        }
      )
    }
  },
  postInit: async ({ cwd, packageManager, cfg }) => {
    // we don't have a consistent way to rebuild and
    // esbuild has hit all the bugs and struggles to install on the postinstall
    await shell('node', ['./node_modules/esbuild/install.js'], { cwd })
    if (packageManager === 'npm') {
      await shell('npm', ['run', 'build'], { cwd })
    } else {
      await shell(packageManager, ['build'], { cwd })
    }

    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${packageManager === 'npm' ? 'npm run' : packageManager} tauri dev
    `)
    return await Promise.resolve()
  }
}

export { vite }
