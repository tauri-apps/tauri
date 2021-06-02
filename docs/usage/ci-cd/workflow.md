---
id: workflow
title: Workflow
---

## Continuous Integration

Github Actions has two triggers of which we make heavy use: `push` and `pull_request`. Every commit that made to the repo is a `push`. When you open a pull request from a branch (call it `great_feature`) to another branch (our working branch, `dev`), each commit to `great_feature` would possibly trigger both of these events. We can use a filter to focus on the events we care about though. In our workflows, we only PR (pull request) the `dev` and `master` branches. This means that if we filter to only the `dev` and `master` branches on commit, we will only run that workflow when we _merge_ a PR. A merged PR typically only occurs once a day or less so this will be a good fit for the longer running tests, e.g. the smoke tests in our case. Below is how that might look.

Unit tests:

```yml
# these run fast so we can have them run on any commit
name: unit tests
on:
  pull_request:
  push:
    branches:
      - dev
      - master
```

Smoke tests:

```yml
# these run slower so we run only on merges to dev or master branch
name: smoke tests
on:
  push:
    branches:
      - dev
      - master
```

Tauri operates off the `dev` branch as default, and merges to `master` for release. With these Github Actions set up, we will run the unit tests on every commit to an open PR (see `pull_request`). When that PR is merged into `dev`, we will run both the unit tests and the smoke tests.

## Continuous Deployment

### Introduction to immutable checksum

It is not only possible, but trivial to modify release notes and artifacts after it has been published on Github. While there are very valid reasons for doing this, it is not exactly a totally trustworthy method - i.e. you have no guarantee that what you are reading is really reflective of the underlying truth or the tarballs. It is technically possible to change downloads over the wire or in the box or change checksums in targeted attacks. What we are seeking to accomplish is a best case scenario where:

1. Human error is reduced to a minimum, but humans are still integral in the actual release
2. Machine built assets, changelogs and attached security audits are verifiable with checksums that are published in an immutable, globally available store.

To this end we fashioned a workflow shown below. As it stands now, we have #3 through #6 implemented. We manually do #2 which then feeds into #3 and kicks off the rest of the automatic workflow.

1. a human pushes to dev through a pull request (can happen any number of times)
   - pull request includes a changeset file describing the change and required version bump
2. a pull request is created (or updated) to include the change and version bump
   - this pull request stays open and will be force pushed until it gets merged (and published)
   - increase the version number based on changesets
   - delete all changeset files
3. a codeowner merges the publish PR to dev (no direct push permissible for anyone)
   - all tests (unit, e2e, smoke tests) are run on the PR
   - failures prevent the publish so they must pass before merge
4. merge to dev triggers release sequence
   - changes are squashed and a PR is opened against master
5. when PR to master is merged...
   - vulnerability audit (crates and yarn) and output saved
   - checksums and metadata and output saved
   - packages are published on npm/cargo, tarball/zip created
   - release is created for each package that had updates (if version isn't changed, build skips the publish steps)
   - output from audit/checksums is piped into the release body
   - tarball / zip attached to release
   - async process to publish to IOTA tangle (feeless) via release tag [note: still have things to resolve here]
6. release is complete
   - master has updated code and tagged
   - GitHub release has tarballs, checksums, and changelog (may have multiple releases if more than one package published) [note: is part of step 2 and is not yet implemented]

### Next Steps

Next steps may include transferring and publishing the built assets to additional places:

1. Tauri's private verdaccio
2. IPFS
3. PureOS Gitlab
4. GitHub Packages

We can also do some interesting things like signing our releases, including a hash in the release and/or even publishing this information on a blockchain that it can be easily verified. Publishing on the blockchain is another avenue to increase the confidence that what is seen on GitHub matches what you have downloaded. The IOTA foundation created a Github Action which will publish a release to their blockchain. This has shown promise, but he gave a couple errors to tackle still.
