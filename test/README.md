# TESTING

While we get the test harnesses unified, which will happen before the 1.0.0 stable release, we are using the manual testing approach of building real Quasar apps and vetting them for functionality.

In this folder there are two of them, distinguished by the fact that one uses a localhost server (cloudish) and the other is a pure rust host (trollbridge). These follow the principles laid out in the Design Patterns. 

## PRE-RELEASE NOTE
Until upstream `quasarframework/quasar` is merged and released as `v1.1.0`, we are linking against a local git clone.

Here is how to do that (assuming you are in the base of this repo):
```bash
$ yarn
$ git clone https://github.com/lucasfernog/quasar.git # premerge usage of our working fork
$ cd quasar
$ git checkout feature/tauri-package # premerge usage of our working fork
$ cd app && yarn
$ cd ../ui
$ yarn && yarn build
$ cd ../../test/cloudish 
$ yarn add ../../quasar/app ../../quasar/ui  # or yarn install if a previous version of yarn
```
