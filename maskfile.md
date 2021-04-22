# Shorthand Commands

<!-- 
// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
-->

## prepare

> Setup all stuff needed for running the smoke tests

```sh
git clone --recursive git@github.com:tauri-apps/smoke-tests.git \
|| (cd smoke-tests && git pull origin dev; cd ..) 		# always prepare up-to-date smoke tests in case it's already available

cargo build --lib
cargo install cargo-web 			# used by rust/yew

. .scripts/setup.sh
```

```powershell
# Setup Environment to execute in the tauri directory.
$CWD = [Environment]::CurrentDirectory
Push-Location $MyInvocation.MyCommand.Path
[Environment]::CurrentDirectory = $PWD
# send git stderr to powershell stdout
$env:GIT_REDIRECT_STDERR = '2>&1'

# if the smoke-tests path doesn't exist.
if (-Not (Test-Path $CWD\smoke-tests -PathType Any)) {
  Start-Job -ScriptBlock {
    # Setup Environment to execute in the tauri directory.
    $CWD = [Environment]::CurrentDirectory
    Push-Location $MyInvocation.MyCommand.Path
    [Environment]::CurrentDirectory = $PWD

    #clone the smoke-tests repo into the smoke-tests folder
    git clone --recursive https://github.com/tauri-apps/smoke-tests.git $CWD\smoke-tests
  } | Receive-Job -AutoRemoveJob -Wait
}

# Enter the smoke-tests folder and pull the latest data from origin/dev
cd smoke-tests; git pull origin dev; cd ..

# build and install everything Rust related.
cargo build --lib
cargo install cargo-web

. .scripts/setup.ps1
```

## run

![tauri-mask-run-smoke-test](https://user-images.githubusercontent.com/4953069/75866011-00ed8600-5e37-11ea-9106-3cb104a05f80.gif)

### run smoke-test (name)

> Run specific smoketest in dev mode

```sh
shopt -s globstar

cd smoke-tests/**/$name 2>/dev/null \
|| cd smoke-tests/**/$name/$name 	# workaround for rust/yew/todomvc/todomvc

case "$PWD" in
*/node/*)
  yarn && yarn tauri:dev
;;
*/rust/*)
  cargo web deploy
  [ $name = `basename $(dirname $PWD)` ] && cd ..

  yarn add tauri@link:../../../tooling/cli.js
  yarn && yarn tauri dev
;;
*)
  echo unknown project $(dirname $name)/$name
;;
esac
```

```powershell
param(
  [string] $smoke_test_name
)

# Setup Environment to execute in the tauri directory.
$CWD = [Environment]::CurrentDirectory
Push-Location $MyInvocation.MyCommand.Path
[Environment]::CurrentDirectory = $PWD

# get the example paths.
$smoke_test_path = Get-ChildItem smoke-tests\*\*\$env:name

# if the example path is null get the todomvc path.
if ($smoke_test_path -eq $null) {
  $smoke_test_path = Get-ChildItem smoke-tests\*\*\*\$env:name\$env:name
}

# if the example path is still null get the helloworld example path.
if ($smoke_test_path -eq $null) {
  $smoke_test_path = Get-ChildItem smoke-tests\tauri\*\$env:name
}

# switch on the parent folder name.
switch ($smoke_test_path.parent) {
  # if node, run yarn.
  {"vanillajs" -Or "react" -Or "svelte" -Or "vue"} {
    cd $smoke_test_path.FullName; yarn; yarn tauri:dev
  }
  # if rust, run cargo web deploy
  "yew" {
    cd $smoke_test_path.FullName; cargo web deploy
  }
  # if tauri run the helloworld example from the tauri folder.
  "tauri" {
    cd $CWD/tauri; cargo run --bin helloworld
  }
  # transpiled are not supported yet.
  "transpiled" {
    Write-Output("Example not supported yet")
  }
}
```

## list

### list smoke tests

> List all available smoke tests

```sh
find smoke-tests/*/*/* -maxdepth 0 -type d -not -path '*.git*' \
-exec sh -c 'echo $(basename $(dirname {}))/$(basename {})' \;
```

```powershell
# Setup Environment to execute in the tauri directory.
$CWD = [Environment]::CurrentDirectory
Push-Location $MyInvocation.MyCommand.Path
[Environment]::CurrentDirectory = $PWD

# initialize the smoke-tests list.
$smoke_tests = @()

# get the helloworld smoke tests
$smoke_tests += Get-ChildItem smoke-tests/*/* -Filter helloworld
# get the rest of the smoke-tests.
$smoke_tests += Get-ChildItem smoke-tests/*/* -Directory -Exclude ('src*', 'public', 'test*', 'source', 'lib', 'web', 'dist', 'node_*')

# print out the smoke tests.
foreach($e in $smoke_tests) {
  Write-Output("$($e.Name):  $($e.Parent)/$($e.Name)")
}
```

## clean

> Remove installed dependencies and reset smoke tests in case something gone wrong

```sh
cargo clean

shopt -s globstar
rm -r **/node_modules

cd smoke-tests
git checkout -- . 	# discard all unstaged changes
git clean -dfX 		# remove all untracked files & directories
```

```powershell
# Setup Environment to execute in the tauri directory.
$CWD = [Environment]::CurrentDirectory
Push-Location $MyInvocation.MyCommand.Path
[Environment]::CurrentDirectory = $PWD

# clean up any artifacts.
cargo clean

# find any node_module folders.
$node_paths = Get-ChildItem -Path smoke-tests\ -Filter node_modules -Recurse -ErrorAction SilentlyContinue -Force

if (-Not $node_paths -eq $null) {
# delete all of the node_module folders.
  foreach ($path in $node_paths) {
    $path.Delete()
  }
  # enter the smoke-tests folder and remove any changes.
  cd $CWD/smoke-tests; git checkout -- .; git clean -dfX
}


```
