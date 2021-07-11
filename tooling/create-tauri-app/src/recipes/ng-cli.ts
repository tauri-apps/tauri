import { PackageManager } from '../dependency-manager'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const addAdditionalPackage = async (
  packageManager: PackageManager,
  cwd: string,
  projectName: string,
  packageName: string
) => {
  if (packageManager === 'yarn') {
    await shell('yarn', ['ng', 'add', packageName], {cwd: `${cwd}/${projectName}`});
  } else {
    await shell('npm', ['run', '--prefix', `${cwd}`, 'ng', 'add', packageName]);
  }
}

const ngcli: Recipe = {
  descriptiveName: {
    name: 'Angular CLI (https://angular.io/cli)',
    value: 'ng-cli'
  },
  shortName: 'ngcli',
  extraNpmDependencies: [],
  extraNpmDevDependencies: [],
  configUpdate: ({ cfg }) => ({
    ...cfg,
    distDir: `../dist/${cfg.appName}`,
    devPath: 'http://localhost:4200'
  }),
  extraQuestions: ({ ci }) => {
    return [
      {
        type: 'confirm',
        name: 'material',
        message: 'Add Angular Material (https://material.angular.io/)?',
        default: 'No',
        validate: async (input) => {
          return input.toLowerCase === 'yes'
        },
        loop: false,
        when: !ci
      },
      {
        type: 'confirm',
        name: 'eslint',
        message:
          'Add Angular ESLint (https://github.com/angular-eslint/angular-eslint)?',
        default: 'No',
        validate: async (input) => {
          return input.toLowerCase === 'yes'
        },
        loop: false,
        when: !ci
      }
    ]
  },
  preInit: async ({ cwd, cfg }) => {
    // Angular CLI creates the folder for you
    await shell('npx', ['-p', '@angular/cli', 'ng', 'new', `${cfg.appName}`], {
      cwd
    })
  },
  postInit: async ({ cwd, cfg, packageManager, answers }) => {
    if (answers) {
      if (answers.material) {
        await addAdditionalPackage(packageManager, cwd, cfg.appName, '@angular/material')
      }

      if (answers.eslint) {
        await addAdditionalPackage(packageManager, cwd, cfg.appName, '@angular-eslint/schematics')
      }
    }

    console.log(`
      Your installation completed. Change directory to \`${cwd}\`.
      To start, run ${packageManager} start and ${
      packageManager === 'yarn' ? 'yarn' : 'npm run'
    } tauri dev
    `);

    return await Promise.resolve()
  }
}

export { ngcli }
