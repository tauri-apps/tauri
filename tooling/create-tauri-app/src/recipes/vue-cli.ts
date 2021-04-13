import { Recipe } from "..";
import { join } from "path";
//@ts-ignore
import { shell } from "../shell";

const completeLogMsg = `
  Your installation completed.
  To start, run yarn tauri:serve
`;

const vuecli: Recipe = {
  descriptiveName: "Vue CLI",
  shortName: "vuecli",
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  configUpdate: ({ cfg }) => cfg,
  preInit: async ({ cwd, cfg }) => {
    // Vue CLI creates the folder for you
    await shell("npx", ["@vue/cli", "create", `${cfg.appName}`], { cwd });
    await shell(
      "npx",
      [
        "@vue/cli",
        "add",
        "tauri",
        "--appName",
        `${cfg.appName}`,
        "--windowTitle",
        `${cfg.windowTitle}`,
      ],
      {
        cwd: join(cwd, cfg.appName),
      }
    );
  },
  postInit: async () => {
    console.log(completeLogMsg);
  },
};

export { vuecli };
