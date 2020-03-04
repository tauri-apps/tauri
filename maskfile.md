# Shorthand Commands

## prepare
> Setup all stuffs needed for runing the examples

```sh
git clone --recursive git@github.com:tauri-apps/examples.git \
|| cd examples && git pull; cd .. 		# always prepare up-to-date examples in case it's already available
source .scripts/init_env.sh

cargo build
cargo install --path cli/tauri-bundler --force
cargo install cargo-web 			# used by example rust/yew

cd cli/tauri.js
yarn && yarn build
```

## run

![tauri-mask-run-example](https://user-images.githubusercontent.com/4953069/75866011-00ed8600-5e37-11ea-9106-3cb104a05f80.gif)

### run example (example)
> Run specific example in dev mode

```sh
source .scripts/init_env.sh
shopt -s globstar

cd examples/**/$example 2>/dev/null \
|| cd examples/**/$example/$example 	# workaround for rust/yew/todomvc/todomvc 

case "$PWD" in
*/node/*)
  yarn && yarn tauri:source dev
;;
*/rust/*)
  cargo web deploy
  [ $example = `basename $(dirname $PWD)` ] && cd ..

  yarn add tauri@link:../../../cli/tauri.js
  yarn && yarn tauri dev
;; 
*)
  echo unknown project $(dirname $example)/$example 
;; 
esac
```

## list

### list examples
> List all available examples

```sh
find examples/*/*/* -maxdepth 0 -type d -not -path '*.git*' \
-exec sh -c 'echo $(basename $(dirname {}))/$(basename {})' \;
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
