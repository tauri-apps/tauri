# Changes
##### via https://github.com/jbolda/covector

As you create PRs and make changes that require a version bump, please add a new markdown file in this folder. You do not note the version *number*, but rather the type of bump that you expect: major, minor, or patch. The filename is not important, as long as it is a `.md`, but we recommend it represents the overall change for our sanity.

When you select the version bump required, you do *not* need to consider depedencies. Only note the package with the actual change, and any packages that depend on that package will be bumped automatically in the process.

Use the following format:
```md
---
"tauri.js": patch
"tauri": minor
---

Change summary goes here

```
