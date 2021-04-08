import { Recipe } from "..";
import { TauriBuildConfig } from "./../types/config";
import { join } from "path";
//@ts-ignore
import scaffe from "scaffe";
import { shell } from "../shell";

const uiAppDir = "app-ui";

const completeLogMsg = `
  Your installation completed.
  To start, run yarn tauri dev
`;

const afterCra = async (outDir: string, appName: string, version: string) => {
  const templateDir = join(__dirname, "../src/templates/react");
  const variables = {
    name: appName,
    tauri_version: version,
  };

  return await scaffe.generate(
    templateDir,
    outDir,
    variables,
    async (err: Error) => {
      if (err) {
        console.error(err);
      } else {
        console.log(completeLogMsg);
      }
      return;
    }
  );
};

const reactjs: Recipe = {
  descriptiveName: "React.js",
  shortName: "reactjs",
  configUpdate: (cfg) => ({
    ...cfg,
    distDir: `../${uiAppDir}/build`,
    devPath: "http://localhost:3000",
    beforeDevCommand: `yarn --cwd ${uiAppDir} start`,
    beforeBuildCommand: `yarn --cwd ${uiAppDir} build`,
  }),
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  preInit: async ({ cwd, cfg }) => {
    // CRA creates the folder for you
    await shell("yarn", ["create", "react-app", `"${cfg.appName}"`], { cwd });
  },
  postInit: async ({ cwd, cfg }) => {
    const version = await shell("npm", ["view", "tauri", "version"], {
      stdio: "pipe",
    });
    const versionNumber = version.stdout.trim();
    await afterCra(cwd, cfg.appName, versionNumber);
  },
};

const reactts: Recipe = {
  ...reactjs,
  descriptiveName: "React with Typescript",
  shortName: "reactts",
  extraNpmDependencies: [
    "typescript",
    "@types/node",
    "@types/react",
    "@types/react-dom",
    "@types/jest",
  ],
  preInit: async ({ cwd, cfg }) => {
    // CRA creates the folder for you
    await shell(
      "yarn",
      ["create", "react-app", "--template", "typescript", `"${cfg.appName}"`],
      { cwd }
    );
  },
  postInit: async ({ cfg }) => {
    console.log(completeLogMsg);
  },
};

export { reactjs, reactts };
