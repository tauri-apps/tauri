export interface CargoManifest {
  dependencies: { [k: string]: string | CargoManifestDependency }
  package: { version: string }
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
