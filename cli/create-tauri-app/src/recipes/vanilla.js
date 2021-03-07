const path = require("path");
const scaffe = require("scaffe");
const init = require("tauri/dist/api/init");
const { version } = require("tauri/package.json");

module.exports = (args) => {
  return new Promise((resolve, reject) => {
    const appName = args["_"][0];
    const templateDir = path.join(__dirname, "../templates/vanilla");
    const variables = {
      name: appName,
      tauri_version: version
    }

    scaffe.generate(templateDir, appName, variables, async (err) => {
      if(err){
        reject(err);
      }

      init({
        directory: appName,
        force: args.f || null,
        logging: args.l || null,
        tauriPath: args.t || null,
        appName: appName || args.A || null,
        customConfig: {
          tauri: {
            window: {
              title: appName,
            },
          },
          build: {
            devPath: "../dist",
            distDir: "../dist",
          },
        },
      });

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
    })
  })
}