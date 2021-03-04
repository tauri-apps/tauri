import { map, identity, find } from "lodash";
import { TauriBuildConfig } from "./types/config";
import { reactjs, reactts } from "./recipes/react";

export interface Recipe {
  descriptiveName: string;
  shortName: string;
  configUpdate: (cfg: TauriBuildConfig) => TauriBuildConfig;
  extraNpmDependencies: string[];
  extraNpmDevDependencies: string[];
  postConfiguration: (cwd: string) => void;
}

const none = {
  descriptiveName: "No recipe",
  shortName: "none",
  configUpdate: identity,
  extraNpmDependencies: [],
  extraNpmDevDependencies: [],
  postConfiguration: (cwd: string) => {},
};

export const allRecipes: Recipe[] = [none, reactjs, reactts];

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
