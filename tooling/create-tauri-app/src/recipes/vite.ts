// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { Recipe } from '..'
import { join } from 'path'
import { readdirSync } from 'fs'
// @ts-expect-error
import scaffe from 'scaffe'
import { shell } from '../shell'
import inquirer from 'inquirer'

const afterViteCA = async (
  cwd: string,
  appName: string,
  template: string
): Promise<void> => {
  const templateDir = join(__dirname, `../src/templates/vite/${template}`)

  try {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
    await scaffe.generate(templateDir, join(cwd, appName), {
      overwrite: true
    })
  } catch (err) {
    console.log(err)
  }
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
        choices: readdirSync(join(__dirname, '../src/templates/vite')),
        default: 'vue',
        when: !ci
      }
    ]
  },
  preInit: async ({ cwd, cfg, packageManager, answers }) => {
    let template = 'vue-ts'
    if (answers) {
      template = answers.template
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
    } else {
      await shell(
        'npx',
        ['@vitejs/create-app', `${cfg.appName}`, '--template', `${template}`],
        {
          cwd
        }
      )
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
