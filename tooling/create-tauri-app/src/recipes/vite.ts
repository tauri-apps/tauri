// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// import { join } from 'path'
/// import scaffe from 'scaffe'
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
  descriptiveName: {
    name: '@vitejs/create-app (https://vitejs.dev/guide/#scaffolding-your-first-vite-project)',
    value: 'vite-create-app'
  },
  shortName: 'vite',
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../dist`,
    devPath: 'http://localhost:3000',
    beforeDevCommand: `${packageManager === 'yarn' ? 'yarn' : 'npm run'} dev`,
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
        'npx',
        [
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

    await afterViteCA(cwd, cfg.appName, template)
  },
  postInit: async ({ cwd, packageManager }) => {
    // we don't have a consistent way to rebuild and
    // esbuild has hit all the bugs and struggles to install on the postinstall
    await shell('node', ['./node_modules/esbuild/install.js'], { cwd })
    if (packageManager === 'yarn') {
      await shell('yarn', ['build'], { cwd })
    } else {
      await shell('npm', ['run', 'build'], { cwd })
    }
    console.log(`
    Your installation completed. Change directories to \`${cwd}\`.
    To start, run ${packageManager === 'yarn' ? 'yarn' : 'npm run'} tauri ${
      packageManager === 'npm' ? '--' : ''
    } dev
  `)
    return await Promise.resolve()
  }
}

export { vite }
