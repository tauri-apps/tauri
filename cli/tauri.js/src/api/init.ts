import { TauriConfig } from 'types'
import { inject } from '../template'
import { resolve } from 'path'
import toml, { JsonMap } from '@tauri-apps/toml'
import { readFileSync, writeFileSync } from 'fs'
import { kebabCase } from 'lodash'
import { CargoManifest } from 'src/types/cargo'

module.exports = (args: {
  directory: string
  force: false | 'conf' | 'template' | 'all'
  logging: boolean
  tauriPath?: string
  customConfig?: Partial<TauriConfig>
  appName?: string
}): boolean => {
  const injectResult = inject(
    args.directory,
    'all',
    {
      force: args.force,
      logging: args.logging,
      tauriPath: args.tauriPath
    },
    args.customConfig
  )
  if (args.appName) {
    const manifestPath = resolve(args.directory, 'src-tauri/Cargo.toml')
    const cargoManifest = (toml.parse(
      readFileSync(manifestPath).toString()
    ) as unknown) as CargoManifest
    const binName = kebabCase(args.appName)
    cargoManifest.package.name = binName
    cargoManifest.package['default-run'] = binName
    if (cargoManifest.bin?.length) {
      cargoManifest.bin[0].name = binName
    }
    writeFileSync(
      manifestPath,
      toml.stringify((cargoManifest as unknown) as JsonMap)
    )
  }
  return injectResult
}
