export interface CargoToml {
  dependencies: { [k: string]: string | CargoTomlDependency }
  package: { version: string }
}

export interface CargoTomlDependency {
  version?: string
  path?: string
  features?: string[]
}
