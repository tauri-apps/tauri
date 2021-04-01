import { Recipe } from "..";
import { TauriBuildConfig } from "../types/config";
import { join } from "path";
//@ts-ignore
import scaffe from "scaffe";
import { shell } from "../shell";

export const vanillajs: Recipe = {
  descriptiveName: "Vanilla.js",
  shortName: "vanillajs",
  configUpdate: (cfg) => ({
    ...cfg,
    distDir: `../dist`,
    devPath: `../dist`,
    beforeDevCommand: `yarn start`,
    beforeBuildCommand: `yarn build`,
  }),
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  preInit: async ({ cwd, cfg }) => {
    // we have an issue with mkdir?
    await shell("mkdir", [`\"${join(cwd, cfg.appName)}\"`]);
    const version = await shell("npm", ["view", "tauri", "version"], {
      stdio: "pipe",
    });
    const versionNumber = version.stdout.trim();
    await run(cfg, cwd, versionNumber);
  },
};

export const run = (args: TauriBuildConfig, cwd: string, version: string) =>
  new Promise((resolve, reject) => {
    const { appName } = args;
    const templateDir = join(__dirname, "../templates/vanilla");
    const variables = {
      name: appName,
      tauri_version: version,
    };

    scaffe.generate(
      templateDir,
      join(cwd, appName),
      variables,
      async (err: Error) => {
        if (err) {
          reject(err);
        }

        resolve({
          output: `
  change directory:
    $ cd ${appName}

  install dependencies:
    $ yarn # npm install

  run the app:
    $ yarn tauri dev # npm run tauri dev
        `,
        });
      }
    );
  });
