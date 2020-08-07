import { Recipe } from '.'
import { TauriBuildConfig } from '../../types/config'
import { spawnSync } from '../../helpers/spawn'
import logger from '../../helpers/logger'

const uiAppDir = 'app-ui'

const log = logger('react-recipe')

const completeLogMsg = `
  Your installation completed.
  Run these commands to start:
    For development server (provides UI): cd ${uiAppDir}; yarn start
    For tauri (runs desktop app) in this directory: yarn tauri dev  
`

const reactjs: Recipe = {
  descriptiveName: 'React.js',
  shortName: 'reactjs',
  configUpdate: (cfg: TauriBuildConfig): TauriBuildConfig => ({
    ...cfg,
    distDir: `../${uiAppDir}/build`,
    devPath: 'http://localhost:3000'
  }),
  extraNpmDevDependencies: ['create-react-app'],
  extraNpmDependencies: ['react'],
  postConfiguration: (cwd: string) => {
    spawnSync('yarn', ['create-react-app', uiAppDir], cwd)
    log(completeLogMsg)
  }
}

const reactts: Recipe = {
  ...reactjs,
  descriptiveName: 'React with Typescript',
  shortName: 'reactts',
  extraNpmDependencies: ['typescript', '@types/node', '@types/react', '@types/react-dom', '@types/jest'],
  postConfiguration: (cwd: string) => {
    spawnSync('yarn', ['create-react-app', '--template', 'typescript', uiAppDir], cwd)
    log(completeLogMsg)
  }
}

export {
  reactjs,
  reactts
}
