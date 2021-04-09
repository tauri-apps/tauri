// SPDX-License-Identifier: Apache-2.0 OR MIT

export interface CargoManifest {
  dependencies: { [k: string]: string | CargoManifestDependency }
  package: { version: string; name: string; 'default-run': string }
  bin: Array<{
    name: string
    path: string
  }>
}

export interface CargoManifestDependency {
  version?: string
  path?: string
  features?: string[]
}

export interface CargoLock {
  package: [CargoLockPackage]
}

export interface CargoLockPackage {
  name: string
  version: string
}
