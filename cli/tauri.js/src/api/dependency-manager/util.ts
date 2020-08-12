import https from "https";
import { IncomingMessage } from "http";
import { spawnSync } from "../../helpers/spawn";
import { sync as crossSpawnSync } from "cross-spawn";
import { appDir, resolve as appResolve } from "../../helpers/app-paths";
import { existsSync } from "fs";
import semver from "semver";

const BASE_URL = "https://docs.rs/crate/";

async function getCrateLatestVersion(crateName: string): Promise<string> {
  return await new Promise((resolve, reject) => {
    const url = `${BASE_URL}${crateName}`;
    https.get(url, (res: IncomingMessage) => {
      if (res.statusCode !== 302 || !res.headers.location) {
        reject(res);
      } else {
        const version = res.headers.location.replace(url + "/", "");
        resolve(version);
      }
    });
  });
}

function getNpmLatestVersion(packageName: string): string {
  const child = crossSpawnSync("npm", ["show", packageName, "version"], {
    cwd: appDir,
  });
  return String(child.output[1]).replace("\n", "");
}

async function getNpmPackageVersion(
  packageName: string
): Promise<string | null> {
  return await new Promise((resolve) => {
    const child = crossSpawnSync(
      "npm",
      ["list", packageName, "version", "--depth", "0"],
      { cwd: appDir }
    );
    const output = String(child.output[1]);
    // eslint-disable-next-line security/detect-non-literal-regexp
    const matches = new RegExp(packageName + "@(\\S+)", "g").exec(output);
    if (matches?.[1]) {
      resolve(matches[1]);
    } else {
      resolve(null);
    }
  });
}

function installNpmPackage(packageName: string): void {
  const usesYarn = existsSync(appResolve.app("yarn.lock"));
  if (usesYarn) {
    spawnSync("yarn", ["add", packageName], appDir);
  } else {
    spawnSync("npm", ["install", packageName], appDir);
  }
}

function updateNpmPackage(packageName: string): void {
  const usesYarn = existsSync(appResolve.app("yarn.lock"));
  if (usesYarn) {
    spawnSync("yarn", ["upgrade", packageName, "--latest"], appDir);
  } else {
    spawnSync("npm", ["install", `${packageName}@latest`], appDir);
  }
}

function padVersion(version: string): string {
  let count = (version.match(/\./g) ?? []).length;
  while (count < 2) {
    count++;
    version += ".0";
  }
  return version;
}

function semverLt(first: string, second: string): boolean {
  return semver.lt(padVersion(first), padVersion(second));
}

export {
  getCrateLatestVersion,
  getNpmLatestVersion,
  getNpmPackageVersion,
  installNpmPackage,
  updateNpmPackage,
  padVersion,
  semverLt,
};
