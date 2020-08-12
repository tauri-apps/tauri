import { ManagementType, Result } from "./types";
import {
  getNpmLatestVersion,
  getNpmPackageVersion,
  installNpmPackage,
  updateNpmPackage,
  semverLt,
} from "./util";
import logger from "../../helpers/logger";
import { resolve } from "../../helpers/app-paths";
import inquirer from "inquirer";
import { existsSync } from "fs";

const log = logger("dependency:npm-packages");

const dependencies = ["tauri"];

async function manageDependencies(
  managementType: ManagementType
): Promise<Result> {
  const installedDeps = [];
  const updatedDeps = [];

  if (existsSync(resolve.app("package.json"))) {
    for (const dependency of dependencies) {
      const currentVersion = await getNpmPackageVersion(dependency);
      if (currentVersion === null) {
        log(`Installing ${dependency}...`);
        installNpmPackage(dependency);
        installedDeps.push(dependency);
      } else if (managementType === ManagementType.Update) {
        const latestVersion = getNpmLatestVersion(dependency);
        if (semverLt(currentVersion, latestVersion)) {
          const inquired = await inquirer.prompt([
            {
              type: "confirm",
              name: "answer",
              message: `[NPM]: "${dependency}" latest version is ${latestVersion}. Do you want to update?`,
              default: false,
            },
          ]);
          if (inquired.answer) {
            log(`Updating ${dependency}...`);
            updateNpmPackage(dependency);
            updatedDeps.push(dependency);
          }
        } else {
          log(`"${dependency}" is up to date`);
        }
      } else {
        log(`"${dependency}" is already installed`);
      }
    }
  }

  const result: Result = new Map<ManagementType, string[]>();
  result.set(ManagementType.Install, installedDeps);
  result.set(ManagementType.Update, updatedDeps);

  return result;
}

async function install(): Promise<Result> {
  return await manageDependencies(ManagementType.Install);
}

async function update(): Promise<Result> {
  return await manageDependencies(ManagementType.Update);
}

export { install, update };
