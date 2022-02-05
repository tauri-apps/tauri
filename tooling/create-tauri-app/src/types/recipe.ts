import { Answers, QuestionCollection } from 'inquirer'
import { PackageManager } from '../dependency-manager'
import { TauriBuildConfig } from './config'

export interface RecipeArgs {
  cwd: string
  cfg: TauriBuildConfig
  packageManager: PackageManager
  ci: boolean
  answers?: Answers
}

export interface Recipe {
  descriptiveName: { name: string; value: string }
  shortName: string
  configUpdate?: (args: RecipeArgs) => TauriBuildConfig
  extraNpmDependencies: string[]
  extraNpmDevDependencies: string[]
  extraQuestions?: (args: RecipeArgs) => QuestionCollection[]
  preInit?: (args: RecipeArgs) => Promise<void>
  postInit?: (args: RecipeArgs) => Promise<void>
}
