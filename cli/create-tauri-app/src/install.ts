import {
  installThese,
  installTheseDev,
} from "./dependency-manager/npm-packages";
import { Recipe } from ".";
import { Result } from "./dependency-manager/types";
import logger from "../../tauri.js/src/helpers/logger";

export async function installRecipeDependencies(
  recipe: Recipe
): Promise<Result> {
  const log = logger("recipe:install");

  log(`Installing dependencies for ${recipe.descriptiveName}`);
  return await installThese(recipe.extraNpmDependencies).then(
    async (results) =>
      await installTheseDev(recipe.extraNpmDevDependencies).then(
        (results2) =>
          new Map([
            ...Array.from(results.entries()),
            ...Array.from(results2.entries()),
          ])
      )
  );
}

export async function runRecipePostConfig(
  recipe: Recipe,
  cwd: string
): Promise<void> {
  const log = logger("recipe:postconfig");

  log(`Running post configuration for ${recipe.descriptiveName}`);
  return await new Promise(() => recipe.postConfiguration(cwd));
}
