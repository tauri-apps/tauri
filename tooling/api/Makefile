.DEFAULT_GOAL:=help

# Helper vars for pretty display
_TITLE := "\033[32m[%s]\033[0m %s\n"
_ERROR := "\033[31m[%s]\033[0m %s\n"

##
## Tauri API
## ─────────
##

help: ## ❓ Show this help.
	@printf "\n Available commands:\n\n"
	@grep -E '(^[a-zA-Z_-]+:.*?##.*$$)|(^##)' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-25s\033[0m %s\n", $$1, $$2}' | sed -e 's/\[32m## */[33m/'
.PHONY: help

bundle: ## Build the API with Deno to the "dist/index_deno_bundle.js" file.
	deno bundle src/index.ts --config=tsconfig.json --import-map=import_map.json dist/index_deno_bundle.js
.PHONY: bundle

emit: ## Build the API with Deno and outputs it to the terminal (because it causes errors)
	deno run --unstable --allow-read --allow-net build.ts
