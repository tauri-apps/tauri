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
    const version = await shell("npm", ["view", "tauri", "version"], {
      stdio: "pipe",
    });
    const versionNumber = version.stdout.trim();
    await run(cfg, cwd, versionNumber);
  },
  postInit: async ({ cfg }) => {
    console.log(`
      change directory:
        $ cd ${cfg.appName}
    
      install dependencies:
        $ yarn # npm install
    
      run the app:
        $ yarn tauri dev # npm run tauri dev
            `);
  },
};

export const run = async (
  args: TauriBuildConfig,
  cwd: string,
  version: string
) => {
  const { appName } = args;
  const templateDir = join(__dirname, "../src/templates/vanilla");
  const variables = {
    name: appName,
    tauri_version: version,
  };

  try {
    await scaffe.generate(templateDir, join(cwd, appName), {
      overwrite: true,
      variables,
    });
  } catch (err) {
    console.log(err);
  }
};
