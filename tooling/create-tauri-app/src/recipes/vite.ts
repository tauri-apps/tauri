import { Recipe } from "..";
import { join } from "path";
import { readdirSync } from "fs";
//@ts-ignore
import scaffe from "scaffe";
import { shell } from "../shell";
import inquirer from "inquirer";

const afterViteCA = async (
  cwd: string,
  appName: string,
  version: string,
  template: string
) => {
  const templateDir = join(__dirname, `../src/templates/vite/${template}`);
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

const vite: Recipe = {
  descriptiveName: "Vite backed recipe",
  shortName: "vite",
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../dist`,
    devPath: "http://localhost:3000",
    beforeDevCommand: `${packageManager === "yarn" ? "yarn" : "npm run"} start`,
    beforeBuildCommand: `${
      packageManager === "yarn" ? "yarn" : "npm run"
    } build`,
  }),
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  preInit: async ({ cwd, cfg, packageManager }) => {
    try {
      const { template } = await inquirer.prompt([
        {
          type: "list",
          name: "template",
          message: "Which vite template would you like to use?",
          choices: readdirSync(join(__dirname, "../src/templates/vite")),
          default: "vue",
        },
      ]);

      // Vite creates the folder for you
      if (packageManager === "yarn") {
        await shell(
          "yarn",
          [
            "create",
            "@vitejs/app",
            `${cfg.appName}`,
            "--template",
            `${template}`,
          ],
          {
            cwd,
          }
        );
      } else {
        await shell(
          "npm",
          [
            "init",
            "@vitejs/app",
            `${cfg.appName}`,
            "--template",
            `${template}`,
          ],
          {
            cwd,
          }
        );
      }

      const version = await shell("npm", ["view", "tauri", "version"], {
        stdio: "pipe",
      });
      const versionNumber = version.stdout.trim();
      await afterViteCA(cwd, cfg.appName, versionNumber, template);
    } catch (error) {
      if (error.isTtyError) {
        // Prompt couldn't be rendered in the current environment
        console.log(
          "It appears your terminal does not support interactive prompts. Using default values."
        );
      } else {
        // Something else went wrong
        console.error("An unknown error occurred:", error);
      }
    }
  },
  postInit: async ({ packageManager }) => {
    console.log(`
    Your installation completed.
    To start, run ${packageManager === "yarn" ? "yarn" : "npm run"} tauri dev
  `);
  },
};

export { vite };
