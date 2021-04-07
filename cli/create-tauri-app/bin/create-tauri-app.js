const parseArgs = require("minimist");
const inquirer = require("inquirer");
const { resolve, join } = require("path");
const {
  recipeShortNames,
  recipeDescriptiveNames,
  recipeByDescriptiveName,
  recipeByShortName,
  shell,
} = require("../dist/");
const { dir } = require("console");

/**
 * @type {object}
 * @property {boolean} h
 * @property {boolean} help
 * @property {boolean} v
 * @property {boolean} version
 * @property {string|boolean} f
 * @property {string|boolean} force
 * @property {boolean} l
 * @property {boolean} log
 * @property {boolean} d
 * @property {boolean} directory
 * @property {string} r
 * @property {string} recipe
 */
const createTauriApp = async (cliArgs) => {
  const argv = parseArgs(cliArgs, {
    alias: {
      h: "help",
      v: "version",
      f: "force",
      l: "log",
      d: "directory",
      b: "binary",
      t: "tauri-path",
      A: "app-name",
      W: "window-title",
      D: "dist-dir",
      P: "dev-path",
      r: "recipe",
    },
    boolean: ["h", "l", "ci"],
  });

  if (argv.help) {
    printUsage();
    return 0;
  }

  if (argv.v) {
    console.log(require("../package.json").version);
    return false; // do this for node consumers and tests
  }

  if (argv.ci) {
    return runInit(argv);
  } else {
    return getOptionsInteractive(argv).then((responses) =>
      runInit(argv, responses)
    );
  }
};

function printUsage() {
  console.log(`
  Description
    Starts a new tauri app from a "recipe" or pre-built template.
  Usage
    $ yarn create tauri-app <app-name> # npm create-tauri-app <app-name>
  Options
    --help, -h           Displays this message
    -v, --version        Displays the Tauri CLI version
    --ci                 Skip prompts
    --force, -f          Force init to overwrite [conf|template|all]
    --log, -l            Logging [boolean]
    --directory, -d      Set target directory for init
    --binary, -b         Optional path to a tauri binary from which to run init
    --app-name, -A       Name of your Tauri application
    --window-title, -W   Window title of your Tauri application
    --dist-dir, -D       Web assets location, relative to <project-dir>/src-tauri
    --dev-path, -P       Url of your dev server
    --recipe, -r         Add UI framework recipe. None by default. 
                         Supported recipes: [${recipeShortNames.join("|")}]
    `);
}

const getOptionsInteractive = (argv) => {
  let defaultAppName = argv.A || "tauri-app";

  return inquirer
    .prompt([
      {
        type: "input",
        name: "appName",
        message: "What is your app name?",
        default: defaultAppName,
        when: !argv.A,
      },
      {
        type: "input",
        name: "tauri.window.title",
        message: "What should the window title be?",
        default: "Tauri App",
        when: () => !argv.W,
      },
      {
        type: "list",
        name: "recipeName",
        message: "Would you like to add a UI recipe?",
        choices: recipeDescriptiveNames,
        default: "No recipe",
        when: () => !argv.r,
      },
    ])
    .then((answers) =>
      inquirer
        .prompt([
          {
            type: "input",
            name: "build.devPath",
            message: "What is the url of your dev server?",
            default: "http://localhost:4000",
            when: () =>
              (!argv.P && !argv.p && answers.recipeName === "No recipe") ||
              argv.r === "none",
          },
          {
            type: "input",
            name: "build.distDir",
            message:
              'Where are your web assets (HTML/CSS/JS) located, relative to the "<current dir>/src-tauri" folder that will be created?',
            default: "../dist",
            when: () =>
              (!argv.D && answers.recipeName === "No recipe") ||
              argv.r === "none",
          },
        ])
        .then((answers2) => ({ ...answers, ...answers2 }))
    )
    .catch((error) => {
      if (error.isTtyError) {
        // Prompt couldn't be rendered in the current environment
        console.log(
          "It appears your terminal does not support interactive prompts. Using default values."
        );
        runInit();
      } else {
        // Something else when wrong
        console.error("An unknown error occurred:", error);
      }
    });
};

async function runInit(argv, config = {}) {
  const {
    appName,
    recipeName,
    tauri: {
      window: { title },
    },
  } = config;

  let recipe;

  if (recipeName !== undefined) {
    recipe = recipeByDescriptiveName(recipeName);
  } else if (argv.r) {
    recipe = recipeByShortName(argv.r);
  }

  let buildConfig = {
    distDir: argv.D,
    devPath: argv.P,
  };

  if (recipe !== undefined) {
    buildConfig = recipe.configUpdate(buildConfig);
  }

  const directory = argv.d || process.cwd();
  const cfg = {
    ...buildConfig,
    appName: appName || argv.A,
    windowTitle: title || argv.w,
  };
  // note that our app directory is reliant on the appName and
  // generally there are issues if the path has spaces (see Windows)
  const appDirectory = join(directory, cfg.appName);

  if (recipe.preInit) {
    console.log("===== running initial command(s) =====");
    await recipe.preInit({ cwd: directory, cfg });
  }

  const initArgs = [
    ["--app-name", cfg.appName],
    ["--window-title", cfg.windowTitle],
    ["--dist-dir", cfg.distDir],
    ["--dev-path", cfg.devPath],
  ].reduce((final, argSet) => {
    if (argSet[1]) {
      return final.concat([argSet[0], `\"${argSet[1]}\"`]);
    } else {
      return final;
    }
  }, []);

  if (!argv.b) {
    console.log("===== adding tauri =====");
    await shell("yarn add tauri --dev", {
      cwd: appDirectory,
    });
  }

  console.log("===== running tauri init =====");
  const binary = !argv.b ? "yarn" : resolve(appDirectory, argv.b);
  await shell(binary, ["tauri", "init", ...initArgs], {
    ...(!argv.b ? {} : { shell: true }),
    cwd: appDirectory,
  });

  if (recipe.postInit) {
    console.log("===== running final command(s) =====");
    await recipe.postInit({
      cwd: appDirectory,
      cfg,
    });
  }
}

createTauriApp(process.argv.slice(2)).catch((err) => {
  console.error(err);
});
