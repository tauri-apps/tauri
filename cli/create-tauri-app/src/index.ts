import { map, identity, find } from "lodash";
import { TauriBuildConfig } from "./types/config";
import { reactjs, reactts } from "./recipes/react";
import { vanillajs } from "./recipes/vanilla";

import { shell } from "./shell";
export { shell };

export interface Recipe {
  descriptiveName: string;
  shortName: string;
  configUpdate?: (cfg: TauriBuildConfig) => TauriBuildConfig;
  extraNpmDevDependencies: string[];
  extraNpmDependencies: string[];
  preInit?: ({
    cwd,
    cfg,
  }: {
    cwd: string;
    cfg: TauriBuildConfig;
  }) => Promise<void>;
  postInit?: ({
    cwd,
    cfg,
  }: {
    cwd: string;
    cfg: TauriBuildConfig;
  }) => Promise<void>;
}

export const allRecipes: Recipe[] = [vanillajs, reactjs, reactts];

export const recipeNames: Array<[string, string]> = map(
  allRecipes,
  (r: Recipe) => [r.shortName, r.descriptiveName]
);

export const recipeByShortName = (name: string): Recipe | undefined =>
  find(allRecipes, (r: Recipe) => r.shortName === name);

export const recipeByDescriptiveName = (name: string): Recipe | undefined =>
  find(allRecipes, (r: Recipe) => r.descriptiveName === name);

export const recipeShortNames: string[] = map(
  allRecipes,
  (r: Recipe) => r.shortName
);

export const recipeDescriptiveNames: string[] = map(
  allRecipes,
  (r: Recipe) => r.descriptiveName
);
