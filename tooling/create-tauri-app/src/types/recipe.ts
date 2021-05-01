import { Answers, QuestionCollection } from 'inquirer'
import { PackageManager } from '../dependency-manager'
import { TauriBuildConfig } from './config'

export interface RecipeArgs {
  cwd: string
  cfg: TauriBuildConfig
  packageManager: PackageManager
  ci: boolean
  // eslint-disable-next-line @typescript-eslint/no-invalid-void-type
  answers?: undefined | void | Answers
}

export interface Recipe {
  descriptiveName: string
  shortName: string
  configUpdate?: (args: RecipeArgs) => TauriBuildConfig
  extraNpmDependencies: string[]
  extraNpmDevDependencies: string[]
  extraQuestions?: (args: RecipeArgs) => QuestionCollection[]
  preInit?: (args: RecipeArgs) => Promise<void>
  postInit?: (args: RecipeArgs) => Promise<void>
}
