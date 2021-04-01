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

const afterCra = async (args: TauriBuildConfig, version: string) => {
  const { appName } = args;
  const templateDir = join(__dirname, "../templates/react");
  const variables = {
    name: appName,
    tauri_version: version,
  };

  console.log(completeLogMsg);
  return scaffe.generate(templateDir, appName, variables);
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
  extraNpmDevDependencies: ["create-react-app"],
  extraNpmDependencies: ["react"],
  preInit: async ({ cwd, cfg }) => {
    // CRA creates the folder for you
    await shell("yarn", ["create", "react-app", cfg.appName], { cwd });
  },
  postInit: async ({ cfg }) => {
    const version = await shell("npm", ["view", "tauri.js", "version"]);
    const versionNumber = version.stdout.trim();
    await afterCra(cfg, versionNumber);
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
      ["create", "react-app", "--template", "typescript", cfg.appName],
      { cwd }
    );
  },
  postInit: async ({ cfg }) => {
    console.log(completeLogMsg);
  },
};

export { reactjs, reactts };
