# Contributing

See the main document in the `.github` folder for general instructions. This document includes information regarding specific architecture choices.

# Development and Publishing

This package is primarily a CLI. This means that the output should not contain any Typescript, and only use the `bin` directory with `.js` (or files runnable directly in pure node) or the `dist` directory which includes processed Typescript files. Running `npm publish --dry-run` will give you a list of files and the package structure that are included when it is published. This may be helpful for understanding path requirements.

The `scripts` folder is included for use in dependency manager to install rustup.

Webpack is our current bundler of choice. It is not bundling the `node_modules` so our `package.json` dependency array needs to include anything required within the repository.
