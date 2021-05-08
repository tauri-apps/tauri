// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
// @ts-expect-error
import scaffe from 'scaffe'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const afterCra = async (
  cwd: string,
  appName: string,
  typescript: boolean = false
): Promise<void> => {
  const templateDir = join(
    __dirname,
    `../src/templates/react/${typescript ? 'react-ts' : 'react'}`
  )

  try {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
    await scaffe.generate(templateDir, join(cwd, appName), {
      overwrite: true
    })
  } catch (err) {
    console.log(err)
  }
}

export const cra: Recipe = {
  descriptiveName: {
    name: 'create-react-app (https://create-react-app.dev/)',
    value: 'create-react-app'
  },
  shortName: 'cra',
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../build`,
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
          { name: 'create-react-app (JavaScript)', value: 'cra.js' },
          { name: 'create-react-app (Typescript)', value: 'cra.ts' }
        ],
        default: 'cra.js',
        loop: false,
        when: !ci
      }
    ]
  },
  preInit: async ({ cwd, cfg, packageManager, answers }) => {
    let template = 'cra.js'
    if (answers) {
      template = answers.template ? (answers.template as string) : 'vue'
    }
    // CRA creates the folder for you
    if (packageManager === 'yarn') {
      await shell(
        'yarn',
        [
          'create',
          'react-app',
          ...(template === 'cra.ts' ? ['--template', 'typescript'] : []),
          `${cfg.appName}`
        ],
        {
          cwd
        }
      )
    } else {
      await shell(
        'npx',
        [
          'create-react-app',
          ...(template === 'cra.ts' ? ['--template', 'typescript'] : []),
          `${cfg.appName}`,
          '--use-npm'
        ],
        {
          cwd
        }
      )
    }
    await afterCra(cwd, cfg.appName, template === 'cra.ts')
  },
  postInit: async ({ packageManager }) => {
    console.log(`
    Your installation completed.
    To start, run ${packageManager === 'yarn' ? 'yarn' : 'npm run'} tauri dev
  `)
    return await Promise.resolve()
  }
}
