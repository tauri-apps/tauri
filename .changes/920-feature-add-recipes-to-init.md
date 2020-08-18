---
"tauri.js": minor
---

* Break out TauriBuildConfig interface from TauriConfig build property
* Create recipes. A recipe:
	* Updates the TauriBuildConfig during the init process
	* Specifies npm dev and production dependencies to be installed
	* Runs extra installation scripts
* Create React JS and React TS recipes
* Add new top level command `create`, which accepts a recipe as a CLI, or runs 
interactively, prompting for a recipe out of a menu of choices defined by `api/recipes/index`
* Refactor `init` command so that it is just an alias for `create` with no recipe
