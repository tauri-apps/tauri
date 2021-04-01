import { Recipe } from "..";
import { TauriBuildConfig } from "./../types/config";
import { shell } from "../shell";

const uiAppDir = "app-ui";

const completeLogMsg = `
  Your installation completed.
  To start, run yarn tauri dev
`;

const afterCra = (): void => {
  // copyTemplates({
  //   source: resolve(__dirname, "../../templates/recipes/react/"),
  //   scope: {},
  //   target: join(uiAppDir, "./src/"),
  // });
  console.log(completeLogMsg);
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
  postInit: async () => {
    afterCra();
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
  postInit: async () => {
    afterCra();
  },
};

export { reactjs, reactts };
