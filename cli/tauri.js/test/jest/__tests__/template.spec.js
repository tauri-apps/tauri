const fixtureSetup = require('../fixtures/app-test-setup');
const { resolve } = require('path');
const { writeFileSync, readFileSync } = require('fs');

describe('[CLI] tauri.js template', () => {
  it('init a project and builds it', async () => {
    const cwd = process.cwd();
    const fixturePath = resolve(__dirname, '../fixtures/empty');
    const tauriFixturePath = resolve(fixturePath, 'src-tauri');

    fixtureSetup.initJest('empty');

    process.chdir(fixturePath);

    const init = require('api/init');
    init({
      directory: process.cwd(),
      force: 'all',
      tauriPath: resolve(__dirname, '../../../../..')
    });

    process.chdir(tauriFixturePath);

    const manifestPath = resolve(tauriFixturePath, 'Cargo.toml');
    const manifestFile = readFileSync(manifestPath).toString();
    writeFileSync(manifestPath, `workspace = { }\n\n${manifestFile}`);

    const build = require('api/build');
    await build({
      tauri: {
        bundle: {
          targets: ['deb', 'osx', 'msi', 'appimage'] // we can't bundle dmg on CI so we remove it here
        }
      }
    }).promise.then(() => {
      writeFileSync('a.b', 'finished');
      process.chdir(cwd);
    });
  });
});
