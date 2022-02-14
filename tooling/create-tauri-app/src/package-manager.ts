import { shell } from './shell'

export interface PackageManager {
  /** Install npm packages */
  install: (
    packages: string[],
    options?: { dev?: boolean; cwd?: string }
  ) => Promise<void>

  /**
   * Run a npm package bin or a package.json script, if the pacakge manager is npm, it is always a package.json script so you'd have to make sure the script exists
   */
  run: (
    bin: string,
    args: string[],
    options?: { cwd?: string }
  ) => Promise<void>

  /**
   * Invokes create-* packages to bootstrap new projects
   *
   * @param createApp the create-* package to invoke without "create-" prefix, ex: "react-app" for "create-react-app"
   */
  create: (
    createApp: string,
    args: string[],
    options?: { cwd?: string }
  ) => Promise<void>
}

export function getPkgManagerFromUA(userAgent: string | undefined):
  | {
      name: string
      version: string
    }
  | undefined {
  if (!userAgent) return undefined
  const pkgSpec = userAgent.split(' ')[0]
  const pkgSpecArr = pkgSpec.split('/')
  return {
    name: pkgSpecArr[0],
    version: pkgSpecArr[1]
  }
}

export class Npm implements PackageManager {
  private version: number
  private log: boolean
  private ci: boolean
  constructor(version: number, options?: { ci?: boolean; log?: boolean }) {
    this.version = version
    this.log = options?.log ?? false
    this.ci = options?.ci ?? false
  }

  async install(
    packages: string[],
    options?: { dev?: boolean; cwd?: string | undefined } | undefined
  ) {
    switch (this.version) {
      case 8:
      case 7:
      case 6:
        await shell(
          'npm',
          ['install', options?.dev ? '--save-dev' : '', ...packages],
          { cwd: options?.cwd },
          this.log
        )
        break
      default:
        throw 'Unsupported npm version'
    }
  }

  async run(
    bin: string,
    args: string[],
    options?: { cwd?: string | undefined } | undefined
  ) {
    switch (this.version) {
      case 8:
      case 7:
      case 6:
        await shell(
          'npm',
          ['run', bin, '--', ...args],
          { cwd: options?.cwd },
          this.log
        )
        break
      default:
        throw 'Unsupported npm version'
    }
  }

  async create(
    createApp: string,
    args: string[],
    options?: { cwd?: string | undefined } | undefined
  ) {
    switch (this.version) {
      case 8:
      case 7:
        await shell(
          'npm',
          ['init', createApp, this.ci ? '--yes' : '', '--', ...args],
          {
            cwd: options?.cwd
          },
          this.log
        )
        break
      case 6:
        await shell(
          'npm',
          ['init', this.ci ? '--yes' : '', createApp, ...args],
          { cwd: options?.cwd },
          this.log
        )
        break
      default:
        throw 'Unsupported npm version'
    }
  }
}

export class Yarn implements PackageManager {}

export class Pnpm implements PackageManager {}
