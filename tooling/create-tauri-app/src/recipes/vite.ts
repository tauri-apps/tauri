// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
// @ts-expect-error
import scaffe from 'scaffe'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const afterViteCA = async (
  cwd: string,
  appName: string,
  template: string
): Promise<void> => {
  // template dir temp removed, will eventually add it back for APIs
  // leaving this here until then

  // const templateDir = join(__dirname, `../src/templates/vite/${template}`)

  // try {
  //   await scaffe.generate(templateDir, join(cwd, appName), {
  //     overwrite: true
  //   })
  // } catch (err) {
  //   console.log(err)
  // }
}

const vite: Recipe = {
  descriptiveName: 'Vite backed recipe',
  shortName: 'vite',
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../dist`,
    devPath: 'http://localhost:3000',
    beforeDevCommand: `${packageManager === 'yarn' ? 'yarn' : 'npm run'} start`,
    beforeBuildCommand: `${
      packageManager === 'yarn' ? 'yarn' : 'npm run'
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
  preInit: async ({ cwd, cfg, packageManager, answers }) => {
    let template = 'vue'
    if (answers) {
      template = answers.template ? (answers.template as string) : 'vue'
    }

    // Vite creates the folder for you
    if (packageManager === 'yarn') {
      await shell(
        'yarn',
        [
          'create',
          '@vitejs/app',
          `${cfg.appName}`,
          '--template',
          `${template}`
        ],
        {
          cwd
        }
      )
      await shell('yarn', ['install'], { cwd })
    } else {
      await shell(
        'npx',
        ['@vitejs/create-app', `${cfg.appName}`, '--template', `${template}`],
        {
          cwd
        }
      )
      await shell('npm', ['install'], { cwd })
    }

    await afterViteCA(cwd, cfg.appName, template)
  },
  postInit: async ({ packageManager }) => {
    console.log(`
    Your installation completed.
    To start, run ${packageManager === 'yarn' ? 'yarn' : 'npm run'} tauri ${
      packageManager === 'npm' ? '--' : ''
    } dev
  `)
    return await Promise.resolve()
  }
}

export { vite }
