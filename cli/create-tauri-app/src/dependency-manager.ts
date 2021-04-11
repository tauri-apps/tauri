import { ManagementType, Result } from "./types/deps";
import { shell } from "./shell";
import { existsSync } from "fs";
import { join } from "path";

export async function install({
  appDir,
  dependencies,
  devDependencies,
}: {
  appDir: string;
  dependencies: string[];
  devDependencies?: string[];
}) {
  return await manageDependencies(appDir, dependencies, devDependencies);
}

async function manageDependencies(
  appDir: string,
  dependencies: string[] = [],
  devDependencies: string[] = []
): Promise<{ result: Result; packageManager: string }> {
  const installedDeps = [...dependencies, ...devDependencies];
  console.log(`Installing ${installedDeps.join(", ")}...`);

  const packageManager = await usePackageManager(appDir);

  await installNpmDevPackage(devDependencies, packageManager, appDir);
  await installNpmPackage(dependencies, packageManager, appDir);

  const result: Result = new Map<ManagementType, string[]>();
  result.set(ManagementType.Install, installedDeps);

  return { result, packageManager };
}

async function usePackageManager(appDir: string): Promise<"yarn" | "npm"> {
  const hasYarnLockfile = existsSync(join(appDir, "yarn.lock"));
  let yarnChild;
  // try yarn first if there is a lockfile
  if (hasYarnLockfile) {
    yarnChild = await shell("yarn", ["--version"], { stdio: "pipe" });
    if (!yarnChild.failed) return "yarn";
  }

  // try npm then as the "default"
  const npmChild = await shell("npm", ["--version"], { stdio: "pipe" });
  if (!npmChild.failed) return "npm";

  // try yarn as maybe only yarn is installed
  if (yarnChild && !yarnChild.failed) return "yarn";

  // if we have reached here, we can't seem to run anything
  throw new Error(
    `Must have npm or yarn installed to manage dependencies. Is either in your PATH? We tried running in ${appDir}`
  );
}

async function installNpmPackage(
  packageNames: string[],
  packageManager: string,
  appDir: string
): Promise<void> {
  if (packageNames.length === 0) return;
  if (packageManager === "yarn") {
    await shell("yarn", ["add", packageNames.join(" ")], {
      cwd: appDir,
    });
  } else {
    await shell("npm", ["install", packageNames.join(" ")], {
      cwd: appDir,
    });
  }
}

async function installNpmDevPackage(
  packageNames: string[],
  packageManager: string,
  appDir: string
): Promise<void> {
  if (packageNames.length === 0) return;
  if (packageManager === "yarn") {
    await shell(
      "yarn",
      ["add", "--dev", "--ignore-scripts", packageNames.join(" ")],
      {
        cwd: appDir,
      }
    );
  } else {
    await shell(
      "npm",
      ["install", "--save-dev", "--ignore-scripts", packageNames.join(" ")],
      {
        cwd: appDir,
      }
    );
  }
}
