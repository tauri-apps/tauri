const path = require("path");
const scaffe = require("scaffe");
const init = require("tauri/dist/api/init");

module.exports = (args) => {
  return new Promise((resolve, reject) => {
    const appName = args["_"][0];
    const templateDir = path.join(__dirname, "../templates/vanilla");
  
    scaffe.generate(templateDir, appName, {name: appName}, async (err) => {
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
          },
        },
      });

      resolve({
        appDir: appName,
      });
    })
  })
}