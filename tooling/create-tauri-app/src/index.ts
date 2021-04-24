// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { TauriBuildConfig } from './types/config'
import { reactjs, reactts } from './recipes/react'
import { vuecli } from './recipes/vue-cli'
import { vanillajs } from './recipes/vanilla'
import { vite } from './recipes/vite'
import { PackageManager } from './dependency-manager'

export { shell } from './shell'
export { install, checkPackageManager } from './dependency-manager'
export { addTauriScript } from './helpers/add-tauri-script'

export interface Recipe {
  descriptiveName: string
  shortName: string
  configUpdate?: ({
    cfg,
    packageManager
  }: {
    cfg: TauriBuildConfig
    packageManager: PackageManager
  }) => TauriBuildConfig
  extraNpmDependencies: string[]
  extraNpmDevDependencies: string[]
  preInit?: ({
    cwd,
    cfg,
    packageManager
  }: {
    cwd: string
    cfg: TauriBuildConfig
    packageManager: PackageManager
  }) => Promise<void>
  postInit?: ({
    cwd,
    cfg,
    packageManager
  }: {
    cwd: string
    cfg: TauriBuildConfig
    packageManager: PackageManager
  }) => Promise<void>
}

export const allRecipes: Recipe[] = [vanillajs, reactjs, reactts, vite, vuecli]

export const recipeNames: Array<[string, string]> = allRecipes.map((r) => [
  r.shortName,
  r.descriptiveName
])

export const recipeByShortName = (name: string): Recipe | undefined =>
  allRecipes.find((r) => r.shortName === name)

export const recipeByDescriptiveName = (name: string): Recipe | undefined =>
  allRecipes.find((r) => r.descriptiveName === name)

export const recipeShortNames = allRecipes.map((r) => r.shortName)

export const recipeDescriptiveNames = allRecipes.map((r) => r.descriptiveName)
