# Tauri Releasing Handbook

This handbook contains information about our release pipeline and how to deal with common issues.
This document is mainly intended for team members responsible for maintaining the project.

- [Covector](#covector)
- [Version Pull Request](#version-pull-request)
- [Releasing and Publishing](#releasing-and-publishing)
- [Publishing failed, what to do?](#publishing-failed-what-to-do)

## Covector

We use [`covector`](https://github.com/jbolda/covector) to manage our version bumps and release pipeline.
It can be configured in [`.changes/config.json`](../.changes/config.json) which includes how each package should be published step by step.

Some packages can't be published directly using `covector` as it requires to be built on a matrix of platforms
such as `tauri-cli` prebuilt binaries which is published using [publish-cli-rs.yml](./workflows/publish-cli-rs.yml)
and `@tauri-apps/cli` native Node.js modules which is published using using [publish-cli-js.yml](./workflows/publish-cli-js.yml)
both of which are triggered after `covector` has created a github release for both of them, see `Trigger @tauri-apps/cli publishing workflow`
and `Trigger tauri-cli publishing workflow` steps in [covector-version-or-publish.yml](./workflows/covector-version-or-publish.yml)

## Version Pull Request

On each pull request merged, [covector-version-or-publish.yml](./workflows/covector-version-or-publish.yml) workflow will run, and:

When there're change files inside `.changes` folder and they're not all included in `pre.json` (usually this is only when we are in `-alpha` to `-rc` phase), it will open/update an `Apply Version Updates From Current Changes` PR (https://github.com/tauri-apps/tauri/pull/11029 for example) that bumps all packages based on current existing change files and generate `CHANGELOG.md` entries. see `Create Pull Request With Versions Bumped` step in [covector-version-or-publish.yml](./workflows/covector-version-or-publish.yml).

Otherwise, covector will start to publish packages configured in [`.changes/config.json`](../.changes/config.json).

## Releasing and Publishing

Releasing can be as easy as merging the version pull request but here is a checklist to follow:

- [ ] Double check that every package is bumped correctly and there are no accidental major or minor being released unless that is indeed the intention.
- [ ] Make sure that there are no pending or unfinished [covector-version-or-publish.yml](./workflows/covector-version-or-publish.yml) workflow runs.
- [ ] Sign the Version PR before merging as we require signed commits
  - [ ] `git fetch --all`
  - [ ] `git checkout release/version-updates`
  - [ ] `git commit --amend -S`
  - [ ] `git push --force`
- [ ] Approve and merge the version pull request

## Publishing failed, what to do?

It is possible and due to many factors that one or many packages release can fail to release, there is no reason to panic, we can fix this.

Did all of the packages fail to release?

- yes?
  - [ ] `git checkout -b revert-branch`
  - [ ] `git revert HEAD~1`
- no?
  - [ ] `git checkout -b revert-branch`
  - [ ] `git revert HEAD~1 --no-commit`
  - [ ] Edit the commit and revert only changes related to packages that failed to publish
  - [ ] `git revert --continue`

Then:

- [ ] Make a pull request with reverted changes, get it approved and merged
- [ ] Fix the issue that caused releases to fail in another PR, get it approved and merged
- [ ] Repeat the release process again.
