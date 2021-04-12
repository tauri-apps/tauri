import { Recipe } from "..";
import { join } from "path";
//@ts-ignore
import scaffe from "scaffe";
import { shell } from "../shell";

const completeLogMsg = `
  Your installation completed.
  To start, run yarn tauri dev
`;

const afterCra = async (cwd: string, appName: string, version: string) => {
  const templateDir = join(__dirname, "../src/templates/react");
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

const reactjs: Recipe = {
  descriptiveName: "React.js",
  shortName: "reactjs",
  configUpdate: (cfg) => ({
    ...cfg,
    distDir: `../build`,
    devPath: "http://localhost:3000",
    beforeDevCommand: `npm start`,
    beforeBuildCommand: `npm build`,
  }),
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  preInit: async ({ cwd, cfg }) => {
    // CRA creates the folder for you
    await shell("npx", ["create-react-app", `${cfg.appName}`], { cwd });
    const version = await shell("npm", ["view", "tauri", "version"], {
      stdio: "pipe",
    });
    const versionNumber = version.stdout.trim();
    await afterCra(cwd, cfg.appName, versionNumber);
  },
  postInit: async ({ cfg }) => {
    console.log(completeLogMsg);
  },
};

const reactts: Recipe = {
  ...reactjs,
  descriptiveName: "React with Typescript",
  shortName: "reactts",
  extraNpmDependencies: [],
  preInit: async ({ cwd, cfg }) => {
    // CRA creates the folder for you
    await shell(
      "npx",
      ["create-react-app", "--template", "typescript", `${cfg.appName}`],
      { cwd }
    );
  },
  postInit: async ({ cfg }) => {
    console.log(completeLogMsg);
  },
};

export { reactjs, reactts };
