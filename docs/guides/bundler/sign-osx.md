---
title: How to code-sign and notorize a OSX .dmg file with GitHub Actions
sidebar_label: OSX Code-signing with GitHub Actions
---

import Alert from '@theme/Alert'

## Intro

Tauri has a smooth code-signing & notarization functionality built directly into the bundler and configured via the `tauri.conf.json` 

This guide will give a brief overview of how to sign an application, and how to get the app notarized with Apple. All in a GitHub action. 

## Prerequisits 
- OSX - This will be needed to create/export the certificate.
- [Apple Developer Program](https://developer.apple.com/programs/) subscription
- [Developer ID Application](https://developer.apple.com/developer-id/) certificate
  - see [this](https://localazy.com/blog/how-to-automatically-sign-macos-apps-using-github-actions#reference) guide for additional help
- Working Tauri application, being built and published via GitHub Actions, as shown in [tauri-action](https://github.com/tauri-apps/tauri-action)  

<Alert title="Note" icon="info-alt">
If you are not utilizing GitHub Actions to perform builds of OSX DMGs, you will need to ensure the environment variable `CI=true` exists. For more information refer to [Issue #592](https://github.com/tauri-apps/tauri/issues/592).
</Alert>

## GitHub Secrets

We will need to add a few GitHub secrets for the proper configuration of the GitHub Action. These can be named however you would like, but we must assign them to the correct Tauri variables, so keep them as relevant as possible. 
- You can view [this](https://docs.github.com/en/actions/reference/encrypted-secrets) guide for how to add GitHub secrets. 

The secrets I used are as follows

| GitHub Secrets | Value for Variable |
|     :---:      |        :---:            |
|APPLE_CERTIFICATE| Base64 encoded version of your .p12 certificate. You can find a guide [here](https://localazy.com/blog/how-to-automatically-sign-macos-apps-using-github-actions#reference)|
|APPLE_CERTIFICATE_PASSWORD|Certificate password used on creation of certificate|
|APPLE_IDENTITY_ID|"Developer ID Application: Your Company, Inc (XXXXXXXXX)" shown in your keychain. you can also use `security find-identity -v -p codesigning` on OSX to find this identity |
|APPLE_ID|this is the email used to request the certificate|
APPLE_PASSWORD|This is an app-specific password, that must also be created by the same account used to request the certificate. Guide [here](https://support.apple.com/en-ca/HT204397)| 

Once we have established the GitHub Secrets we will need to make some modifications to our GitHub publish action in `.github/workflows/main.yml` 


---
### Workflow Modifications

All we will have to do from here is assign the GitHub secrets to the proper environment variables. 
```
ENABLE_CODE_SIGNING: ${{ secrets.APPLE_CERTIFICATE }}
APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_IDENTITY_ID }}
APPLE_ID: ${{ secrets.APPLE_ID }}
APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
```

If you are using the tauri-action publish template, then your result should look similar the the `env:` portion below. 
```
name: "publish"
on:
  push:
    branches:
      - release

jobs:
  publish-tauri:
    strategy:
      fail-fast: false
      matrix:
        platform: [macos-latest, ubuntu-latest, windows-latest]

    runs-on: ${{ matrix.platform }}
    steps:
    - uses: actions/checkout@v2
    - name: setup node
      uses: actions/setup-node@v2
      with:
        node-version: 12
    - name: install Rust stable
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: install webkit2gtk (ubuntu only)
      if: matrix.platform == 'ubuntu-latest'
      run: |
        sudo apt-get update
        sudo apt-get install -y webkit2gtk-4.0
    - name: install app dependencies and build it
      run: yarn && yarn build
    - uses: tauri-apps/tauri-action@v0
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        ENABLE_CODE_SIGNING: ${{ secrets.APPLE_CERTIFICATE }}
        APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
        APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
        APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_IDENTITY_ID }}
        APPLE_ID: ${{ secrets.APPLE_ID }}
        APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
      with:
        tagName: app-v__VERSION__ # the action automatically replaces \_\_VERSION\_\_ with the app version
        releaseName: "App v__VERSION__"
        releaseBody: "See the assets to download this version and install."
        releaseDraft: true
        prerelease: false
```
