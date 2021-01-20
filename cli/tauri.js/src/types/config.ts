/**
 * A CLI argument definition
 */
export interface CliArg {
  /**
   * the short version of the argument, without the preceding -
   * NOTE: Any leading - characters will be stripped, and only the first non - character will be used as the short version
   */
  short?: string
  /**
   * the unique argument name
   */
  name: string
  /**
   * the argument description which will be shown on the help information
   * typically, this is a short (one line) description of the arg
   */
  description?: string
  /**
   * the argument long description which will be shown on the help information
   * typically this a more detailed (multi-line) message that describes the argument
   */
  longDescription?: string
  /**
   * specifies that the argument takes a value at run time.
   * NOTE: values for arguments may be specified in any of the following methods
   * - Using a space such as -o value or --option value
   * - Using an equals and no space such as -o=value or --option=value
   * - Use a short and no space such as -ovalue
   */
  takesValue?: boolean
  /**
   * specifies that the argument may appear more than once.
   * for flags, this results in the number of occurrences of the flag being recorded. For example -ddd or -d -d -d would count as three occurrences.
   * for options there is a distinct difference in multiple occurrences vs multiple values. For example, --opt val1 val2 is one occurrence, but two values. Whereas --opt val1 --opt val2 is two occurrences.
   */
  multiple?: boolean
  /**
   * specifies that the argument may appear more than once.
   */
  multipleOccurrences?: boolean
  /**
   * specifies a list of possible values for this argument. At runtime, the CLI verifies that only one of the specified values was used, or fails with an error message.
   */
  possibleValues?: string[]
  /**
   * specifies the minimum number of values for this argument.
   * for example, if you had a -f <file> argument where you wanted at least 2 'files' you would set `minValues: 2`, and this argument would be satisfied if the user provided, 2 or more values.
   */
  minValues?: number
  /**
   * specifies the maximum number of values are for this argument.
   * for example, if you had a -f <file> argument where you wanted up to 3 'files' you would set .max_values(3), and this argument would be satisfied if the user provided, 1, 2, or 3 values.
   */
  maxValues?: number
  /**
   * sets whether or not the argument is required by default
   * required by default means it is required, when no other conflicting rules have been evaluated
   * conflicting rules take precedence over being required.
   */
  required?: boolean
  /**
   * sets an arg that override this arg's required setting
   * i.e. this arg will be required unless this other argument is present
   */
  requiredUnless?: string
  /**
   * sets args that override this arg's required setting
   * i.e. this arg will be required unless all these other arguments are present
   */
  requiredUnlessAll?: string[]
  /**
   * sets args that override this arg's required setting
   * i.e. this arg will be required unless at least one of these other arguments are present
   */
  requiredUnlessOne?: string[]
  /**
   * sets a conflicting argument by name
   * i.e. when using this argument, the following argument can't be present and vice versa
   */
  conflictsWith?: string
  /**
   * the same as conflictsWith but allows specifying multiple two-way conflicts per argument
   */
  conflictsWithAll?: string
  /**
   * sets an argument by name that is required when this one is present
   * i.e. when using this argument, the following argument must be present
   */
  requires?: string
  /**
   * sets multiple arguments by names that are required when this one is present
   * i.e. when using this argument, the following arguments must be present
   */
  requiresAll?: string[]
  /**
   * allows a conditional requirement with the signature [arg: string, value: string]
   * the requirement will only become valid if `arg`'s value equals `${value}`
   */
  requiresIf?: [string, string]
  /**
   * allows specifying that an argument is required conditionally with the signature [arg: string, value: string]
   * the requirement will only become valid if the `arg`'s value equals `${value}`.
   */
  requiredIf?: [string, string]
  /**
   * requires that options use the --option=val syntax
   * i.e. an equals between the option and associated value
   */
  requireEquals?: boolean
  /**
   * The positional argument index, starting at 1.
   *
   * The index refers to position according to other positional argument.
   * It does not define position in the argument list as a whole. When utilized with multiple=true,
   * only the last positional argument may be defined as multiple (i.e. the one with the highest index).
   */
  index?: number
}

