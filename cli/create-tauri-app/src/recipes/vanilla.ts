import { Recipe } from "..";
import { TauriBuildConfig } from "../types/config";
const path = require("path");
const scaffe = require("scaffe");
const { version } = require("tauri/package.json");

export const vanillajs: Recipe = {
  descriptiveName: "Vanilla.js",
  shortName: "vanillajs",
  configUpdate: (cfg) => ({
    ...cfg,
    distDir: `../dist`,
    devPath: "../dist",
    beforeDevCommand: `yarn start`,
    beforeBuildCommand: `yarn build`,
  }),
  extraNpmDevDependencies: [],
  extraNpmDependencies: ["react"],
  preInit: async ({ cwd, cfg }) => {
    await run(cfg, cwd);
  },
};

export const run = (args: TauriBuildConfig, cwd: string) =>
  new Promise((resolve, reject) => {
    const { appName } = args;
    const templateDir = path.join(__dirname, "../templates/vanilla");
    const variables = {
      name: appName,
      tauri_version: version,
    };

    scaffe.generate(templateDir, appName, variables, async (err: Error) => {
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
    });
  });
