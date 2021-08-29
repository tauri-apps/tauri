// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
// @ts-expect-error
import scaffe from 'scaffe'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'
import { rmSync, existsSync } from 'fs'

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
    beforeDevCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } start`,
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
        message: 'Which create-react-app template would you like to use?',
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
  preInit: async ({ cwd, cfg, packageManager, answers, ci }) => {
    let template = 'cra.js'
    if (answers) {
      template = answers.template ? (answers.template as string) : 'vue'
    }
    // CRA creates the folder for you
    if (packageManager === 'yarn') {
      await shell(
        'yarn',
        [
          ci ? '--non-interactive' : '',
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
          ci ? '--yes' : '',
          'create-react-app@latest',
          ...(template === 'cra.ts' ? ['--template', 'typescript'] : []),
          `${cfg.appName}`,
          '--use-npm'
        ],
        {
          cwd
        }
      )
    }

    // create-react-app doesn't support pnpm, so we remove `node_modules` and any lock files then install them again using pnpm
    if (packageManager === 'pnpm') {
      const npmLock = join(cwd, cfg.appName, 'package-lock.json')
      const yarnLock = join(cwd, cfg.appName, 'yarn.lock')
      const nodeModules = join(cwd, cfg.appName, 'node_modules')
      if (existsSync(npmLock)) rmSync(npmLock)
      if (existsSync(yarnLock)) rmSync(yarnLock)
      if (existsSync(nodeModules))
        rmSync(nodeModules, {
          recursive: true,
          force: true
        })
      await shell('pnpm', ['install'], { cwd })
    }

    await afterCra(cwd, cfg.appName, template === 'cra.ts')
  },
  postInit: async ({ packageManager, cfg }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${packageManager === 'npm' ? 'npm run' : packageManager} tauri ${
      packageManager === 'npm' ? '--' : ''
    }dev
    `)
    return await Promise.resolve()
  }
}
