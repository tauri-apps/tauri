import { shell } from '../shell'
import { Recipe } from '../types/recipe'

export const ngcli: Recipe = {
  shortName: 'ngcli',
  descriptiveName: {
    name: 'Angular CLI (https://angular.io/cli)',
    value: 'ng-cli'
  },
  configUpdate: ({ cfg, pm }) => ({
    ...cfg,
    distDir: `../dist/${cfg.appName}`,
    devPath: 'http://localhost:4200',
    beforeDevCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} start`,
    beforeBuildCommand: `${pm.name === 'npm' ? 'npm run' : pm.name} build`
  }),
  preInit: async ({ cwd, cfg, pm, ci }) => {
    await shell(
      'npx',
      [
        ci ? '--yes' : '',
        '-p',
        'ng',
        'new',
        `${cfg.appName}`,
        `--package-manager=${pm.name}`
      ],
      {
        cwd
      }
    )
  },
  postInit: async ({ pm, cfg }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${pm.name === 'npm' ? 'npm run' : pm.name} tauri dev
    `)

    return await Promise.resolve()
  }
}
