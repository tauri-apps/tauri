// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import execa from 'execa'
import fixtures from 'fixturez'
const f = fixtures(__dirname)
import path from 'path'
import fs from 'fs'

const ctaBinary = path.resolve('./bin/create-tauri-app.js')
const clijs = path.resolve('../cli.js/')
const api = path.resolve('../api/')

const packageFile = (template: string): object => {
  const packageFilesReturn: { [k: string]: object } = {
    vanillajs: {
      name: 'tauri-app',
      scripts: {
        tauri: 'tauri'
      }
    },
    reactjs: {
      name: 'tauri-app',
      scripts: {
        start: 'react-scripts start',
        build: 'react-scripts build',
        test: 'react-scripts test',
        eject: 'react-scripts eject',
        tauri: 'tauri'
      }
    },
    reactts: {
      name: 'tauri-app',
      scripts: {
        start: 'react-scripts start',
        build: 'react-scripts build',
        test: 'react-scripts test',
        eject: 'react-scripts eject',
        tauri: 'tauri'
      }
    }
  }

  return packageFilesReturn[template] ?? packageFilesReturn['vanillajs']
}

const manager = process.env.TAURI_RUN_MANAGER ?? 'npm'
const recipes = process.env.TAURI_RECIPE
  ? [process.env.TAURI_RECIPE]
  : ['vanillajs', 'reactjs', 'reactts', 'vite', 'vuecli']
const timeout3m = 180000
const timeout3plusm = 200000
const logOut = false ? 'inherit' : 'pipe'

beforeAll(async () => {
  const installCLI = await execa('yarn', [], {
    stdio: logOut,
    cwd: clijs,
    timeout: timeout3m
  })

  const buildCLI = await execa('yarn', ['build-release'], {
    stdio: logOut,
    cwd: clijs,
    timeout: timeout3m
  })

  const linkCLI = await execa('yarn', ['link'], {
    stdio: logOut,
    cwd: clijs,
    timeout: timeout3m
  })

  const installAPI = await execa('yarn', [], {
    stdio: logOut,
    cwd: api,
    timeout: timeout3m
  })

  const buildAPI = await execa('yarn', ['build'], {
    stdio: logOut,
    cwd: api,
    timeout: timeout3m
  })

  const linkAPI = await execa('yarn', ['link'], {
    stdio: logOut,
    cwd: api,
    timeout: timeout3m
  })
}, timeout3plusm)

describe('CTA', () => {
  describe.each(recipes.map((recipe) => [recipe, packageFile(recipe)]))(
    `%s recipe`,
    (recipe: any, packageFile: any) => {
      it(
        'runs',
        async () => {
          const folder = f.temp()

          const child = await execa(
            'node',
            [
              ctaBinary,
              '--manager',
              manager,
              '--recipe',
              recipe,
              '--ci',
              '--dev'
            ],
            {
              all: true,
              stdio: logOut,
              cwd: folder,
              timeout: timeout3m
            }
          )

          const packageFileOutput: {
            [k: string]: string | object
          } = JSON.parse(
            await fs.promises.readFile(
              path.join(folder, 'tauri-app', 'package.json'),
              'utf-8'
            )
          )
          expect(packageFileOutput['name']).toBe(packageFile['name']!)
          expect(packageFileOutput['scripts']).toMatchObject(
            packageFile['scripts']!
          )
        },
        timeout3plusm
      )
    }
  )
})
