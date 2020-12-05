/* -- NOTES --
 * help message
 * triggers whenever there's '-h' or '--help' flag
**/

module.exports = () => {
  console.log(`create-tauri-app v0.1.0

  Usage
    $ yarn create tauri-app <app-name> # npm create-tauri-app <app-name>
  Options
    -v, --version               displays the Tauri CLI version
        --recipe <recipe-name>  add that framework boilderplate (react|vue|svelte|none) (defaults to none)
    -f, --force                 force on non-empty directory
    -h, --help                  displays this message
  `);
}
