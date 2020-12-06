# Shorthand Commands

## prepare

> Setup all stuffs needed for running the examples

```sh
git clone --recursive git@github.com:tauri-apps/examples.git \
|| (cd examples && git pull origin dev; cd ..) 		# always prepare up-to-date examples in case it's already available

cargo build
cargo install --path cli/tauri-bundler --force
cargo install cargo-web 			# used by example rust/yew

cd cli/bindings/tauri.js
yarn && yarn build
```

```powershell
# Setup Environment to execute in the tauri directory.
$CWD = [Environment]::CurrentDirectory
Push-Location $MyInvocation.MyCommand.Path
[Environment]::CurrentDirectory = $PWD
# send git stderr to powershell stdout
$env:GIT_REDIRECT_STDERR = '2>&1'

# if the examples path doesn't exist.
if (-Not (Test-Path $CWD\examples -PathType Any)) {
  Start-Job -ScriptBlock {
    # Setup Environment to execute in the tauri directory.
    $CWD = [Environment]::CurrentDirectory
    Push-Location $MyInvocation.MyCommand.Path
    [Environment]::CurrentDirectory = $PWD

    #clone the examples repo into the examples folder
    git clone --recursive https://github.com/tauri-apps/examples.git $CWD\examples
  } | Receive-Job -AutoRemoveJob -Wait
}

# Enter the examples folder and pull the latest data from origin/dev
cd examples; git pull origin dev; cd ..

# set the env vars.
$env:TAURI_DIST_DIR = Resolve-Path $dist_path
$env:TAURI_DIR = Resolve-Path $src_path

# build and install everything Rust related.
cargo build
cargo install --path cli\tauri-bundler --force
cargo install cargo-web

# install the tauri Node CLI and transpile the TS version of the API.
cd cli\tauri.js
yarn; yarn build;
```

## run

![tauri-mask-run-example](https://user-images.githubusercontent.com/4953069/75866011-00ed8600-5e37-11ea-9106-3cb104a05f80.gif)

### run example (example)

> Run specific example in dev mode

```sh
shopt -s globstar

cd examples/**/$example 2>/dev/null \
|| cd examples/**/$example/$example 	# workaround for rust/yew/todomvc/todomvc

case "$PWD" in
*/node/*)
  yarn && yarn tauri:dev
;;
*/rust/*)
  cargo web deploy
  [ $example = `basename $(dirname $PWD)` ] && cd ..

  yarn add tauri@link:../../../cli/bindings/tauri.js
  yarn && yarn tauri dev
;;
*)
  echo unknown project $(dirname $example)/$example
;;
esac
```

```powershell
param(
  [string] $example_name
)

# Setup Environment to execute in the tauri directory.
$CWD = [Environment]::CurrentDirectory
Push-Location $MyInvocation.MyCommand.Path
[Environment]::CurrentDirectory = $PWD

# Invoke the command script.
Invoke-Expression -Command .scripts\init_env.ps1

# get the example paths.
$example_path = Get-ChildItem examples\*\*\$env:example

# if the example path is null get the todomvc path.
if ($example_path -eq $null) {
  $example_path = Get-ChildItem examples\*\*\*\$env:example\$env:example
}

# if the example path is still null get the communication example path.
if ($example_path -eq $null) {
  $example_path = Get-ChildItem examples\tauri\*\$env:example
}

# switch on the parent folder name.
switch ($example_path.parent) {
  # if node, run yarn.
  {"vanillajs" -Or "react" -Or "svelte" -Or "vue"} {
    cd $example_path.FullName; yarn; yarn tauri:dev
  }
  # if rust, run cargo web deploy
  "yew" {
    cd $example_path.FullName; cargo web deploy
  }
  # if tauri run the communication example from the tauri folder.
  "tauri" {
    cd $CWD/tauri/examples/communication/src-tauri; cargo run
  }
  # transpiled are not supported yet.
  "transpiled" {
    Write-Output("Example not supported yet")
  }
}
```

## list

### list examples

> List all available examples

```sh
find examples/*/*/* -maxdepth 0 -type d -not -path '*.git*' \
-exec sh -c 'echo $(basename $(dirname {}))/$(basename {})' \;
```

```powershell
# Setup Environment to execute in the tauri directory.
$CWD = [Environment]::CurrentDirectory
Push-Location $MyInvocation.MyCommand.Path
[Environment]::CurrentDirectory = $PWD

# initialize the examples list.
$examples = @()

# get the communication example
$examples += Get-ChildItem examples/*/* -Filter communication
# get the rest of the examples.
$examples += Get-ChildItem examples/*/* -Directory -Exclude ('src*', 'public', 'test*', 'source', 'lib', 'web', 'dist', 'node_*')

# print out the examples.
foreach($e in $examples) {
  Write-Output("$($e.Name):  $($e.Parent)/$($e.Name)")
}
```

## clean

> Remove installed dependencies and reset examples in case something gone wrong

```sh
cargo uninstall tauri-bundler
cargo clean

shopt -s globstar
rm -r **/node_modules

cd examples
git checkout -- . 	# discard all unstaged changes
git clean -dfX 		# remove all untracked files & directories
```

```powershell
# Setup Environment to execute in the tauri directory.
$CWD = [Environment]::CurrentDirectory
Push-Location $MyInvocation.MyCommand.Path
[Environment]::CurrentDirectory = $PWD

# uninstall the bundler and clean up any artifacts.
cargo uninstall tauri-bundler
cargo clean

# find any node_module folders.
$node_paths = Get-ChildItem -Path examples\ -Filter node_modules -Recurse -ErrorAction SilentlyContinue -Force

if (-Not $node_paths -eq $null) {
# delete all of the node_module folders.
  foreach ($path in $node_paths) {
    $path.Delete()
  }
  # enter the examples folder and remove any changes.
  cd $CWD/examples; git checkout -- .; git clean -dfX
}


```
