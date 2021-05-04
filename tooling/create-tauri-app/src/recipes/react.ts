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

const reactjs: Recipe = {
  descriptiveName: 'React.js',
  shortName: 'reactjs',
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
  preInit: async ({ cwd, cfg, packageManager }) => {
    // CRA creates the folder for you
    if (packageManager === 'yarn') {
      await shell('yarn', ['create', 'react-app', `${cfg.appName}`], {
        cwd
      })
    } else {
      await shell('npx', ['create-react-app', `${cfg.appName}`, '--use-npm'], {
        cwd
      })
    }
    await afterCra(cwd, cfg.appName)
  },
  postInit: async ({ packageManager }) => {
    console.log(`
    Your installation completed.
    To start, run ${packageManager === 'yarn' ? 'yarn' : 'npm run'} tauri dev
  `)
    return await Promise.resolve()
  }
}

const reactts: Recipe = {
  ...reactjs,
  descriptiveName: 'React with Typescript',
  shortName: 'reactts',
  extraNpmDependencies: [],
  preInit: async ({ cwd, cfg, packageManager }) => {
    // CRA creates the folder for you
    if (packageManager === 'yarn') {
      await shell(
        'yarn',
        ['create', 'react-app', '--template', 'typescript', `${cfg.appName}`],
        {
          cwd
        }
      )
    } else {
      await shell(
        'npx',
        [
          'create-react-app',
          `${cfg.appName}`,
          '--use-npm',
          '--template',
          'typescript'
        ],
        {
          cwd
        }
      )
    }
    await afterCra(cwd, cfg.appName, true)
  },
  postInit: async ({ packageManager }) => {
    console.log(`
    Your installation completed.
    To start, run ${packageManager === 'yarn' ? 'yarn' : 'npm run'} tauri dev
  `)
    return await Promise.resolve()
  }
}

export { reactjs, reactts }