/**
 * describes a CLI configuration
 */
export interface CliConfig {
  /**
   * list of args for the command
   */
  args?: CliArg[]
  /**
   * command description which will be shown on the help information
   */
  description?: string
  /**
   * command long description which will be shown on the help information
   */
  longDescription?: string
  /**
   * adds additional help information to be displayed in addition to auto-generated help
   * this information is displayed before the auto-generated help information.
   * this is often used for header information
   */
  beforeHelp?: string
  /**
   * adds additional help information to be displayed in addition to auto-generated help
   * this information is displayed after the auto-generated help information
   * this is often used to describe how to use the arguments, or caveats to be noted.
   */
  afterHelp?: string
  /**
   * list of subcommands of this command
   *
   * subcommands are effectively sub-apps, because they can contain their own arguments, subcommands, usage, etc.
   * they also function just like the app command, in that they get their own auto generated help and usage
   */
  subcommands?: { [name: string]: CliConfig }
}

export interface TauriBuildConfig {
  /**
   * the path to the app's dist dir
   * this path must contain your index.html file
   */
  distDir: string
  /**
   * the app's dev server URL, or the path to the directory containing an index.html to open
   */
  devPath: string
  /**
   * a shell command to run before `tauri dev` kicks in
   */
  beforeDevCommand?: string
  /**
   * a shell command to run before `tauri build` kicks in
   */
  beforeBuildCommand?: string
  withGlobalTauri?: boolean
}

/**
 * Tauri configuration
 */
export interface TauriConfig {
  /**
   * build/dev configuration
   */
  build: TauriBuildConfig
  /**
   * the context of the current `tauri dev` or `tauri build`
   */
  ctx: {
    /**
     * whether we're building for production or not
     */
    prod?: boolean
    /**
     * whether we're running on the dev environment or not
     */
    dev?: boolean
    /**
     * the target of the compilation (see `rustup target list`)
     */
    target?: string
    /**
     * whether the app should be built on debug mode or not
     */
    debug?: boolean
    /**
     * defines we should exit the `tauri dev` process if a Rust code error is found
     */
    exitOnPanic?: boolean
  }
  /**
   * tauri root configuration object
   */
  tauri: {
    /**
     * app's CLI definition
     */
    cli?: CliConfig
    /**
     * the embedded server configuration
     */
    embeddedServer: {
      /**
       * whether we should use the embedded-server or the no-server mode
       */
      active?: boolean
      /**
       * the embedded server port number or the 'random' string to generate one at runtime
       */
      port?: number | 'random' | undefined
    }
    /**
     * tauri bundler configuration
     */
    bundle: {
      /**
       * whether we should build your app with tauri-bundler or plain `cargo build`
       */
      active?: boolean
      /**
       * the bundle targets, currently supports ["deb", "osx", "msi", "appimage", "dmg"] or "all"
       */
      targets?: string | string[]
      /**
       * the app's identifier
       */
      identifier: string
      /**
       * the app's icons
       */
      icon: string[]
      /**
       * app resources to bundle
       * each resource is a path to a file or directory
       * glob patterns are supported
       */
      resources?: string[]
      externalBin?: string[]
      copyright?: string
      category?: string
      shortDescription?: string
      longDescription?: string
      deb?: {
        depends?: string[]
        useBootstrapper?: boolean
      }
      osx?: {
        frameworks?: string[]
        minimumSystemVersion?: string
        license?: string
        useBootstrapper?: boolean
      }
      exceptionDomain?: string
    }
    updater: {
      active?: boolean
      pubkey: string
      endpoints: string[]
    }
    allowlist: {
      all: boolean
      [index: string]: boolean
    }
    window: {
      title: string
      width?: number
      height?: number
      resizable?: boolean
      fullscreen?: boolean
    }
    security: {
      csp?: string
    }
    inliner: {
      active?: boolean
    }
  }
  plugins?: {
    [name: string]: {
      [key: string]: any
    }
  }
  /**
   * Whether or not to enable verbose logging
   */
  verbose?: boolean
}

export default TauriConfig
