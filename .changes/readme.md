# Changes

##### via https://github.com/jbolda/covector

As you create PRs and make changes that require a version bump, please add a new markdown file in this folder. You do not note the version _number_, but rather the type of bump that you expect: major, minor, or patch. The filename is not important, as long as it is a `.md`, but we recommend it represents the overall change for our sanity.

When you select the version bump required, you do _not_ need to consider dependencies. Only note the package with the actual change, and any packages that depend on that package will be bumped automatically in the process.

Use the following format:

```md
---
"api": patch
"tauri-bundler": patch
"tauri-cli": patch
"tauri.js": patch
"tauri-utils": patch
"tauri-macros": patch
"tauri-updater": patch
"tauri-build": patch
"tauri-codegen": patch
"tauri": patch
---

Change summary goes here
```

Summaries do not have a specific character limit, but are text only. These summaries are used within the (future implementation of) changelogs. They will give context to the change and also point back to the original PR if more details and context are needed.

Changes will be designated as a `major`, `minor` or `patch` as further described in [semver](https://semver.org/).

Given a version number MAJOR.MINOR.PATCH, increment the:

- MAJOR version when you make incompatible API changes,
- MINOR version when you add functionality in a backwards compatible manner, and
- PATCH version when you make backwards compatible bug fixes.

Additional labels for pre-release and build metadata are available as extensions to the MAJOR.MINOR.PATCH format, but will be discussed prior to usage (as extra steps will be necessary in consideration of merging and publishing).
