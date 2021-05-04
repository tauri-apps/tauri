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

const manager = process.env.TAURI_RUN_MANAGER ?? 'npm'
const recipes = process.env.TAURI_RECIPE
  ? [process.env.TAURI_RECIPE]
  : ['vanillajs', 'reactjs', 'reactts', 'vite', 'vuecli']
const timeoutLong = 900000
const timeoutLittleLonger = 930000
const logOut = false ? 'inherit' : 'pipe'

beforeAll(async () => {
  const installCLI = await execa('yarn', [], {
    stdio: logOut,
    cwd: clijs,
    timeout: timeoutLong
  })

  const buildCLI = await execa('yarn', ['build-release'], {
    stdio: logOut,
    cwd: clijs,
    timeout: timeoutLong
  })

  const linkCLI = await execa('yarn', ['link'], {
    stdio: logOut,
    cwd: clijs,
    timeout: timeoutLong
  })

  const installAPI = await execa('yarn', [], {
    stdio: logOut,
    cwd: api,
    timeout: timeoutLong
  })

  const buildAPI = await execa('yarn', ['build'], {
    stdio: logOut,
    cwd: api,
    timeout: timeoutLong
  })

  const linkAPI = await execa('yarn', ['link'], {
    stdio: logOut,
    cwd: api,
    timeout: timeoutLong
  })
}, timeoutLittleLonger)

describe('CTA', () => {
  describe.each(recipes.map((recipe) => [recipe, 'tauri-app']))(
    `%s recipe`,
    (recipe: string, appName: string) => {
      it(
        'runs',
        async () => {
          // creates a temp folder to run CTA within (this is our cwd)
          const folder = f.temp()
          const appFolder = path.join(folder, appName)

          // runs CTA with all args set to avoid any prompts
          const cta = await execa(
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
              timeout: timeoutLong
            }
          )

          // check to make certain it didn't fail anywhere
          expect(cta.failed).toBe(false)
          expect(cta.timedOut).toBe(false)
          expect(cta.isCanceled).toBe(false)
          expect(cta.killed).toBe(false)
          expect(cta.signal).toBe(undefined)

          // run a tauri build to check if what we produced
          //  can actually create an app
          //  TODO long term we will want to hook this up to a real test harness
          //  and then run that test suite instead
          let opts: string[] = []
          if (manager === 'npm') {
            opts =
              recipe == 'vuecli'
                ? ['run', 'tauri:build']
                : ['run', 'tauri', '--', 'build']
          } else if (manager === 'yarn') {
            opts = recipe == 'vuecli' ? ['tauri:build'] : ['tauri', 'build']
          }
          const tauriBuild = await execa(manager, opts, {
            all: true,
            stdio: logOut,
            cwd: appFolder,
            timeout: timeoutLong
          })

          expect(tauriBuild.failed).toBe(false)
          expect(tauriBuild.timedOut).toBe(false)
          expect(tauriBuild.isCanceled).toBe(false)
          expect(tauriBuild.killed).toBe(false)
          expect(tauriBuild.signal).toBe(undefined)

          const packageFileOutput: {
            [k: string]: string | object
          } = JSON.parse(
            await fs.promises.readFile(
              path.join(appFolder, 'package.json'),
              'utf-8'
            )
          )
          expect(packageFileOutput['name']).toBe(appName)

          const assertCustom: { [k: string]: Function } = {
            vanillajs: () => {
              expect(packageFileOutput['scripts']).toMatchObject({
                tauri: 'tauri'
              })
            },
            reactjs: () => {
              expect(packageFileOutput['scripts']).toEqual(
                expect.objectContaining({
                  tauri: 'tauri'
                })
              )
            },
            reactts: () => {
              expect(packageFileOutput['scripts']).toEqual(
                expect.objectContaining({
                  tauri: 'tauri'
                })
              )
            },
            vite: () => {
              expect(packageFileOutput['scripts']).toEqual(
                expect.objectContaining({
                  tauri: 'tauri'
                })
              )
            },
            vuecli: () => {
              expect(packageFileOutput['scripts']).toEqual(
                expect.objectContaining({
                  'tauri:build': expect.anything(),
                  'tauri:serve': expect.anything()
                })
              )
            }
          }

          const getCustomAsserts = assertCustom[recipe]
          if (getCustomAsserts) getCustomAsserts()
        },
        timeoutLittleLonger
      )
    }
  )
})
