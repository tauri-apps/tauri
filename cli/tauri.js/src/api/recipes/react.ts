import { Recipe } from '.'
import { TauriBuildConfig } from '../../types/config'
import { spawnSync } from '../../helpers/spawn'
import logger from '../../helpers/logger'
import copyTemplates from '../../helpers/copy-templates'
import { resolve, join } from 'path'

const uiAppDir = 'app-ui'

const log = logger('react-recipe')

const completeLogMsg = `
  Your installation completed.
  To start, run yarn tauri dev
`

const afterCra = (): void => {
  copyTemplates({
    source: resolve(__dirname, '../../templates/recipes/react/'),
    scope: {},
    target: join(uiAppDir, './src/')
  })
  log(completeLogMsg)
}

const reactjs: Recipe = {
  descriptiveName: 'React.js',
  shortName: 'reactjs',
  configUpdate: (cfg: TauriBuildConfig): TauriBuildConfig => ({
    ...cfg,
    distDir: `../${uiAppDir}/build`,
    devPath: 'http://localhost:3000',
    beforeDevCommand: `yarn --cwd ${uiAppDir} start`,
    beforeBuildCommand: `yarn --cwd ${uiAppDir} build`
  }),
  extraNpmDevDependencies: ['create-react-app'],
  extraNpmDependencies: ['react'],
  postConfiguration: (cwd: string) => {
    spawnSync('yarn', ['create-react-app', uiAppDir], cwd)
    afterCra()
  }
}

const reactts: Recipe = {
  ...reactjs,
  descriptiveName: 'React with Typescript',
  shortName: 'reactts',
  extraNpmDependencies: ['typescript', '@types/node', '@types/react', '@types/react-dom', '@types/jest'],
  postConfiguration: (cwd: string) => {
    spawnSync('yarn', ['create-react-app', '--template', 'typescript', uiAppDir], cwd)
    afterCra()
  }
}

export {
  reactjs,
  reactts
}
