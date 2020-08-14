import { TauriConfig } from 'types';
import { merge } from 'webpack-merge';
import Runner from '../runner';
import getTauriConfig from '../helpers/tauri-config';
import logger from '../helpers/logger';
import chalk from 'chalk';
import { platform } from 'os';
import { resolve } from 'path';
import { sync as spawnSync } from 'cross-spawn';

const error = logger('tauri:dev', chalk.red);

interface DevResult {
  promise: Promise<void>;
  runner: Runner;
}

module.exports = (config: TauriConfig): DevResult => {
  if (platform() === 'win32') {
    const child = spawnSync('powershell', [
      resolve(__dirname, '../../scripts/is-admin.ps1')
    ]);
    const response = String(child.output[1]).replace('\n', '').trim();
    if (response === 'True') {
      error(
        "Administrator privileges detected. Tauri doesn't work when running as admin, see https://github.com/Boscop/web-view/issues/96"
      );
      process.exit(1);
    }
  }

  const tauri = new Runner();
  const tauriConfig = getTauriConfig(
    merge(
      {
        ctx: {
          debug: true,
          dev: true
        }
      } as any,
      config as any
    ) as TauriConfig
  );

  return {
    runner: tauri,
    promise: tauri.run(tauriConfig)
  };
};
