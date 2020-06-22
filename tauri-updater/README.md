# Tauri Updater
---
> ⚠️ This project is a working project. Expect breaking changes.
---

The updater is focused on making Tauri's application updates **as safe
and transparent as updates to a website**.

Instead of publishing a feed of versions from which your app must select,
Tauri updates to the version your server tells it to. This allows you to
intelligently update your clients based on the request you give to Tauri.

The server can remotely drive behaviors like rolling back or phased rollouts.

The update JSON Tauri requests should be dynamically generated based on
criteria in the request, and whether an update is required.

Tauri's installer is also designed to be fault tolerant, and ensure that any updates installed are valid and safe.

# Configuration

Once you have your tauri project ready, you need to configure the updater.

Add this in tauri.config.js
```json
"updater": {
    "active": true,
    "endpoints": [
        "https://releases.myapp.com/{target}}/{current_version}}"
    ],
    "dialog": true,
    "pubkey": "",
}
```

The required keys are "active" and "endpoints", others are optional.

"active" must be a boolean. By default it's set to false.

"endpoints" must be an array. The string `{{target}}` and `{{current_version}}` are automatically replaced in the URL allowing you determine [server-side](#update-server-json-format) if an update is available. If multiple endpoints are specified, the updater will fallback if a server is not responding within the pre-defined timeout.

"dialog" if present must be a boolean. By default it's set to true. If enabled, [events](#events) are turned-off as the updater will handle everything. If you need the custom events, you MUST turn off the built-in dialog.

"pubkey" if present must be a valid public-key generated with tauri cli. See [Signing updates](#signing-updates).

## Update Requests

Tauri is indifferent to the request the client application provides for
update checking.

`Accept: application/json` is added to the request headers because Tauri is responsible for parsing the response.

For the requirements imposed on the responses and the body format of an update response see [Server Support](#server-support).

Your update request must *at least* include a version identifier so that the server can determine whether an update for this specific version is required.

It may also include other identifying criteria such as operating system version, to allow the server to deliver as fine grained an update as you would like.

How you include the version identifier or other criteria is specific to the server that you are requesting updates from. A common approach is to use query parameters, [Configuration](#configuration) shows an example of this.

## Built-in dialog

By default, updater use built-in dialog API from Tauri.

![New Update](https://i.imgur.com/6qGiIlJ.png)

The dialog first line of text is represented by the update `note` provided by the [server](#server-support).

If user accept, the download and install is initialized. The user will be then prompted to restart the application.

## Events

**Attention, you need to disable built-in dialog in your [tauri configuration](#configuration), otherwise events aren't emitted.**

To know when an update is ready to be installed, you can subscribe to these events:

### Listen New Update Available

Event : `update-available`

Emitted data:
```
version    Version announced by the server
date       Date announced by the server
body       Note announced by the server
```

### Rust
```rust
event::listen(String::from("update-available"), move |update| {
    println("New version available: {:?}", update);
})
```

### Javascript
```js
window.tauri.listen("update-available", function (res) {
    console.log("New version available: ", res);
}
```

### Emit Install and Download

You need to emit this event to initialize the download and listen the [install progress](#listen-install-progress).

Event : `updater-install`

### Rust
```rust
event::emit(String::from("updater-install"), None)
```

### Javascript
```js
window.tauri.emit("updater-install");
```

### Listen Install Progress

Event : `update-install-status`

Emitted data:
```
status    [PENDING/DONE]
```

PENDING is emitted when the download is started and DONE when the install is complete. You can then ask to restart the application.


### Rust
```rust
event::listen(String::from("update-install-status"), move |update| {
    println("Status change: {:?}", update);
})
```

### Javascript
```js
window.tauri.listen("update-install-status", function (res) {
    console.log("New status: ", res);
}
```

# Server Support

Your server should determine whether an update is required based on the
[Update Request](#update-requests) your client issues.

If an update is required your server should respond with a status code of [200 OK](http://tools.ietf.org/html/rfc2616#section-10.2.1) and include the [update JSON](#update-server-json-format) in the body. To save redundantly downloading the same version multiple times your server must not inform the client to update.

If no update is required your server must respond with a status code of [204 No Content](http://tools.ietf.org/html/rfc2616#section-10.2.5).

## Update Server JSON Format

When an update is available, Tauri expects the following schema in response to the update request provided:

```json
{
	"url": "https://mycompany.example.com/myapp/releases/myrelease.tar.gz",
    "version": "0.0.1",
    "notes": "Theses are some release notes",
    "pub_date": "2020-09-18T12:29:53+01:00",
    "signature": ""
}
```

The only required keys are "url" and "version", the others are optional.

"pub_date" if present must be formatted according to ISO 8601.

"signature" if present must be a valid signature generated with tauri cli. See [Signing updates](#signing-updates).

# Bundler (Artifacts)

The Tauri bundler will automatically generate update artifacts if the updater is enabled in your tauri.config.js

If the bundler can locate your private and pubkey, your update artifacts will be automatically signed.

The signature can be found in the `sig` file. Signature can be uploaded to github safely or made public as long as your private key is secure.

## MacOS

On MACOS we create a .tar.gz from the whole application. (.app)

```
target/release/bundle
└── osx
    └── app.app
    └── app.app.tar.gz (update bundle)
    └── app.app.tar.gz.sig (if signature enabled)
```

## Windows

On Windows we create a .zip from the MSI, when downloaded and validated, we run the MSI install.

```
target/release/bundle
└── win
    └── app.x64.msi
    └── app.x64.msi.zip (update bundle)
    └── app.x64.msi.zip.sig (if signature enabled)
```

## Linux

On Linux we create a .tar.gz from the AppImage.

```
target/release/bundle
└── appimage
    └── app.AppImage
    └── app.AppImage.tar.gz (update bundle)
    └── app.AppImage.tar.gz.sig (if signature enabled)
```

# Signing updates

We offer built-in signature to ensure your update are safe to be installed.

To sign your updates, you need two things.

The *Public key* (pubkey) should be added inside your tauri.config.js to validate the update archive before installing.

The *Private key* (privkey) is used to sign your update and should NEVER be shared with anyone. Also, if you lost this key, you'll NOT be able to publish a new update to the current user base (if pubkey is set in tauri.config.js). It's important to save it at a safe place and you can always access it.

To generate your keys you need to use the tauri cli. (will be built in tauri main CLI soon)

```bash
cargo tauri-sign-updates -g
```

You have multiple options available
```bash
OPTIONS:
    -f, --force                              Overwrite private key even if exist on the specified path
    -g, --generate-key                       Generate keypair to sign files
    -h, --help                               Prints help information
        --no-password                        Set empty password for your private key
    -P, --password <PASSWORD>                Set private key password when signing
        --private-key <STRING>               Load the private key from a string
    -p, --private-key-path <PATH>            Load the private key from a file
    -s, --sign <PATH>                        Sign the specified binary
    -w, --write-private-key <DESTINATION>    Write private key to a file
```

***

Environment variabled used to sign:

`TAURI_PRIVATE_KEY`  Path or String of your private key

`TAURI_KEY_PASSWORD`  Your private key password (optional)